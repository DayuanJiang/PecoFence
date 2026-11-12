# PR #216: 闲置时的不透明度 (Settings > General > Appearance, `idleOpacity`, 1 = off). A fence
# nobody uses fades its plate, icons, names and shadow to that opacity and comes back when the
# pointer is over it, the keyboard is in it, something is dragged, a name box, a note or a filter
# box of it is open, its menu is open, or a Peek shows the fences.
# Real mouse and keyboard on a release build. The opacity is read twice: from the app's dump
# line (`idle_opacity`, what it gave the compositor) and from the screen (the brightness of a
# patch of the plate over a grey wallpaper), the latter also while a popup menu holds the app.
# Parts (run.sh --var Part=...): A the states one by one, the values the command line and the
# config file take, a restart; B the idle fade together with the other animations (roll, slide,
# drag, resize, drops, tools, Peek, virtual desktops, Win+D, animations off, neighbours); C the
# card on the settings page (a debug build: run.sh --bin target/wintest-bin-debug).
# Default: A and B. Runs on a 1600x900 box at 100 %.
if (-not $Part) { $Part = 'AB' }
if ($PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
trap { PfTrap $_; continue }
. C:\pf\kit\vdesk.ps1

$IfFiles = @('if a.txt', 'if b.txt', 'if c.txt', 'if d.txt', 'if e.txt', 'if f.txt', 'if g.txt', 'if h.txt')
$IfDir = "$PfRun\if-src"
function IfClean() {
  Get-ChildItem -LiteralPath $PfDesk -Force -ErrorAction SilentlyContinue | Where-Object { $_.Name -like 'if *' } | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
}
function Hw($t) { return [G]::FindWindowW('PecoFence.Fence', $t) }
function Win($t) { return (Rect (Hw $t)) }
function Mid($t) { $r = Win $t; return @([int](($r.L + $r.R) / 2), [int](($r.T + $r.B) / 2)) }
function TitlePt($t) { $r = Win $t; return @(($r.L + 100), ($r.T + 18)) }
function Place($t, $x, $y, $w, $h) { Pf fence move $t --x $x --y $y | Out-Null; Pf fence resize $t --w $w --h $h | Out-Null }
function Park() { ParkMouse 1300 720; Start-Sleep -Milliseconds 350 }
# The pointer comes to rest in the middle of the fence (two moves, as a hand does).
function Over($t) { $p = Mid $t; [G]::MoveTo($p[0] - 4, $p[1]); Start-Sleep -Milliseconds 60; [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 400 }
# What the app gave the compositor for the fence: the idle layer's opacity; -1 without a dump.
function Io($t) { $w = DumpOf $t 4; if (-not $w) { return -1 }; return [double](DumpVal $w 'idle_opacity') }
function IoWait($t, $want, $sec = 3) {
  $v = -1
  for ($i = 0; $i -lt $sec * 3; $i++) { $v = Io $t; if ([math]::Abs($v - $want) -lt 0.02) { break }; Start-Sleep -Milliseconds 120 }
  return $v
}
function IoIs($what, $t, $want) { $v = IoWait $t $want; Check $what ([math]::Abs($v - $want) -lt 0.02) "$t idle_opacity=$v, expected $want" }
# The same from the screen: mean brightness of a 40 x 24 patch at the bottom of the plate, where
# no icon is ($left: at the left end, for when a menu covers the right one). The plate differs
# from the grey wallpaper, so a faded fence is nearer to the wallpaper's grey.
function Lum($t, [bool]$left = $false) {
  $r = Win $t
  $x = if ($left) { $r.L + 20 } else { $r.R - 60 }
  $b = [Px]::Grab($x, $r.B - 40, 40, 24); $m = MeanRgb $b; $b.Dispose()
  return [math]::Round(($m[0] + $m[1] + $m[2]) / 3.0, 1)
}
# How far the patch is from the full plate towards the faded one: 0 = full, 1 = faded.
function Faded($t, [bool]$left = $false) { if ($script:LumIdle -eq $script:LumFull) { return -1 }; return [math]::Round(((Lum $t $left) - $script:LumFull) / ($script:LumIdle - $script:LumFull), 2) }
function ScreenIs($what, $t, [bool]$faded, [bool]$left = $false) {
  Start-Sleep -Milliseconds 350
  $f = Faded $t $left
  $ok = if ($faded) { $f -gt 0.8 } else { $f -lt 0.2 }
  Check $what $ok "$t on screen: $f of the way to the faded plate (0 = full, 1 = faded)"
}
# The setting as a number (an f32 in the app: 0.45 comes back as 0.4499999880790710).
function IdleGet() { $t = (Pf settings get idleOpacity).Trim(); $v = 0.0; if ([double]::TryParse($t, [Globalization.NumberStyles]::Float, [Globalization.CultureInfo]::InvariantCulture, [ref]$v)) { return [math]::Round($v, 3) }; return $t }
function SetIdle($v) { Pf settings set idleOpacity $v | Out-Null; Start-Sleep -Milliseconds 500 }
function ShadowOk($t, $want) {
  $w = DumpOf $t 4
  $a = [int](DumpVal $w 'shadow_alpha'); $plate = [double](DumpVal $w 'plate_alpha')
  return @(([math]::Abs($a - [math]::Round(255 * $want * $plate)) -le 1), "shadow_alpha=$a plate_alpha=$plate, expected $([math]::Round(255 * $want * $plate))")
}
function ExOpen($path, $x, $y, $w, $h) {
  Start-Process explorer.exe "`"$path`""
  for ($i = 0; $i -lt 60; $i++) {
    foreach ($win in @((New-Object -ComObject Shell.Application).Windows())) {
      $p = try { $win.Document.Folder.Self.Path } catch { '' }
      if ($p -eq $path) { $hw = [IntPtr]$win.HWND; [L]::MoveWindow($hw, $x, $y, $w, $h, $true) | Out-Null; Start-Sleep -Milliseconds 1200; return $hw }
    }
    Start-Sleep -Milliseconds 250
  }
  return [IntPtr]::Zero
}
function ExItemPt($hw, $leaf) {
  $root = [Windows.Automation.AutomationElement]::FromHandle($hw)
  $isItem = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::ListItem)
  for ($k = 0; $k -lt 20; $k++) {
    foreach ($n in @($leaf, [IO.Path]::GetFileNameWithoutExtension($leaf))) {
      $byName = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::NameProperty, $n)
      $el = $root.FindFirst([Windows.Automation.TreeScope]::Descendants, (New-Object Windows.Automation.AndCondition($isItem, $byName)))
      # The icon at the left of the row: a press beside a short name starts a selection box.
      if ($el) { $r = $el.Current.BoundingRectangle; return @([int]($r.Left + 16), [int](($r.Top + $r.Bottom) / 2)) }
    }
    Start-Sleep -Milliseconds 250
  }
  return $null
}

Case 'setup'
CloseWindows
VdOnlyOne
IfClean
foreach ($n in $IfFiles) { DeskFile $n | Out-Null }
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
# An RDP session shows only an outline while a window is dragged: show the window itself.
[G]::SystemParametersInfoW(0x25, 1, [NullString]::Value, 3) | Out-Null
GreyWallpaper
PfStart -Script (DumpScript 150 14)
ClearFences | Out-Null
Pf fence create --title Left --rect '300,200,300,260' | Out-Null
Pf fence create --title Right --rect '608,200,300,260' | Out-Null
Pf item move "$PfDesk\if a.txt" "$PfDesk\if b.txt" "$PfDesk\if c.txt" --to Left | Out-Null
Pf item move "$PfDesk\if d.txt" "$PfDesk\if e.txt" "$PfDesk\if f.txt" --to Right | Out-Null
Start-Sleep -Seconds 1; Park
$inbox = @(PfJson fence list) | Where-Object { $_ -and $_.kind -eq 'inbox' } | Select-Object -First 1
Log "inbox '$($inbox.title)' desktopLayout=$((FenceGet $inbox.id).desktopLayout)"

Case 'off is the default'
$get = (Pf settings get idleOpacity).Trim()
Check 'idleOpacity is 1 on a fresh start' ($get -match '^1(\.0)?$') "settings get idleOpacity: $get"
IoIs 'off: a fence at rest is not faded' 'Left' 1.0
IoIs 'off: the other fence too' 'Right' 1.0
$script:LumFull = Lum 'Left'
$r = ShadowOk 'Left' 1.0; Check 'off: the shadow has its full strength' $r[0] $r[1]
Shot 'off'

Case 'on: fences at rest fade'
SetIdle 0.45; Park
IoIs 'idleOpacity 0.45: a fence at rest goes to 0.45' 'Left' 0.45
IoIs 'idleOpacity 0.45: the other fence too' 'Right' 0.45
$script:LumIdle = Lum 'Left'
Check 'the fade reaches the screen' ([math]::Abs($LumIdle - $LumFull) -ge 8) "plate patch brightness full=$LumFull faded=$LumIdle (grey wallpaper 128)"
$r = ShadowOk 'Left' 0.45; Check 'the shadow fades with the plate' $r[0] $r[1]
$w = DumpOf $inbox.title 4
Check 'the fence arranged like the desktop has no plate and does not fade' ((DumpVal $w 'idle_opacity') -eq '1.00') "inbox idle_opacity=$(DumpVal $w 'idle_opacity')"
Shot 'on-idle'

if ($Part -match 'A') {
  Case 'A pointer'
  Over 'Left'
  IoIs 'the pointer over a fence: full' 'Left' 1.0
  ScreenIs 'the pointer over a fence: full on screen' 'Left' $false
  IoIs 'its neighbour stays faded' 'Right' 0.45
  $r = ShadowOk 'Left' 1.0; Check 'the shadow comes back with the plate' $r[0] $r[1]
  Shot 'A-hover-left'
  # Straight across the 8 px gap into the neighbour.
  $a = Mid 'Left'; $b = Mid 'Right'
  Glide $a[0] $a[1] $b[0] $b[1] 10 20; Start-Sleep -Milliseconds 400
  IoIs 'the pointer moved on to the neighbour: the first fence fades' 'Left' 0.45
  IoIs 'the neighbour is full' 'Right' 1.0
  Park
  IoIs 'the pointer left: faded again' 'Right' 0.45
  ScreenIs 'the pointer left: faded on screen' 'Right' $true
  # In and out within 40 ms: the fade turns round and ends faded.
  $p = Mid 'Left'; [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 40; Park
  IoIs 'in and out within 40 ms ends faded' 'Left' 0.45

  Case 'A keyboard'
  $p = ItemPt 'Left' 'if a'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 300; Park
  Log "after a click on an icon and the pointer away: idle_opacity=$(Io 'Left')"
  [G]::Press(0x27); Start-Sleep -Milliseconds 400
  IoIs 'an arrow key in the fence that has the keyboard: full' 'Left' 1.0
  [G]::Press(0x09); Start-Sleep -Milliseconds 500
  IoIs 'Tab moved the keyboard to the next fence: it is full' 'Right' 1.0
  IoIs 'the fence the keyboard left fades' 'Left' 0.45
  DeskClick | Out-Null; Park
  IoIs 'a click on the desktop: the fence that had the keyboard fades' 'Right' 0.45

  Case 'A name boxes'
  $p = ItemPt 'Left' 'if a'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 300
  [G]::Press(0x71); $ed = WaitWin 'PecoFence.Rename' 6; Park
  Check 'F2 opens the name box of the icon' ($ed -ne [IntPtr]::Zero) "name box $ed"
  IoIs 'the name box of an icon is open, the pointer away: full' 'Left' 1.0
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  DeskClick | Out-Null; Park
  IoIs 'the name box closed and the desktop clicked: faded' 'Left' 0.45
  $p = TitlePt 'Left'; [G]::RClick($p[0], $p[1]); Start-Sleep -Milliseconds 600
  MenuPick '^Rename' | Out-Null
  $ed = WaitWin 'PecoFence.Rename' 6; Park
  Check 'Rename in the fence menu opens the name box of the fence' ($ed -ne [IntPtr]::Zero) "name box $ed"
  IoIs 'the name box of the fence is open, the pointer away: full' 'Left' 1.0
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  DeskClick | Out-Null; Park
  IoIs 'that name box closed and the desktop clicked: faded' 'Left' 0.45

  Case 'A popup menu'
  # While a menu is open the app answers no dump: the screen is read instead.
  $p = TitlePt 'Left'; [G]::RClick($p[0], $p[1]); Start-Sleep -Milliseconds 700
  $m = MenuWin 5
  $mr = Rect $m
  # The pointer goes onto the menu (off the fence) and rests there.
  [G]::MoveTo($mr.L + 60, $mr.B - 30); Start-Sleep -Milliseconds 600
  Check 'the fence menu is open' ($m -ne [IntPtr]::Zero) "menu $m at $($mr.L),$($mr.T)"
  # The menu covers the right half of the fence: its left end is read.
  ScreenIs 'the fence whose menu is open stays full with the pointer on the menu' 'Left' $false $true
  ScreenIs 'a fence that has nothing to do with the menu stays faded' 'Right' $true
  Shot 'A-menu-open'
  CloseMenus; Park
  IoIs 'the menu closed, the pointer away: faded' 'Left' 0.45
  IoIs 'the other fence is faded after the menu' 'Right' 0.45
  $p = ItemPt 'Right' 'if f'; [G]::RClick($p[0], $p[1]); Start-Sleep -Milliseconds 900
  $m = MenuWin 5; $mr = Rect $m
  [G]::MoveTo($mr.L + 60, $mr.B - 30); Start-Sleep -Milliseconds 600
  Log "icon menu at $($mr.L),$($mr.T),$($mr.R),$($mr.B)"
  ScreenIs 'the fence whose icon menu is open stays full' 'Right' $false $true
  ScreenIs 'the other fence stays faded during the icon menu' 'Left' $true
  CloseMenus; DeskClick | Out-Null; Park
  IoIs 'the icon menu closed: faded' 'Right' 0.45
  # The tray menu belongs to no fence: none of them changes.
  $ctl = [G]::FindWindowW('PecoFence.Control', [NullString]::Value)
  [L]::PostMessageW($ctl, 0x8003, [IntPtr](1500 -bor (830 -shl 16)), [IntPtr](0x7B -bor (1 -shl 16))) | Out-Null
  $m = MenuWin 5; Start-Sleep -Milliseconds 600
  Check 'the tray menu is open' ($m -ne [IntPtr]::Zero) "menu $m"
  ScreenIs 'the tray menu is open: a fence stays faded' 'Left' $true
  ScreenIs 'the tray menu is open: the other fence too' 'Right' $true
  CloseMenus; Park
  IoIs 'the tray menu closed: faded' 'Left' 0.45

  Case 'A rolled up'
  Pf fence roll Left | Out-Null; Start-Sleep -Milliseconds 900
  IoIs 'a rolled-up fence at rest is faded' 'Left' 0.45
  $p = TitlePt 'Left'; [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 200; [G]::MoveTo($p[0] + 2, $p[1]); Start-Sleep -Milliseconds 300
  IoIs 'the pointer on its title row: full' 'Left' 1.0
  Start-Sleep -Milliseconds 1100
  IoIs 'opened by the resting pointer: full' 'Left' 1.0
  Park; Start-Sleep -Milliseconds 1200
  IoIs 'the pointer left, it rolled up again: faded' 'Left' 0.45
  Pf fence unroll Left | Out-Null; Start-Sleep -Milliseconds 900
  IoIs 'opened from the command line with the pointer away: faded' 'Left' 0.45
  # Rolled up while it has the keyboard: nothing of it can be used any more.
  $p = ItemPt 'Left' 'if a'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 300; Park
  [G]::Press(0x27); Start-Sleep -Milliseconds 300
  IoIs 'the keyboard is in the fence: full' 'Left' 1.0
  Pf fence roll Left | Out-Null; Start-Sleep -Milliseconds 1000
  IoIs 'rolled up while it had the keyboard, the pointer away: faded' 'Left' 0.45
  Pf fence unroll Left | Out-Null; Start-Sleep -Milliseconds 900
  DeskClick | Out-Null; Park

  Case 'A Peek'
  PeekKey; Start-Sleep -Milliseconds 900
  $peek = @([W]::Tops('PecoFence.PeekDimmer')).Count -gt 0
  Check 'Ctrl+Alt+Space opens the Peek' $peek "dimmer shown: $peek"
  IoIs 'during the Peek every fence is full (1)' 'Left' 1.0
  IoIs 'during the Peek every fence is full (2)' 'Right' 1.0
  [G]::Press(0x1B); Start-Sleep -Milliseconds 900; Park
  IoIs 'the Peek ended: faded (1)' 'Left' 0.45
  IoIs 'the Peek ended: faded (2)' 'Right' 0.45

  Case 'A quick hide'
  $s = FreeSpot 1100 600
  DblClick $s[0] $s[1]; Start-Sleep -Milliseconds 1200
  $w = DumpOf 'Left' 4
  Check 'a double-click on the desktop hides the fences' ($w.raw -notmatch 'visible=true' -or -not [G]::IsWindowVisible((Hw 'Left'))) "Left visible: $([G]::IsWindowVisible((Hw 'Left')))"
  DblClick $s[0] $s[1]; Start-Sleep -Milliseconds 1200
  Check 'the next double-click shows them' ([G]::IsWindowVisible((Hw 'Left'))) "Left visible: $([G]::IsWindowVisible((Hw 'Left')))"
  IoIs 'shown again with the pointer on the desktop: faded' 'Left' 0.45
  ScreenIs 'shown again: faded on screen' 'Left' $true
  $r = ShadowOk 'Left' 0.45; Check 'shown again: the shadow is as faint as the plate' $r[0] $r[1]
  # Hidden while the pointer is on it, shown again with the pointer elsewhere.
  Over 'Left'
  IoIs 'the pointer on the fence: full' 'Left' 1.0
  Pf fence hide-all | Out-Null; Start-Sleep -Milliseconds 900
  Park
  Pf fence show-all | Out-Null; Start-Sleep -Milliseconds 900
  IoIs 'hidden under the pointer, shown again with the pointer elsewhere: faded' 'Left' 0.45

  Case 'A tabs'
  Pf fence merge Right --into Left | Out-Null; Start-Sleep -Milliseconds 900; Park
  IoIs 'a fence with two tabs at rest: faded' 'Left' 0.45
  Over 'Left'
  IoIs 'the pointer over it: full' 'Left' 1.0
  # A click on the other tab (the second tab of the strip).
  $w = DumpOf 'Left' 4
  $r = Win 'Left'
  if ($w.rects.Count -ge 2) { [G]::Click([int]($r.L + $w.rects[1][0] + $w.rects[1][1] / 2), $r.T + 18); Start-Sleep -Milliseconds 700 }
  Park
  $w = @(PfDump) | Where-Object { $_.ntabs -ge 2 } | Select-Object -First 1
  Check 'after a tab switch and the pointer away: faded' ((DumpVal $w 'idle_opacity') -eq '0.45') "tab window '$($w.title)' idle_opacity=$(DumpVal $w 'idle_opacity')"
  Pf fence detach Right | Out-Null; Start-Sleep -Milliseconds 900
  Place Right 608 200 300 260
  Start-Sleep -Milliseconds 600; Park
  Log "after detach: Left $((Win 'Left').L),$((Win 'Left').T) Right $((Win 'Right').L),$((Win 'Right').T)"

  Case 'A off, on, restart'
  SetIdle 1; Park
  IoIs 'idleOpacity 1 again: full' 'Left' 1.0
  ScreenIs 'idleOpacity 1 again: full on screen' 'Left' $false
  SetIdle 0.45; Park
  IoIs 'idleOpacity 0.45 again: faded' 'Left' 0.45
  PfQuit
  PfStart -Script (DumpScript 150 12)
  Park; Start-Sleep -Seconds 1
  Check 'the value is kept over a restart' ((IdleGet) -eq 0.45) "settings get idleOpacity: $(IdleGet)"
  IoIs 'after the restart a fence at rest is faded' 'Left' 0.45

  Case 'A values from the command line'
  foreach ($bad in '0.05', '1.5', '0', '-1', 'abc', 'null') {
    $out = Pf settings set idleOpacity $bad
    $said = if ($out -match '"message":"([^"]*)"') { $Matches[1] } else { "$out" }
    Check "settings set idleOpacity $bad is refused" ($out -match 'validation_failed' -and (IdleGet) -eq 0.45) "answer: $said; value now $(IdleGet)"
  }
  $out = Pf settings set idleOpacity 0.1; Start-Sleep -Milliseconds 500; Park
  IoIs 'the lowest value, 0.1, is taken' 'Left' 0.1
  $out = Pf settings set idleOpacity 0.37; Start-Sleep -Milliseconds 500
  IoIs 'a value between the four steps of the settings page is taken' 'Left' 0.37

  Case 'A value in a hand-edited config file'
  # A value out of range in config.json must not cost the user the whole file.
  PfQuit
  $cfg = "$PfApp\config\config.json"
  foreach ($v in '0', '7') {
    if (-not (Test-Path $cfg)) { Check "config.json is there for idleOpacity $v" $false 'the start before moved it aside'; continue }
    $text = [IO.File]::ReadAllText($cfg)
    $text = [regex]::Replace($text, '"idleOpacity":\s*[-\d.eE]+', "`"idleOpacity`": $v")
    [IO.File]::WriteAllText($cfg, $text, (New-Object Text.UTF8Encoding($false)))
    PfStart -Script (DumpScript 150 12)
    $titles = @(PfJson fence list | ForEach-Object { $_.title })
    $aside = @(Get-ChildItem "$PfApp\config" -Filter 'config.unreadable-*' -ErrorAction SilentlyContinue).Count
    $want = if ($v -eq '0') { 0.1 } else { 1 }
    Check "config.json with idleOpacity $v`: the fences are still there" ($titles -contains 'Left' -and $titles -contains 'Right' -and $aside -eq 0) "fences: $($titles -join ', '); config files moved aside: $aside"
    Check "config.json with idleOpacity $v loads as the nearest value" ((IdleGet) -eq $want) "settings get idleOpacity: $(IdleGet), expected $want"
    PfQuit
    Get-ChildItem "$PfApp\config" -Filter 'config.unreadable-*' -ErrorAction SilentlyContinue | Remove-Item -Force
  }
  PfStart -Script (DumpScript 150 12)
  SetIdle 0.45; Park
}

if ($Part -match 'B') {
  Case 'B the fade is gradual, and a jump with animations off'
  Park; IoWait 'Left' 0.45 | Out-Null
  $r = Win 'Left'; $p = Mid 'Left'
  function FadeSteps() {
    # Frames of the plate patch for 500 ms from the moment the pointer arrives; how many of
    # them are between the faded and the full plate.
    [G]::MoveTo($p[0] - 4, $p[1]); [G]::MoveTo($p[0], $p[1])
    [Px]::Burst($r.R - 60, $r.B - 40, 40, 24, 500, 0)
    $levels = @([Px]::Frames | ForEach-Object { $m = MeanRgb $_; [math]::Round((($m[0] + $m[1] + $m[2]) / 3.0 - $LumFull) / ($LumIdle - $LumFull), 2) })
    $mid = @($levels | Where-Object { $_ -gt 0.15 -and $_ -lt 0.85 }).Count
    return @($mid, $levels.Count, ($levels -join ' '))
  }
  $f = FadeSteps
  Check 'animations on: the fence comes back over several frames' ($f[0] -ge 2) "$($f[0]) of $($f[1]) frames between faded and full: $($f[2])"
  Park; IoWait 'Left' 0.45 | Out-Null
  [Anim]::Set($false); Start-Sleep -Milliseconds 800
  $f = FadeSteps
  Check 'animations off in Windows: it comes back in one step' ($f[0] -le 1) "$($f[0]) of $($f[1]) frames between faded and full: $($f[2])"
  Park
  IoIs 'animations off: faded again when the pointer leaves' 'Left' 0.45
  [Anim]::Set($true); Start-Sleep -Milliseconds 800

  Case 'B roll up and open while faded'
  Park
  $h0 = (Win 'Right').B - (Win 'Right').T
  Pf fence roll Right | Out-Null
  $seen = @(1..4 | ForEach-Object { Io 'Right' })
  Start-Sleep -Milliseconds 700
  Check 'rolling up with the pointer away: faded the whole time' (@($seen | Where-Object { [math]::Abs($_ - 0.45) -gt 0.02 }).Count -eq 0) "idle_opacity while it rolled: $($seen -join ' ')"
  $w = DumpOf 'Right' 4
  Check 'it ends rolled up, opaque root, faded' ($w.rolled -and (DumpVal $w 'opacity') -eq '1.00' -and (DumpVal $w 'idle_opacity') -eq '0.45') "rolled=$($w.rolled) opacity=$(DumpVal $w 'opacity') idle_opacity=$(DumpVal $w 'idle_opacity') height=$($w.b - $w.t)"
  Pf fence unroll Right | Out-Null
  # The pointer arrives while it opens.
  Start-Sleep -Milliseconds 60; Over 'Right'
  IoIs 'the pointer arrives while it opens: full' 'Right' 1.0
  Start-Sleep -Milliseconds 700
  $h1 = (Win 'Right').B - (Win 'Right').T
  Check 'it opens to its height' ($h1 -eq $h0) "height $h1, before $h0"
  Park
  IoIs 'open, the pointer away: faded' 'Right' 0.45

  Case 'B icons slide in a faded fence'
  Park
  Remove-Item -LiteralPath "$PfDesk\if d.txt" -Force
  $seen = @(1..5 | ForEach-Object { Io 'Right' })
  Check 'a file deleted in a faded fence: it stays faded while the icons slide' (@($seen | Where-Object { [math]::Abs($_ - 0.45) -gt 0.02 }).Count -eq 0) "idle_opacity while they slid: $($seen -join ' ')"
  ScreenIs 'and is faded on screen afterwards' 'Right' $true

  Case 'B drag and resize'
  $p = TitlePt 'Right'
  [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 200; MDown; Start-Sleep -Milliseconds 150
  Glide $p[0] $p[1] ($p[0] + 160) ($p[1] + 120) 12 30
  IoIs 'while the title is dragged: full' 'Right' 1.0
  MUp; Start-Sleep -Milliseconds 700
  IoIs 'let go with the pointer still on it: full' 'Right' 1.0
  Park
  IoIs 'the pointer left the moved fence: faded' 'Right' 0.45
  $r = ShadowOk 'Right' 0.45; Check 'the shadow of the moved fence is as faint as the plate' $r[0] $r[1]
  $r = Win 'Right'
  $ex = [int](($r.L + $r.R) / 2); $ey = $r.B - 2
  [G]::MoveTo($ex, $ey); Start-Sleep -Milliseconds 200; MDown; Start-Sleep -Milliseconds 150
  Glide $ex $ey $ex ($ey + 70) 10 30
  IoIs 'while the bottom edge is dragged: full' 'Right' 1.0
  # The pointer runs ahead of the edge and is let go off the fence.
  MUp; Start-Sleep -Milliseconds 300
  Park
  IoIs 'after the resize, the pointer away: faded' 'Right' 0.45
  $r2 = Win 'Right'
  Check 'the resize took place' (($r2.B - $r2.T) -gt ($r.B - $r.T) + 30) "height $($r.B - $r.T) -> $($r2.B - $r2.T)"
  Place Right 608 200 300 260; Start-Sleep -Milliseconds 700; Park

  Case 'B a file dragged from another fence'
  $from = ItemPt 'Left' 'if a'; $to = Mid 'Right'
  [G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 200; MDown; Start-Sleep -Milliseconds 200
  Glide $from[0] $from[1] $to[0] $to[1] 14 30; Start-Sleep -Milliseconds 400
  IoIs 'a file from another fence held over a faded fence: that fence is full' 'Right' 1.0
  IoIs 'the fence the file comes from stays full during the drag' 'Left' 1.0
  MUp; Start-Sleep -Milliseconds 900
  Check 'the drop lands in the fence' ((WaitItemFence 'if a.txt' 'Right' 5) -eq 'Right') "if a.txt is in $(ItemFence 'if a.txt')"
  Park
  IoIs 'after the drop, the pointer away: the target is faded' 'Right' 0.45
  IoIs 'and the source is faded' 'Left' 0.45

  Case 'B a file dragged from File Explorer'
  New-Item -ItemType Directory -Force $IfDir | Out-Null
  Set-Content -LiteralPath "$IfDir\if x.txt" 'x'
  $ex = ExOpen $IfDir 960 420 620 400
  $from = ExItemPt $ex 'if x.txt'
  Check 'an Explorer window with the file' ($ex -ne [IntPtr]::Zero -and $from) "window $ex, file at $from"
  $to = Mid 'Left'
  [G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 200; MDown; Start-Sleep -Milliseconds 200
  Glide $from[0] $from[1] $to[0] $to[1] 16 30; Start-Sleep -Milliseconds 500
  IoIs 'a file from Explorer held over a faded fence: full' 'Left' 1.0
  # Carried on, off the fence, onto the desktop: the fence fades while the drag goes on.
  $s = FreeSpot 500 620
  Glide $to[0] $to[1] $s[0] $s[1] 10 30; Start-Sleep -Milliseconds 500
  IoIs 'the drag left the fence: faded again' 'Left' 0.45
  Glide $s[0] $s[1] $to[0] $to[1] 10 30; Start-Sleep -Milliseconds 400
  MUp; Start-Sleep -Milliseconds 1500
  Check 'the drop from Explorer lands in the fence' ((WaitItemFence 'if x.txt' 'Left' 6) -eq 'Left') "if x.txt is in $(ItemFence 'if x.txt')"
  CloseWindows; Park
  IoIs 'after the drop from Explorer, the pointer away: faded' 'Left' 0.45

  Case 'B a fence at 25 % takes clicks'
  SetIdle 0.25; Park
  IoIs 'idleOpacity 0.25: faded to 0.25' 'Left' 0.25
  Shot 'B-25-percent'
  $p = ItemPt 'Left' 'if b'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  $w = DumpOf 'Left' 4
  Check 'a click on an icon of a fence at 25 % selects it' ((DumpVal $w 'selected') -eq '1') "selected=$(DumpVal $w 'selected')"
  Park; DeskClick | Out-Null; Park
  $m = RClickItem 'Left' 'if b'
  Check 'a right-click from the desktop opens the icon menu' ($m -ne [IntPtr]::Zero) "menu $m"
  CloseMenus; DeskClick | Out-Null; Park
  $p = ItemPt 'Left' 'if b'; DblClick $p[0] $p[1]
  $np = $null; for ($i = 0; $i -lt 30 -and -not $np; $i++) { Start-Sleep -Milliseconds 200; $np = Get-Process notepad -ErrorAction SilentlyContinue }
  Check 'a double-click from the desktop opens the file' ([bool]$np) "notepad running: $([bool]$np)"
  CloseWindows; DeskClick | Out-Null; Park
  $from = ItemPt 'Left' 'if c'; $to = Mid 'Right'
  DragTo $from[0] $from[1] $to[0] ($to[1] + 50) 14
  Check 'a file dropped on a fence at 25 % lands in it' ((WaitItemFence 'if c.txt' 'Right' 5) -eq 'Right') "if c.txt is in $(ItemFence 'if c.txt')"
  Park
  IoIs 'after that drop, the pointer away: faded to 0.25' 'Right' 0.25
  SetIdle 0.45; Park

  Case 'B tools'
  Pf fence create --tool note --title Memo --rect '300,480,300,180' | Out-Null
  Pf fence create --tool clock --title Clock --rect '608,480,300,180' | Out-Null
  Start-Sleep -Seconds 1; Park
  IoIs 'a note at rest is faded' 'Memo' 0.45
  IoIs 'a clock at rest is faded' 'Clock' 0.45
  $seen = @(1..6 | ForEach-Object { Start-Sleep -Milliseconds 400; Io 'Clock' })
  Check 'a clock stays faded while it ticks' (@($seen | Where-Object { [math]::Abs($_ - 0.45) -gt 0.02 }).Count -eq 0) "idle_opacity over 3 s: $($seen -join ' ')"
  $p = Mid 'Memo'; [G]::Click($p[0], $p[1])
  $ed = WaitWin 'PecoFence.NoteEdit' 8
  Check 'a click in the note opens its editor' ($ed -ne [IntPtr]::Zero) "editor $ed"
  [G]::Type('milk'); Start-Sleep -Milliseconds 300; Park
  IoIs 'a note whose editor is open, the pointer away: full' 'Memo' 1.0
  Shot 'B-note-editor'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 500; DeskClick | Out-Null; Park
  IoIs 'the editor closed, the desktop clicked: faded' 'Memo' 0.45
  # A script fence whose script runs again.
  New-Item -ItemType Directory -Force $IfDir | Out-Null
  Set-Content -LiteralPath "$IfDir\tick.ps1" 'Write-Output ("tick " + (Get-Date -Format HH:mm:ss.fff))'
  Pf fence create --script "$IfDir\tick.ps1" --title Ticker --rect '916,480,300,180' | Out-Null
  Start-Sleep -Seconds 4; Park
  IoIs 'a script fence at rest is faded' 'Ticker' 0.45
  Pf fence refresh Ticker | Out-Null
  $seen = @(1..6 | ForEach-Object { Start-Sleep -Milliseconds 300; Io 'Ticker' })
  Check 'a script fence stays faded while its script runs again' (@($seen | Where-Object { [math]::Abs($_ - 0.45) -gt 0.02 }).Count -eq 0) "idle_opacity over 2 s: $($seen -join ' ')"
  # A folder fence and its filter box.
  New-Item -ItemType Directory -Force "$IfDir\folder" | Out-Null
  'p.txt', 'q.txt' | ForEach-Object { Set-Content -LiteralPath "$IfDir\folder\$_" 'x' }
  Pf fence create --portal "$IfDir\folder" --title Folder --rect '916,200,300,260' | Out-Null
  Start-Sleep -Seconds 1
  $p = Mid 'Folder'; [G]::Click($p[0], $p[1] + 60); Start-Sleep -Milliseconds 300
  Chord 0x11 0x46
  $fb = WaitWin 'PecoFence.Filter' 6; Park
  Check 'Ctrl+F opens the filter box of a folder fence' ($fb -ne [IntPtr]::Zero) "filter box $fb"
  IoIs 'the filter box is open, the pointer away: full' 'Folder' 1.0
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400; DeskClick | Out-Null; Park
  IoIs 'the filter box closed, the desktop clicked: faded' 'Folder' 0.45
  Pf fence delete Folder | Out-Null; Pf fence delete Memo | Out-Null; Pf fence delete Clock | Out-Null; Pf fence delete Ticker | Out-Null
  Start-Sleep -Milliseconds 600

  Case 'B Peek over another window'
  $np = Start-Process notepad -PassThru; Start-Sleep -Seconds 2
  $nw = [G]::FindWindowW('Notepad', [NullString]::Value)
  [L]::MoveWindow($nw, 200, 100, 900, 500, $true) | Out-Null; Start-Sleep -Milliseconds 500
  [G]::Click(1000, 110); Start-Sleep -Milliseconds 400; [G]::MoveTo(1050, 590); Start-Sleep -Milliseconds 400
  PeekKey; Start-Sleep -Milliseconds 900
  $w = DumpOf 'Left' 4
  Check 'Peek over a window that hides the fence: the fence is shown, full' ((DumpVal $w 'idle_opacity') -eq '1.00' -and (DumpVal $w 'opacity') -eq '1.00') "idle_opacity=$(DumpVal $w 'idle_opacity') opacity=$(DumpVal $w 'opacity') peek_faded=$(DumpVal $w 'peek_faded')"
  $r = ShadowOk 'Left' 1.0; Check 'its shadow has its full strength' $r[0] $r[1]
  [G]::Press(0x1B); Start-Sleep -Milliseconds 900
  CloseWindows; Park; Start-Sleep -Milliseconds 500
  $w = DumpOf 'Left' 4
  Check 'after the Peek and the window: shown, faded' ((DumpVal $w 'idle_opacity') -eq '0.45' -and (DumpVal $w 'opacity') -eq '1.00') "idle_opacity=$(DumpVal $w 'idle_opacity') opacity=$(DumpVal $w 'opacity')"
  ScreenIs 'faded on screen after the Peek' 'Left' $true
  $r = ShadowOk 'Left' 0.45; Check 'the shadow is as faint as the plate after the Peek' $r[0] $r[1]

  Case 'B show desktop'
  Chord 0x5B 0x44; Start-Sleep -Milliseconds 1200; Park
  Check 'Win+D: the fences stay' ([G]::IsWindowVisible((Hw 'Left'))) "Left visible: $([G]::IsWindowVisible((Hw 'Left')))"
  IoIs 'Win+D: faded' 'Left' 0.45
  ScreenIs 'Win+D: faded on screen' 'Left' $true
  Over 'Left'
  IoIs 'Win+D: the pointer over a fence brings it back' 'Left' 1.0
  Park
  Chord 0x5B 0x44; Start-Sleep -Milliseconds 1200; Park
  IoIs 'Win+D again: faded' 'Left' 0.45

  Case 'B virtual desktops'
  VdTwo 'One' 'Two'
  Pf fence set Right virtualDesktops Two | Out-Null; Start-Sleep -Milliseconds 800
  Check 'a fence bound to the second desktop is hidden on the first' (-not [G]::IsWindowVisible((Hw 'Right'))) "Right visible: $([G]::IsWindowVisible((Hw 'Right')))"
  Park; VdRight; Start-Sleep -Milliseconds 1500
  Check 'it shows on the second desktop' ([G]::IsWindowVisible((Hw 'Right'))) "Right visible: $([G]::IsWindowVisible((Hw 'Right')))"
  IoIs 'on its desktop, the pointer away: faded' 'Right' 0.45
  ScreenIs 'on its desktop: faded on screen' 'Right' $true
  $r = ShadowOk 'Right' 0.45; Check 'on its desktop: the shadow is as faint as the plate' $r[0] $r[1]
  Over 'Right'
  IoIs 'on its desktop: the pointer brings it back' 'Right' 1.0
  # The desktop is switched with the pointer still on the fence; back with the pointer elsewhere.
  VdLeft; Start-Sleep -Milliseconds 1500
  Park; VdRight; Start-Sleep -Milliseconds 1500
  IoIs 'left behind under the pointer by a desktop switch, back with the pointer elsewhere: faded' 'Right' 0.45
  Park; VdLeft; Start-Sleep -Milliseconds 1500
  IoIs 'back on the first desktop: the fence there is faded' 'Left' 0.45
  Pf fence set Right virtualDesktops null | Out-Null; Start-Sleep -Milliseconds 600
  VdOnlyOne; ClearPfNotices; Park

  Case 'B title on hover and a fence without a plate'
  # Two older settings that follow the pointer as well: the title row that folds away, and a
  # fence whose plate and shadow show only under the pointer.
  Pf settings set rollUp.titleOnHover true | Out-Null; Start-Sleep -Milliseconds 900; Park
  IoIs 'title on hover: at rest the fence is faded' 'Left' 0.45
  $r = ShadowOk 'Left' 0.45; Check 'title on hover: the shadow is as faint as the plate' $r[0] $r[1]
  $w = DumpOf 'Left' 4
  Check 'title on hover: the title row is folded away at rest' ([int](DumpVal $w 'plate_inset') -gt 0) "plate_inset=$(DumpVal $w 'plate_inset')"
  Over 'Left'; Start-Sleep -Milliseconds 400
  IoIs 'title on hover: under the pointer the fence is full' 'Left' 1.0
  $w = DumpOf 'Left' 4
  Check 'title on hover: the title row is back under the pointer' ((DumpVal $w 'plate_inset') -eq '0') "plate_inset=$(DumpVal $w 'plate_inset')"
  $r = ShadowOk 'Left' 1.0; Check 'title on hover: the shadow is back too' $r[0] $r[1]
  Park
  IoIs 'title on hover: faded again when the pointer leaves' 'Left' 0.45
  Pf settings set rollUp.titleOnHover false | Out-Null
  Pf fence set Right opacity transparent | Out-Null; Start-Sleep -Milliseconds 900; Park
  IoIs 'a fence without a plate: at rest its icons are faded' 'Right' 0.45
  $w = DumpOf 'Right' 4
  Check 'a fence without a plate: no shadow at rest' ((DumpVal $w 'shadow_alpha') -eq '0') "shadow_alpha=$(DumpVal $w 'shadow_alpha') plate_alpha=$(DumpVal $w 'plate_alpha')"
  Over 'Right'; Start-Sleep -Milliseconds 400
  IoIs 'a fence without a plate: under the pointer it is full' 'Right' 1.0
  $r = ShadowOk 'Right' 1.0; Check 'a fence without a plate: plate and shadow show under the pointer' ($r[0] -and [double](DumpVal (DumpOf 'Right' 4) 'plate_alpha') -gt 0.9) $r[1]
  Park
  IoIs 'a fence without a plate: faded again when the pointer leaves' 'Right' 0.45
  Pf fence set Right opacity default | Out-Null; Start-Sleep -Milliseconds 600; Park

  Case 'B neighbours'
  # A rolled-up fence above another: opened by the resting pointer it lies over the lower one.
  Place Left 300 200 300 260
  Place Right 300 244 300 200
  Start-Sleep -Milliseconds 600
  Pf fence roll Left | Out-Null; Start-Sleep -Milliseconds 900
  Place Right 300 244 300 200; Start-Sleep -Milliseconds 700; Park
  $l = Win 'Left'; $rr = Win 'Right'
  Log "rolled Left $($l.L),$($l.T),$($l.R),$($l.B); Right $($rr.L),$($rr.T),$($rr.R),$($rr.B)"
  $p = TitlePt 'Left'; [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 200; [G]::MoveTo($p[0] + 2, $p[1]); Start-Sleep -Milliseconds 1500
  $l = Win 'Left'
  Check 'the rolled fence opened under the resting pointer' (($l.B - $l.T) -gt 150) "Left height $($l.B - $l.T)"
  IoIs 'the opened fence is full' 'Left' 1.0
  IoIs 'the fence under it stays faded' 'Right' 0.45
  # Down through the opened fence and out of its bottom edge, into the lower fence.
  $rr = Win 'Right'
  Glide $p[0] $p[1] $p[0] ($rr.B - 20) 14 30; Start-Sleep -Milliseconds 1500
  $l = Win 'Left'
  Log "after leaving through the bottom: Left height $($l.B - $l.T), pointer at $($p[0]),$($rr.B - 20), window there: $([W]::RootClassAt($p[0], ($rr.B - 20)))"
  IoIs 'the pointer left through the lower fence: the upper one is faded' 'Left' 0.45
  IoIs 'the lower fence, now under the pointer, is full' 'Right' 1.0
  Park
  IoIs 'the pointer on the desktop: the lower one fades' 'Right' 0.45
  # A fence pushed away from under the pointer: the pointer rests on the lower fence, the upper
  # one opens from the command line and pushes it down.
  $rr = Win 'Right'
  [G]::MoveTo($rr.L + 150, $rr.T + 16); Start-Sleep -Milliseconds 200; [G]::MoveTo($rr.L + 152, $rr.T + 16); Start-Sleep -Milliseconds 400
  IoIs 'the pointer on the lower fence: full' 'Right' 1.0
  Pf fence unroll Left | Out-Null; Start-Sleep -Milliseconds 1500
  $l = Win 'Left'; $r2 = Win 'Right'
  $under = [W]::RootClassAt(($rr.L + 152), ($rr.T + 16))
  Log "after the push: Left $($l.T)..$($l.B), Right $($r2.T)..$($r2.B), pointer y $($rr.T + 16)"
  $onLeft = ($rr.T + 16) -lt $l.B
  if ($onLeft) {
    IoIs 'the fence pushed away from under the pointer fades' 'Right' 0.45
    IoIs 'the fence that grew under the pointer is full' 'Left' 1.0
  } else {
    Log 'the push did not move the lower fence from under the pointer on this build'
  }
  Park
  IoIs 'the pointer away: the pushed fence is faded' 'Right' 0.45
  IoIs 'the pointer away: the opened fence is faded' 'Left' 0.45
}

if ($Part -match 'C') {
  Case 'C the card on the settings page'
  Check 'part C runs on a debug build' $PfDebugBuild 'run.sh --bin target/wintest-bin-debug'
  SetIdle 1; Park
  Check 'the settings page opens' (PfSettings) "window $([G]::FindWindowW('PecoFence.Settings', [NullString]::Value))"
  $sel = "document.querySelector('[data-bind=idleOpacity]')"
  $card = Js "(() => { const s = $sel, c = s.closest('.card'); let h = c; while (h && h.tagName !== 'H3') h = h.previousElementSibling; return { title: c.querySelector('.t').textContent, hint: c.querySelector('.s').textContent, value: s.value, options: [...s.options].map(o => o.textContent).join(' '), heading: h ? h.textContent : '', page: c.closest('section').id }; })()"
  Check 'the card is on General under Appearance' ($card.page -eq 'page-general' -and $card.heading -eq 'Appearance') "page $($card.page), heading '$($card.heading)'"
  Check 'it is named and explained, with four steps and 100% chosen' ($card.title -eq 'Idle opacity' -and $card.hint -and $card.options -eq '100% 70% 45% 25%' -and $card.value -eq '1') "title '$($card.title)', hint '$($card.hint)', options '$($card.options)', value $($card.value)"
  # The keyboard on the selector: two steps down is 45 %.
  PageClick '[data-bind=idleOpacity]' 0 -40
  Js "$sel.focus()" | Out-Null
  [G]::Press(0x28); Start-Sleep -Milliseconds 150; [G]::Press(0x28); Start-Sleep -Milliseconds 700
  Check 'two Down keys on the selector choose 45%' ((Js "$sel.value") -eq '0.45' -and (IdleGet) -eq 0.45) "selector $(Js "$sel.value"), settings get idleOpacity $(IdleGet)"
  $r = PageRect '[data-bind=idleOpacity]'
  $sw = Rect ([G]::FindWindowW('PecoFence.Settings', [NullString]::Value))
  [G]::MoveTo($r.cx - 300, $r.cy + 60); Start-Sleep -Milliseconds 600
  IoIs 'the fences fade as soon as it is chosen' 'Right' 0.45
  Shot 'C-settings-card'
  Crop 'C-card' ($sw.L + 200) ($r.cy - 150) ($sw.R - $sw.L - 200) 230 | Out-Null
  # A value the command line set between the steps shows as a choice of its own.
  SetIdle 0.37
  $own = JsWait "$sel.value === '0.37' && [...$sel.options].map(o => o.textContent).join(' ')" 6
  Check 'a value set from the command line shows as its own choice' ($own -eq '100% 70% 45% 25% 37%') "options: $own"
  SetIdle 0.7
  $back = JsWait "$sel.value === '0.7' && [...$sel.options].map(o => o.textContent).join(' ')" 6
  Check 'back on a step, that choice is gone' ($back -eq '100% 70% 45% 25%') "options: $back"
  # Find a setting.
  foreach ($word in 'idle', 'fade out', 'opacity', '45%') {
    PageType '#settingsSearch' $word
    $hits = Js "[...document.querySelectorAll('#settingsSearchList [role=option] b')].map(b => b.textContent).join(' | ')"
    Check "Find a setting: '$word' lists the card" ($hits -match 'Idle opacity') "hits: $hits"
  }
  Shot 'C-settings-search'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300
  # The other ten languages have the card's two texts and the names for the search.
  $seen = @()
  foreach ($lang in 'zh-CN', 'zh-TW', 'ja', 'ko', 'de', 'fr', 'es', 'pt-BR', 'ru', 'it') {
    Pf settings set language $lang | Out-Null
    $t = JsWait "(() => { const c = $sel.closest('.card'); const t = c.querySelector('.t').textContent; return t !== 'Idle opacity' && (t + ' / ' + c.querySelector('.s').textContent); })()" 6
    $seen += "$lang`: $t"
    Check "the card is translated in $lang" ([bool]$t -and $t -notmatch 'Idle opacity|Fade backgrounds') "$t"
  }
  Pf settings set language en | Out-Null
  $h = [G]::FindWindowW('PecoFence.Settings', [NullString]::Value)
  [L]::PostMessageW($h, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; Start-Sleep -Milliseconds 800
}

Case 'cleanup'
SetIdle 1
PfQuit
IfClean
Remove-Item -Recurse -Force $IfDir -ErrorAction SilentlyContinue
[Anim]::Set($AnimWas)
