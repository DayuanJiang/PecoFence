# The settings window on the real window (owner round 2026-11-12, third batch). A debug build
# (the page is read over CDP):
#   T  a change of 颜色模式 while the window is open, ten times: the title bar does not change
#      before the page, and is at most one recorded frame behind it. Before the fix the title
#      bar was a frame ahead in some of the switches (5 of 22 recorded on the base build, none
#      of 22 after), so this part can pass on an old build too.
#   N  the window made narrower in steps of 16 px: the navigation's width follows it (no step
#      of 28 or 40 px at 740 and 900 px inside), and its fold to icons at 540 px glides
#   H  with the real mouse: a text box lightens under the pointer, and the selected page's row
#      looks different under the pointer and pressed
# $Rec = 1 keeps the frames in shots\b-<name> for the before/after picture.
if (-not (Get-Variable Rec -ErrorAction SilentlyContinue)) { $Rec = '' }

function SettingsWin() { return [G]::FindWindowW('PecoFence.Settings', [NullString]::Value) }
function SaveBurst($name) {
  if (-not $Rec) { return }
  $d = "$PfShots\b-$name"; New-Item -ItemType Directory -Force $d | Out-Null
  $t = @()
  for ($i = 0; $i -lt [Px]::Frames.Count; $i++) { [Px]::Frames[$i].Save(("$d\{0:000}.png" -f $i)); $t += "$i`t$([Px]::Times[$i])" }
  $t | Set-Content "$d\times.tsv"
}
# Records a strip of the window (200 x 340 px: quick to grab, so the frames are a few ms apart)
# while 颜色模式 changes to $to. The page changes its own list 300 ms after the recording began,
# as a user's choice does (starting pecofence-cli for it kept the guest busy just then, and the
# frames around the change were 60 ms apart). Returns the first frame in which the title bar
# shows the new mode and the first in which the page does (one pixel of each: the empty title
# bar right of the title, the page's margin between the navigation and the cards).
function ModeSwitch($to, $round) {
  $r = Rect $win
  $light = $to -eq 'light'
  Js "(setTimeout(() => { const e = document.querySelector('[data-bind=theme]'); e.value = '$to'; e.dispatchEvent(new Event('change', { bubbles: true })); }, 300), true)" | Out-Null
  [Px]::Burst($r.L + 250, $r.T, 200, 340, 1300, 0); SaveBurst "mode-to-$to-$round"
  $bar = -1; $page = -1
  for ($i = 0; $i -lt [Px]::Frames.Count; $i++) {
    $f = [Px]::Frames[$i]
    $b = ($f.GetPixel(170, 16).R -gt 128) -eq $light
    $p = ($f.GetPixel(14, 320).R -gt 128) -eq $light
    if ($b -and $bar -lt 0) { $bar = $i }
    if ($p -and $page -lt 0) { $page = $i }
  }
  $at = { param($i) if ($i -ge 0) { "frame $i at $([Px]::Times[$i]) ms" } else { 'never' } }
  $lead = if ($bar -ge 0 -and $page -ge 0) { [Px]::Times[$page] - [Px]::Times[$bar] } else { 0 }
  return @{ bar = $bar; page = $page; lead = $lead; text = "title bar: $(& $at $bar); page: $(& $at $page); the title bar is $lead ms ahead; $([Px]::Frames.Count) frames" }
}

Case 'setup'
CloseWindows
Check 'a debug build (the page is reached over CDP)' $PfDebugBuild
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
PfStart -Lang zh-CN
Start-Sleep -Seconds 2
Pf fence create --title '工作文档' --rect '1180,80,360,220' | Out-Null
Pf settings set theme dark | Out-Null
ParkMouse
Check 'settings open over CDP' (PfSettings)
$win = SettingsWin
Start-Sleep -Seconds 1

Case 'T: the title bar changes colour with the page'
foreach ($round in 1..5) {
  foreach ($to in 'light', 'dark') {
    $m = ModeSwitch $to $round
    Check "T round $round, to $to`: both changed" ($m.bar -ge 0 -and $m.page -ge 0) $m.text
    Check "T round $round, to $to`: the title bar does not change before the page" ($m.bar -ge $m.page) $m.text
    Check "T round $round, to $to`: the title bar is at most one recorded frame behind the page" ($m.bar - $m.page -le 1) $m.text
    Start-Sleep -Milliseconds 900
  }
}
Pf settings set theme dark | Out-Null
Start-Sleep -Milliseconds 800

Case 'N: the navigation follows the window'
$r = Rect $win
$top = $r.T; $left = $r.L; $height = $r.B - $r.T
$nav = { [double](Js "document.querySelector('nav').getBoundingClientRect().width") }
$last = $null; $worst = 0; $worstAt = 0; $seen = @()
for ($w = 1000; $w -ge 620; $w -= 16) {
  [L]::MoveWindow($win, $left, $top, $w, $height, $true) | Out-Null
  Start-Sleep -Milliseconds 120
  $now = & $nav
  if ($null -ne $last -and [Math]::Abs($now - $last) -gt $worst) { $worst = [Math]::Abs($now - $last); $worstAt = $w }
  $last = $now; $seen += "$w`:$([Math]::Round($now))"
}
Check 'N between two window widths 16 px apart the navigation changes by at most 8 px' ($worst -le 8) "largest change $worst px at a window width of $worstAt px; widths $($seen -join ' ')"
$inner = { [int](Js 'innerWidth') }
[L]::MoveWindow($win, $left, $top, 620, $height, $true) | Out-Null
Start-Sleep -Milliseconds 300
$pad = 620 - (& $inner)
[L]::MoveWindow($win, $left, $top, 548 + $pad, $height, $true) | Out-Null
Start-Sleep -Milliseconds 500
$open = & $nav
Js "(window.__nav = [], (function tick() { window.__nav.push(Math.round(document.querySelector('nav').getBoundingClientRect().width)); if (window.__nav.length < 40) requestAnimationFrame(tick); })(), true)" | Out-Null
[L]::MoveWindow($win, $left, $top, 532 + $pad, $height, $true) | Out-Null
Start-Sleep -Milliseconds 900
$widths = @(Js 'window.__nav')
$between = @($widths | Where-Object { $_ -gt 60 -and $_ -lt 168 }).Count
# The window's smallest size keeps the page wider than 540 px at 100 % on a large screen: the
# fold is only reached on small screens at a high scale. It is checked where it can be reached.
if ((& $inner) -le 540) { Check 'N folding to icons: the navigation is on its way for at least 3 frames' ($between -ge 3) "open $open px; widths per frame: $($widths -join ' ')" }
else { Check 'N the fold to icons cannot be reached here (the smallest window keeps the page wider than 540 px)' $true "page width $(& $inner) px at the window's smallest size" }
[L]::MoveWindow($win, $r.L, $r.T, $r.R - $r.L, $r.B - $r.T, $true) | Out-Null
Start-Sleep -Milliseconds 700

Case 'H: looks under the pointer'
function Bg($sel) { $q = $sel | ConvertTo-Json; return Js "getComputedStyle(document.querySelector($q)).backgroundColor" }
ParkMouse
PageClick 'nav a[data-page=fences]'
ParkMouse
Start-Sleep -Milliseconds 600
$rest = Bg '#fenceTitle'
$p = PageRect '#fenceTitle'
[G]::MoveTo($p.cx, $p.cy); Start-Sleep -Milliseconds 500
$over = Bg '#fenceTitle'
Check 'H a text box looks different under the pointer' ($rest -ne $over) "rest [$rest] under the pointer [$over]"
ParkMouse; Start-Sleep -Milliseconds 400
$rest = Bg 'nav a.sel'
$p = PageRect 'nav a.sel'
[G]::MoveTo($p.cx, $p.cy); Start-Sleep -Milliseconds 500
$over = Bg 'nav a.sel'
MDown; Start-Sleep -Milliseconds 350
$down = Bg 'nav a.sel'
MUp; Start-Sleep -Milliseconds 300
Check 'H the selected page looks different under the pointer' ($rest -ne $over) "rest [$rest] under the pointer [$over]"
Check 'H the selected page looks different pressed' ($over -ne $down) "under the pointer [$over] pressed [$down]"
ParkMouse
Shot 'end'
