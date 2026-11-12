# The settings window on the real window (owner round 2026-11-12, third batch). A debug build
# (the page is read over CDP):
#   T  a change of 颜色模式 while the window is open: the title bar does not change before the
#      page (it was a frame ahead), and not more than one recorded frame after it
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
# Records the window while the colour mode changes to $to. Returns the first frame in which the
# title bar shows the new mode and the first in which the page does (one pixel of each: the
# empty title bar right of the title, the page's margin between the navigation and the cards).
function ModeSwitch($to) {
  $r = Rect $win
  $light = $to -eq 'light'
  Start-Process "$PfApp\pecofence-cli.exe" -ArgumentList 'settings', 'set', 'theme', $to -WindowStyle Hidden | Out-Null
  [Px]::Burst($r.L, $r.T, $r.R - $r.L, $r.B - $r.T, 1800, 0); SaveBurst "mode-to-$to"
  $bar = -1; $page = -1
  for ($i = 0; $i -lt [Px]::Frames.Count; $i++) {
    $f = [Px]::Frames[$i]
    $b = ($f.GetPixel(420, 16).R -gt 128) -eq $light
    $p = ($f.GetPixel(264, 320).R -gt 128) -eq $light
    if ($b -and $bar -lt 0) { $bar = $i }
    if ($p -and $page -lt 0) { $page = $i }
  }
  $at = { param($i) if ($i -ge 0) { "frame $i at $([Px]::Times[$i]) ms" } else { 'never' } }
  return @{ bar = $bar; page = $page; text = "title bar: $(& $at $bar); page: $(& $at $page); $([Px]::Frames.Count) frames" }
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
foreach ($round in 1..3) {
  foreach ($to in 'light', 'dark') {
    $m = ModeSwitch $to
    Check "T round $round, to $to`: both changed" ($m.bar -ge 0 -and $m.page -ge 0) $m.text
    Check "T round $round, to $to`: the title bar does not change before the page" ($m.bar -ge $m.page) $m.text
    Check "T round $round, to $to`: the title bar is at most one recorded frame behind the page" ($m.bar - $m.page -le 1) $m.text
    Start-Sleep -Milliseconds 900
  }
}
Pf settings set theme dark | Out-Null
Shot 'end'
