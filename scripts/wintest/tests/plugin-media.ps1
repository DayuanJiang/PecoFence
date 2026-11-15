# 插件的媒体接口 (pf.media, pf.apps) with the now-playing plugin of assets/now-playing: what Edge
# plays (assets/media-page: a silent tone with a title, an artist, an album and a cover) shows in
# the fence with its time, the fence's buttons pause, play and skip it, a running spotify.exe gets
# its line, `app` comes only to the plugin that lists the program, and nothing is read while the
# fences are hidden. Screenshots: nothing playing, playing, paused. Either build.
#   run.sh --bin target/wintest-bin-debug scripts/wintest/tests/plugin-media.ps1
$Kit = 'C:\pf\kit\assets'
$Page = "$PfRun\media-page"
$EdgeExe = "${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe"

function CellRects($w) {
  $v = DumpVal $w 'script_cells'
  if (-not $v -or $v -eq '-') { return @() }
  return @($v -split ';' | ForEach-Object {
    $pair = $_ -split ':'; $n = $pair[1] -split ','
    [pscustomobject]@{ i = [int]$pair[0]; l = [double]$n[0]; t = [double]$n[1]; r = [double]$n[2]; b = [double]$n[3] }
  })
}
# Clicks row `row`, cell `cell` of the fence titled `title` as drawn now.
function ClickCell($title, $row, $cell) {
  $w = DumpOf $title 5
  $r = @(CellRects $w | Where-Object { $_.i -eq $row * 100 + $cell }) | Select-Object -First 1
  if (-not $r) { Log "no cell $row/$cell in $(DumpVal $w 'script_cells')"; return $false }
  $sc = [double](DumpVal $w 'content_y') / 36.0
  $x = [int]($w.l + [double](DumpVal $w 'content_x') + ($r.l + $r.r) / 2 * $sc)
  $y = [int]($w.t + [double](DumpVal $w 'content_y') + ($r.t + $r.b) / 2 * $sc)
  [G]::Click($x, $y); Start-Sleep -Milliseconds 600; ParkMouse
  return $true
}
function Rows($fid) { return @((FenceOut $fid) -split "`n") }
function WaitOut($fid, $pattern, $sec = 10) {
  for ($k = 0; $k -lt $sec * 4; $k++) { if ((FenceOut $fid) -match $pattern) { return $true }; Start-Sleep -Milliseconds 250 }
  return $false
}
# The row of the buttons: the one after the time.
function ButtonsRow($fid) { $r = Rows $fid; for ($i = 0; $i -lt $r.Count; $i++) { if ($r[$i] -match '\d+:\d\d / \d+:\d\d') { return $i + 1 } }; return -1 }
# Where it is, in seconds, as the fence shows it.
function Shown($fid) { if ((FenceOut $fid) -match '(\d+):(\d\d) / ') { return [int]$Matches[1] * 60 + [int]$Matches[2] }; return -1 }
function EdgeTitle() { return "$(@(Get-Process msedge -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -like 'PecoFence media test*' } | Select-Object -First 1).MainWindowTitle)" }
function WaitEdge($pattern, $sec = 8) {
  for ($k = 0; $k -lt $sec * 4; $k++) { if ((EdgeTitle) -match $pattern) { return $true }; Start-Sleep -Milliseconds 250 }
  return $false
}
function FenceShot($name, $title) {
  $w = DumpOf $title 5
  if (-not $w) { return }
  ParkMouse; Start-Sleep -Milliseconds 500
  (Crop $name ($w.l - 10) ($w.t - 10) ($w.r - $w.l + 20) ($w.b - $w.t + 20)).Dispose()
}

Case 'setup: the plugin in two fences, a running spotify.exe, nothing playing'
CloseWindows
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
# Placed before the start: PecoFence reads the installed plugins once as it starts.
$script:Config = "$PfApp\config"
PfPlacePlugin "$Kit\now-playing" 'now-playing'
# The same plugin with msedge.exe in its apps in place of spotify.exe.
PfPlacePlugin "$Kit\now-playing" 'now-playing-edge'
$pj = "$script:Config\plugins\now-playing-edge\1.0.0\plugin.json"
[IO.File]::WriteAllText($pj, ([IO.File]::ReadAllText($pj) -replace '"spotify.exe"', '"msedge.exe"'))
# A stand-in for Spotify: ping under the name spotify.exe.
Copy-Item "$env:WINDIR\System32\PING.EXE" "$PfRun\spotify.exe"
$script:Spotify = Start-Process "$PfRun\spotify.exe" -ArgumentList '-n 900 127.0.0.1' -WindowStyle Hidden -PassThru
PfStart -Lang en -Script (DumpScript 400 30 3000)
Pf settings set theme dark | Out-Null
ClearFences | Out-Null
$script:Now = PfPluginFence 'Now playing' '80,60,300,440' 'now-playing'
# The first fence's script may still hold idle.cmd, which the second one writes again.
Start-Sleep -Seconds 1
$script:Edge = PfPluginFence 'Edge' '420,60,300,440' 'now-playing-edge'
$ok = (WaitOut $script:Now '^Nothing is playing' 15) -and (WaitOut $script:Edge '^Nothing is playing' 5)
Check 'both fences show Nothing is playing' $ok "now=$(FenceOut $script:Now) | edge=$(FenceOut $script:Edge)"
Check 'the session is read while the fences show' ([bool](PfWaitLog 'media session: read for plugins from now on, or no more.*on=true' 5)) ''
Check 'Spotify is running: in the fence whose plugin lists spotify.exe' (WaitOut $script:Now '(?m)^Spotify is running$' 8) (FenceOut $script:Now)
Check 'not in the other one' ((FenceOut $script:Edge) -notmatch 'Spotify') (FenceOut $script:Edge)
FenceShot 'nothing-playing' 'Now playing'

Case 'Edge plays: title, artist and album, cover, the time moving on'
New-Item -ItemType Directory -Force $Page | Out-Null
Copy-Item "$Kit\media-page\*" $Page -Force
# Three minutes of silence (16-bit mono, 8 kHz).
$rate = 8000; $n = $rate * 180 * 2; $wav = New-Object byte[] (44 + $n)
$parts = @(@(0, [Text.Encoding]::ASCII.GetBytes('RIFF')), @(4, [BitConverter]::GetBytes([int](36 + $n))), @(8, [Text.Encoding]::ASCII.GetBytes('WAVEfmt ')),
  @(16, [BitConverter]::GetBytes([int]16)), @(20, [BitConverter]::GetBytes([int16]1)), @(22, [BitConverter]::GetBytes([int16]1)),
  @(24, [BitConverter]::GetBytes([int]$rate)), @(28, [BitConverter]::GetBytes([int]($rate * 2))), @(32, [BitConverter]::GetBytes([int16]2)),
  @(34, [BitConverter]::GetBytes([int16]16)), @(36, [Text.Encoding]::ASCII.GetBytes('data')), @(40, [BitConverter]::GetBytes([int]$n)))
foreach ($p in $parts) { [Array]::Copy($p[1], 0, $wav, $p[0], $p[1].Length) }
[IO.File]::WriteAllBytes("$Page\tone.wav", $wav)
$url = 'file:///' + ($Page -replace '\\', '/') + '/play.html'
$edgeArgs = @('--no-first-run', '--no-default-browser-check', "--user-data-dir=$PfRun\edge-media", '--autoplay-policy=no-user-gesture-required', '--window-position=760,60', '--window-size=640,420', $url)
Start-Process $EdgeExe -ArgumentList $edgeArgs | Out-Null
$ok = WaitOut $script:Now 'PecoFence test tone' 30
Check 'the fence shows the title of what Edge plays' $ok "edge=$(EdgeTitle) rows=$((Rows $script:Now) -join ' / ')"
$r = Rows $script:Now
# Edge gives no album to Windows (measured on Server 2025): the plugin shows the artist alone then.
Check 'the artist (and the album when the app gives one), the time and the length' ((($r -contains 'Test artist · Test album') -or ($r -contains 'Test artist')) -and ((FenceOut $script:Now) -match '\d+:\d\d / 3:00')) ($r -join ' / ')
$covers = @(Get-ChildItem "$script:Config\plugins-data\now-playing\media-cover-*" -ErrorAction SilentlyContinue)
Check 'the cover is in the plugin''s data folder, one file, shown in the first row' ($covers.Count -eq 1 -and $r[0] -eq '') "covers=$($covers.Name -join ',') first row='$($r[0])'"
$a = Shown $script:Now; Start-Sleep -Seconds 3; $b = Shown $script:Now
Check 'the time moves on, about a second a second' ($a -ge 0 -and $b - $a -ge 2 -and $b - $a -le 4) "a=$a b=$b"
Check 'app: Msedge in the fence whose plugin lists msedge.exe, none in the other' (((Rows $script:Edge)[0] -eq 'Msedge') -and ((FenceOut $script:Now) -notmatch 'Msedge')) "edge=$((Rows $script:Edge) -join ' / ') now=$((Rows $script:Now) -join ' / ')"
# All of PecoFence's processes (the first one alone may be a helper that does nothing here).
function CpuMs() { return (@(Get-Process pecofence -ErrorAction SilentlyContinue) | ForEach-Object { $_.TotalProcessorTime.TotalMilliseconds } | Measure-Object -Sum).Sum }
$c0 = CpuMs; Start-Sleep -Seconds 10; $c1 = CpuMs
Log "pecofence CPU while two media fences show what plays: $([int]($c1 - $c0)) ms in 10 s"
Check 'PecoFence''s CPU while it plays (two fences, a read and a view a second)' ($c1 -gt $c0 -and $c1 - $c0 -lt 200) "$([int]($c1 - $c0)) ms in 10 s"
FenceShot 'playing' 'Now playing'

Case 'the buttons: pause, play, next, previous, only in a click'
$row = ButtonsRow $script:Now
Check 'the buttons are under the time' ($row -gt 0) ((Rows $script:Now) -join ' / ')
$clicked = ClickCell 'Now playing' $row 1
Check 'play or pause pauses Edge' ($clicked -and (WaitEdge 'paused')) "title=$(EdgeTitle)"
Check 'the fence shows Paused' (WaitOut $script:Now '(?m)^Paused · \d+:\d\d / 3:00$' 5) (FenceOut $script:Now)
$a = Shown $script:Now; Start-Sleep -Seconds 2; $b = Shown $script:Now
Check 'paused, the time stands' ($a -ge 0 -and $a -eq $b) "a=$a b=$b"
Check 'the log names what the plugin asked and that the app took it' ([bool](PfWaitLog 'media session: a plugin played, paused or skipped.*what="playpause".*took=Ok\(true\)' 3)) ''
FenceShot 'paused' 'Now playing'
$clicked = ClickCell 'Now playing' $row 1
Check 'again: Edge plays' ($clicked -and (WaitEdge 'playing')) "title=$(EdgeTitle)"
$clicked = ClickCell 'Now playing' $row 2
Check 'next: the page gets nexttrack' ($clicked -and (WaitEdge 'nexttrack 1')) "title=$(EdgeTitle)"
$clicked = ClickCell 'Now playing' $row 0
Check 'previous: the page gets previoustrack' ($clicked -and (WaitEdge 'previoustrack 2')) "title=$(EdgeTitle)"
Check 'no control was refused' ((LogCount 'plugin call refused.*media\.control') -eq 0) ''

Case 'hidden: nothing is read'
Pf fence hide-all | Out-Null
Check 'reading stops when no media fence shows' ([bool](PfWaitLog 'media session: read for plugins from now on, or no more.*on=false' 5)) ''
$c0 = LogCount 'media session changed: plugins told'
Start-Sleep -Seconds 3
$c1 = LogCount 'media session changed: plugins told'
Check 'no read and no event while hidden' ($c1 -eq $c0) "events $c0 then $c1"
Pf fence show-all | Out-Null
$a = Shown $script:Now; Start-Sleep -Seconds 3; $b = Shown $script:Now
Check 'shown again: read again, the time moves on' ($b -gt $a) "a=$a b=$b"

Case 'Edge and Spotify end: Nothing is playing, no Spotify line'
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Stop-Process -Id $script:Spotify.Id -Force -ErrorAction SilentlyContinue
Check 'Nothing is playing again' (WaitOut $script:Now '^Nothing is playing' 15) (FenceOut $script:Now)
Check 'the Spotify line went' ((WaitOut $script:Now '^Nothing is playing$' 8)) (FenceOut $script:Now)
Check 'no error in the log' ((PfLogErrors).Count -eq 0) ((PfLogErrors) -join ' | ')

Case 'pictures in Chinese: nothing playing, playing, paused, and what the details say it does'
$script:Spotify = Start-Process "$PfRun\spotify.exe" -ArgumentList '-n 900 127.0.0.1' -WindowStyle Hidden -PassThru
if ($PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
PfStart -Lang zh-CN -Script (DumpScript 400 30 3000)
# A plugin reads pf.lang as it starts: once more, with the language kept from the start before.
PfStart -Lang zh-CN -Script (DumpScript 400 30 3000)
Check 'the fence comes back, in Chinese, with the Spotify line' ((WaitOut $script:Now '现在没有在播放的内容' 20) -and (WaitOut $script:Now 'Spotify 在运行' 8)) (FenceOut $script:Now)
FenceShot 'zh-nothing-playing' 'Now playing'
Start-Process $EdgeExe -ArgumentList $edgeArgs | Out-Null
Check 'Edge plays again' (WaitOut $script:Now 'PecoFence test tone' 30) (FenceOut $script:Now)
Start-Sleep -Seconds 2
FenceShot 'zh-playing' 'Now playing'
ClickCell 'Now playing' (ButtonsRow $script:Now) 1 | Out-Null
Check 'paused: 已暂停' (WaitOut $script:Now '已暂停 · ' 5) (FenceOut $script:Now)
FenceShot 'zh-paused' 'Now playing'
if ($PfDebugBuild) {
  $ok = PfSettings; PageClick 'nav a[data-page=tools]'; Start-Sleep -Seconds 2
  # Both test plugins are named 正在播放: open both rows, read the one of now-playing.
  Js "(() => { for (const r of [...document.querySelectorAll('#pluginList .pl-row')].filter(r => r.textContent.startsWith('正在播放'))) if (r.getAttribute('aria-expanded') !== 'true') r.click(); return true; })()" | Out-Null
  Start-Sleep -Seconds 1
  $d = Js "(document.querySelector('#pluginList .pl-detail[data-plugin=now-playing]') || {}).innerText"
  Check 'its details say what it may do: the media session with pausing and skipping, and whether spotify.exe runs' ($ok -and $d -match '看到正在播放的歌曲或视频：名字、歌手、专辑和封面，并能暂停和切歌' -and $d -match '知道这些程序是否在运行：spotify\.exe') "$d"
  $r = PageRect '#pluginList .pl-detail[data-plugin=now-playing]'
  if ($r) { (Crop 'zh-details' ($r.x - 10) ($r.y - 90) ($r.w + 20) ($r.h + 100)).Dispose() }
}
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Stop-Process -Id $script:Spotify.Id -Force -ErrorAction SilentlyContinue
