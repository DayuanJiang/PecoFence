# Motion and feel of the settings window (owner round 2026-11-12, second batch), with the real
# mouse on the real window. A debug build (the page is read over CDP):
#   P  every kind of button looks different pressed than under the pointer, and a colour swatch
#      keeps its colour while it is pressed
#   M  a change of 颜色模式 while the window is open lands in one frame: no frame in which the
#      controls are still fading behind the page
#   L  an opening drop-down list comes in over a few frames
#   G  a rule moved down: its row and the row under it glide to their places
#   R  the card that 「Windows 桌面图标」 = 隐藏 brings up grows open, and closes again
#   S  the switch of a rule slides like every other switch
# The part is $Part (default: all). $Rec = 1 keeps the frames of M, L, G and R (and of a page
# switch and a change of fence) in shots\b-<name> for the before/after pictures.
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'PMLGRS' }
if (-not (Get-Variable Rec -ErrorAction SilentlyContinue)) { $Rec = '' }

Add-Type @'
public static class UxPx {
  // How many pixels of two same-size grabs (Px.Pixels) differ by more than thr in a channel.
  public static int Differ(int[] a, int[] b, int thr) {
    int n = 0;
    for (int i = 0; i < a.Length; i++) {
      int p = a[i], q = b[i];
      if (System.Math.Abs(((p >> 16) & 255) - ((q >> 16) & 255)) > thr || System.Math.Abs(((p >> 8) & 255) - ((q >> 8) & 255)) > thr || System.Math.Abs((p & 255) - (q & 255)) > thr) n++;
    }
    return n;
  }
}
'@

function SettingsWin() { return [G]::FindWindowW('PecoFence.Settings', [NullString]::Value) }
function SaveBurst($name) {
  if (-not $Rec) { return }
  $d = "$PfShots\b-$name"; New-Item -ItemType Directory -Force $d | Out-Null
  $t = @()
  for ($i = 0; $i -lt [Px]::Frames.Count; $i++) { [Px]::Frames[$i].Save(("$d\{0:000}.png" -f $i)); $t += "$i`t$([Px]::Times[$i])" }
  $t | Set-Content "$d\times.tsv"
}
# The frames of the last burst that are neither the picture before the gesture nor the picture
# at rest after it (more than $min pixels away from both): the frames in which something is on
# its way. Returns their count and their times.
function Between($min = 150) {
  $n = [Px]::Frames.Count
  $first = [Px]::Pixels([Px]::Frames[0]); $last = [Px]::Pixels([Px]::Frames[$n - 1])
  $times = @()
  for ($i = 1; $i -lt $n - 1; $i++) {
    $px = [Px]::Pixels([Px]::Frames[$i])
    $a = [UxPx]::Differ($px, $first, 10); $b = [UxPx]::Differ($px, $last, 10)
    if ($a -gt $min -and $b -gt $min) { $times += "$([Px]::Times[$i]) ms ($a from before, $b from after)" }
  }
  return @{ count = $times.Count; text = ($times -join '; '); total = [UxPx]::Differ($first, $last, 10) }
}
function PageSet($q, $value) {
  $s = $q | ConvertTo-Json; $v = $value | ConvertTo-Json
  Js "(() => { const e = document.querySelector($s); e.value = $v; e.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
}
# Somewhere on the page where a mouse button can go up without a click on anything.
function RestPoint() { $r = PageRect '.brand'; return @($r.cx, $r.cy) }
function Look($sel) { $q = $sel | ConvertTo-Json; return Js "(() => { const c = getComputedStyle(document.querySelector($q)); return c.backgroundColor + ' | ' + c.color + ' | ' + c.filter; })()" }

Case 'setup'
CloseWindows
Check 'a debug build (the page is reached over CDP)' $PfDebugBuild
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
PfStart -Lang zh-CN
Start-Sleep -Seconds 2
Pf fence create --title '工作文档' --rect '1180,80,360,220' | Out-Null
Pf fence create --title '图片和截图' --rect '1180,330,360,220' | Out-Null
Pf rule add --name '文档' --ext 'docx,xlsx,pdf' --to '工作文档' | Out-Null
Pf rule add --name '图片' --ext 'jpg,png' --to '图片和截图' | Out-Null
Pf rule add --name '安装包' --ext 'exe,msi' --to '工作文档' | Out-Null
Pf settings set theme dark | Out-Null
ParkMouse
Check 'settings open over CDP' (PfSettings)
$win = SettingsWin
Start-Sleep -Seconds 1

# ---- P: pressed looks
function PartP() {
  trap { PfTrap $_; continue }
  Case 'P: a pressed button looks pressed'
  $rules = @(Rules).Count
  PageClick 'nav a[data-page=rules]'
  Js "(document.getElementById('nrPanel').open = true)" | Out-Null
  Start-Sleep -Milliseconds 600
  $rest = RestPoint
  foreach ($c in @(@('the accent button 添加规则', '#nrAdd', 'rules'), @('the header of 新建规则', '#nrPanel > summary', 'rules'), @('the delete button of a rule', '#ruleList .rule [data-act=del]', 'rules'),
      @('a 主题风格 tile', 'label.tile-glass', 'general'), @('a colour swatch', '#page-general .swatch:not(.none):not(.custom)', 'general'))) {
    if ((Js "document.querySelector('nav a.sel').dataset.page") -ne $c[2]) { PageClick "nav a[data-page=$($c[2])]"; Start-Sleep -Milliseconds 400 }
    $r = PageRect $c[1]
    [G]::MoveTo($r.cx, $r.cy); Start-Sleep -Milliseconds 500
    $hover = Look $c[1]
    MDown; Start-Sleep -Milliseconds 350
    $down = Look $c[1]
    [G]::MoveTo($rest[0], $rest[1]); Start-Sleep -Milliseconds 150
    MUp; Start-Sleep -Milliseconds 300
    Check "P $($c[0]) looks different pressed than under the pointer" ($hover -ne $down) "hover [$hover] pressed [$down]"
    if ($c[0] -match 'swatch') { Check 'P a pressed swatch keeps its colour' (($hover -split ' \| ')[0] -eq ($down -split ' \| ')[0]) "hover [$hover] pressed [$down]" }
  }
  Check 'P nothing was clicked on the way (as many rules as before, Fluent)' (@(Rules).Count -eq $rules -and (PfJson settings get).themeStyle -ne 'liquidGlass') "rules $(@(Rules).Count) (were $rules), style $((PfJson settings get).themeStyle)"
}
if ($Part -match 'P') { PartP }

# ---- M: a change of colour mode lands in one frame
function ModeFrames($to, $tag) {
  $r = Rect $win
  # The client area without the caption (Windows colours it a frame before the page) and the edges.
  $x = $r.L + 12; $y = $r.T + 44; $w = $r.R - $r.L - 44; $h = $r.B - $r.T - 58
  Start-Process "$PfApp\pecofence-cli.exe" -ArgumentList 'settings', 'set', 'theme', $to -WindowStyle Hidden | Out-Null
  [Px]::Burst($x, $y, $w, $h, 1800, 0); SaveBurst "mode-$tag"
  return (Between 400)
}
function PartM() {
  trap { PfTrap $_; continue }
  Case 'M: a change of colour mode lands in one frame'
  PageClick 'nav a[data-page=fences]'
  ParkMouse
  Start-Sleep -Milliseconds 800
  $a = ModeFrames 'light' 'to-light'
  Check 'M dark to light: the page changed' ($a.total -gt 20000) "$($a.total) pixels differ between the first and the last frame"
  Check 'M dark to light: at most one frame is neither the dark nor the light page' ($a.count -le 1) "$($a.count) frames between: $($a.text)"
  Start-Sleep -Seconds 1
  $b = ModeFrames 'dark' 'to-dark'
  Check 'M light to dark: at most one frame is neither the light nor the dark page' ($b.count -le 1) "$($b.count) frames between: $($b.text)"
  Start-Sleep -Seconds 1
}
if ($Part -match 'M') { PartM }

# ---- L: an opening list comes in over a few frames
function PartL() {
  trap { PfTrap $_; continue }
  Case 'L: an opening drop-down list comes in'
  PageClick 'nav a[data-page=general]'
  Start-Sleep -Milliseconds 500
  $r = PageRect '[data-bind=titleAlign]'
  [G]::MoveTo($r.cx, $r.cy); Start-Sleep -Milliseconds 600
  [Px]::Burst($r.x - 20, $r.y + $r.h + 2, $r.w + 40, 130, 600, 1); SaveBurst 'list'
  $b = Between 600
  Check 'L the list is open' (Js "document.querySelector('[data-bind=titleAlign]').matches(':open')") ''
  Check 'L the list is on its way in at least one frame' ($b.count -ge 1) "$($b.count) frames between: $($b.text); $($b.total) pixels differ between the first and the last frame"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  Check 'L Esc closed it' (-not (Js "document.querySelector('[data-bind=titleAlign]').matches(':open')")) ''
}
if ($Part -match 'L') { PartL }

# ---- G: a rule moved down glides
function PartG() {
  trap { PfTrap $_; continue }
  Case 'G: a rule moved down glides to its place'
  PageClick 'nav a[data-page=rules]'
  Start-Sleep -Milliseconds 500
  $first = Js "document.querySelector('#ruleList .rule .t').textContent"
  $l = PageRect '#ruleList'
  $r = PageRect '#ruleList .rule [data-act=down]'
  [G]::MoveTo($r.cx, $r.cy); Start-Sleep -Milliseconds 600
  # The names and conditions of the rows (left of the buttons, which light up under the pointer).
  [Px]::Burst($l.x, $l.y, [Math]::Min(420, $l.w), [Math]::Min(190, $l.h), 600, 1); SaveBurst 'rule-down'
  $b = Between 600
  $second = Js "document.querySelectorAll('#ruleList .rule .t')[1].textContent"
  Check 'G the first rule is now the second' ($second -eq $first) "first was '$first', second is '$second'"
  Check 'G the rows are on their way in at least two frames' ($b.count -ge 2) "$($b.count) frames between: $($b.text)"
  Check 'G the host has the new order' ((Rules)[1].name -eq $first) "$((Rules | ForEach-Object { $_.name }) -join ', ')"
}
if ($Part -match 'G') { PartG }

# ---- R: a card shown by a setting grows open and closes
function PartR() {
  trap { PfTrap $_; continue }
  Case 'R: the card of 「隐藏」 grows open and closes'
  PageClick 'nav a[data-page=general]'
  Start-Sleep -Milliseconds 500
  Js "(document.getElementById('desktopIcons').scrollIntoView({ block: 'center', behavior: 'instant' }), true)" | Out-Null
  Start-Sleep -Milliseconds 300
  $g = PageRect '#desktopIcons'
  $o = ClientOrigin $win
  ParkMouse
  $x = $o.X + 260; $y = $g.y + $g.h + 14; $w = 560; $h = 200
  # The card shows while the icons are hidden: start from the other choice.
  $was = Js "document.getElementById('desktopIcons').value"
  if ($was -eq 'hide') { PageSet '#desktopIcons' 'showNoInbox'; Start-Sleep -Milliseconds 1500 }
  PageSet '#desktopIcons' 'hide'
  [Px]::Burst($x, $y, $w, $h, 700, 0); SaveBurst 'reveal-open'
  $a = Between 600
  Check 'R the card is shown' (-not (Js "document.getElementById('keepIconsCard').hidden")) ''
  Check 'R the card is on its way open in at least two frames' ($a.count -ge 2) "$($a.count) frames between: $($a.text); $($a.total) pixels differ between the first and the last frame"
  Start-Sleep -Milliseconds 600
  PageSet '#desktopIcons' 'showNoInbox'
  [Px]::Burst($x, $y, $w, $h, 600, 0); SaveBurst 'reveal-close'
  $b = Between 600
  Check 'R the card is hidden' (Js "document.getElementById('keepIconsCard').hidden") ''
  Check 'R the card is on its way closed in at least one frame' ($b.count -ge 1) "$($b.count) frames between: $($b.text)"
  Start-Sleep -Milliseconds 500
  if ($was -eq 'hide') { PageSet '#desktopIcons' 'hide'; Start-Sleep -Milliseconds 800 }
}
if ($Part -match 'R') { PartR }

# ---- S: the switch of a rule slides
function PartS() {
  trap { PfTrap $_; continue }
  Case 'S: the switch of a rule slides'
  PageClick 'nav a[data-page=rules]'
  Start-Sleep -Milliseconds 500
  $was = (Rules)[0].enabled
  $r = PageRect '#ruleList .rule .toggle'
  [G]::MoveTo($r.cx, $r.cy); Start-Sleep -Milliseconds 600
  [Px]::Burst($r.x - 6, $r.y - 6, $r.w + 12, $r.h + 12, 500, 1); SaveBurst 'rule-switch'
  $b = Between 40
  Check 'S the host has the rule switched' ((Rules)[0].enabled -ne $was) "enabled was $was, is $((Rules)[0].enabled)"
  Check 'S the knob is on its way in at least two frames' ($b.count -ge 2) "$($b.count) frames between: $($b.text)"
  [G]::Click($r.cx, $r.cy); Start-Sleep -Milliseconds 600
  Check 'S a second click switches it back' ((Rules)[0].enabled -eq $was) "enabled is $((Rules)[0].enabled)"
}
if ($Part -match 'S') { PartS }

# ---- recordings for the before/after pictures: a page switch, another fence
if ($Rec) {
  Case 'recordings'
  PageClick 'nav a[data-page=general]'
  Start-Sleep -Milliseconds 600
  $r = Rect $win
  $p = PageRect 'nav a[data-page=layout]'
  [G]::MoveTo($p.cx, $p.cy); Start-Sleep -Milliseconds 600
  [Px]::Burst($r.L, $r.T, $r.R - $r.L, $r.B - $r.T, 700, 1); SaveBurst 'page'
  PageClick 'nav a[data-page=fences]'
  Start-Sleep -Milliseconds 700
  ParkMouse
  $ids = Js "[...document.getElementById('fenceSel').options].map(o => o.value)"
  PageSet '#fenceSel' $ids[-1]
  [Px]::Burst($r.L, $r.T, $r.R - $r.L, $r.B - $r.T, 600, 0); SaveBurst 'fence'
}
Shot 'end'
