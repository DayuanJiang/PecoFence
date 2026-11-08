# Desktop input checks (fix round 2026-11-07): the desktop's double-click after a start with the
# taskbar in front and after closing windows (S2), the selection and keyboard after Properties
# opened from a fence closes (U4), the hover look after a title drag cancelled with Esc (S9), and
# the fences while Win+D shows and leaves the desktop (P5).
# $Part picks the cases: A (U4, S9, Win+D), B (restart, 20 Notepad rounds), C (watchdog restart
# after a crash with a fence in front, about 3 minutes); default all three. The start right at
# sign-in needs a real reboot: the Hyper-V boot probe of the round checks it.
if (-not $Part) { $Part = 'ABC' }
if ($FxBin) { $PfBin = $FxBin }
. C:\pf\kit\zorder.ps1
$PfAllowLogErrors = $true
$free = @(1100, 650)

function Sel($t) { $d = DumpOf $t 3; return "selected=$(DumpVal $d 'selected') focused=$(DumpVal $d 'focused')" }
function Inside($t) { return (DumpVal (DumpOf $t 3) 'mouse_inside') }
function FRect($t) { return (FenceGet $t).windowRect }
function Park() { [G]::MoveTo(1100, 830); Start-Sleep -Milliseconds 300 }
function PropsWin() { return @([W]::Tops('#32770') | Where-Object { [W]::Title($_) -match 'Properties' -and [W]::ProcOf($_) -eq 'pecofence' }) }
function WaitProps($sec = 6) { for ($i = 0; $i -lt $sec * 5; $i++) { $p = @(PropsWin); if ($p.Count) { return $p[0] }; Start-Sleep -Milliseconds 200 }; return [IntPtr]::Zero }
function OpenNote($tag, $x, $y, $w, $h) {
  $f = "$PfRun\$tag.txt"; Set-Content $f "window $tag" -Encoding ASCII
  $p = Start-Process notepad.exe -ArgumentList $f -PassThru
  for ($i = 0; $i -lt 50; $i++) { Start-Sleep -Milliseconds 200; $p.Refresh(); if ($p.MainWindowHandle -ne 0) { break } }
  if ($p.MainWindowHandle -ne 0) { [L]::MoveWindow([IntPtr]$p.MainWindowHandle, $x, $y, $w, $h, $true) | Out-Null }
  return [IntPtr]$p.MainWindowHandle
}
# Double-click on the bare desktop: did the fences hide (and show again on a second one)?
function QuickHideWorks() {
  $n0 = VisFences; DblClick $free[0] $free[1]; Start-Sleep -Milliseconds 1200; $n1 = VisFences
  if ($n1 -eq 0) { DblClick $free[0] $free[1]; Start-Sleep -Milliseconds 1200 }
  $n2 = VisFences
  if ($n2 -eq 0) { Pf fence show-all | Out-Null; Start-Sleep -Milliseconds 800 }
  return @(($n0 -gt 0 -and $n1 -eq 0 -and $n2 -eq $n0), "$n0 -> $n1 -> $n2")
}
function Seed() {
  CloseWindows
  foreach ($i in 1..6) { Set-Content "$PfDesk\fx-in-$i.txt" "fx $i" }
  PfStart -Script (DumpScript 250 25)
  Pf fence create --title Alpha --rect '300,200,300,250' | Out-Null
  Pf fence create --title Bravo --rect '700,200,300,250' | Out-Null
  Pf fence create --title Photos --rect '700,520,240,200' | Out-Null
  Pf item move "$PfDesk\fx-in-1.txt" "$PfDesk\fx-in-2.txt" --to Alpha | Out-Null
  Pf item move "$PfDesk\fx-in-3.txt" --to Bravo | Out-Null
  Pf item move "$PfDesk\fx-in-4.txt" "$PfDesk\fx-in-5.txt" --to Photos | Out-Null
  Pf fence set --all sort name | Out-Null
  [Anim]::Set($true)
  Start-Sleep -Seconds 2
  Park
}

Case 'seed'
Seed

if ($Part -match 'A') {
  Case 'U4 Properties closed with Esc'
  $p = ItemPt 'Alpha' 'fx-in-1.txt'
  if (-not $p) { $p = ItemPt 'Alpha' 'fx-in-1' }
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600
  $before = Sel 'Alpha'
  [G]::KeyDown(0x12); [G]::Press(0x0D); [G]::KeyUp(0x12)
  $dlg = WaitProps
  Check 'Alt+Enter opens Properties' ($dlg -ne [IntPtr]::Zero) "before: $before"
  Start-Sleep -Milliseconds 800
  [G]::Press(0x1B); Start-Sleep -Milliseconds 1200
  $a = FRect 'Alpha'; Crop 'u4-after-esc' ($a.x - 10) ($a.y - 10) ($a.w + 20) ($a.h + 20) | Out-Null
  $after = Sel 'Alpha'
  Check 'after Esc the fence keeps its selection' ($after -match 'selected=1\b') "before: $before; after: $after"
  Check 'and has the keyboard again' ((Fg) -match '^PecoFence.Fence') "fg=$(Fg)"
  [Windows.Forms.Clipboard]::Clear()
  Chord 0x11 0x43; Start-Sleep -Milliseconds 700
  Check 'Ctrl+C then copies the selected file' (@(@(ClipFiles) -match 'fx-in-1').Count -gt 0) "clip=$((ClipFiles) -join ', ')"

  Case 'U4 Properties closed with its X'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600
  [G]::KeyDown(0x12); [G]::Press(0x0D); [G]::KeyUp(0x12)
  $dlg = WaitProps
  Start-Sleep -Milliseconds 800
  $r = Rect $dlg
  [G]::Click($r.R - 20, $r.T + 15); Start-Sleep -Milliseconds 1200
  Check 'the X closed it' (@(PropsWin).Count -eq 0) ''
  Check 'after the X the fence keeps its selection and the keyboard' ((Sel 'Alpha') -match 'selected=1\b' -and (Fg) -match '^PecoFence.Fence') "$(Sel 'Alpha') fg=$(Fg)"

  Case 'U4 a click on the desktop while Properties is open'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600
  [G]::KeyDown(0x12); [G]::Press(0x0D); [G]::KeyUp(0x12)
  $dlg = WaitProps
  Start-Sleep -Milliseconds 800
  [G]::Click($free[0], $free[1]); Start-Sleep -Milliseconds 900
  Check 'still ends the selection, as a desktop click does' ((Sel 'Alpha') -match 'selected=0\b') "$(Sel 'Alpha') fg=$(Fg)"
  foreach ($d in @(PropsWin)) { [W]::PostMessageW($d, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null }
  Start-Sleep -Milliseconds 900
  Check 'and closing Properties then leaves the desktop in front' ((Fg) -match '^(Progman|WorkerW)') "fg=$(Fg)"

  Case 'U4 Properties over Notepad'
  $n = OpenNote 'U4' 1150 80 400 300
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600
  [G]::KeyDown(0x12); [G]::Press(0x0D); [G]::KeyUp(0x12)
  $dlg = WaitProps
  Start-Sleep -Milliseconds 800
  [G]::Press(0x1B); Start-Sleep -Milliseconds 1200
  Check 'with Notepad open too, Esc gives the fence the keyboard back' ((Sel 'Alpha') -match 'selected=1\b' -and (Fg) -match '^PecoFence.Fence') "$(Sel 'Alpha') fg=$(Fg)"
  CloseWindows
  [G]::Click($free[0], $free[1]); Start-Sleep -Milliseconds 600

  foreach ($style in 'liquidGlass', 'fluent') {
    Case "S9 Esc on a merge hover, $style"
    Pf settings set themeStyle $style | Out-Null; Start-Sleep -Seconds 2
    Park; Start-Sleep -Milliseconds 800
    $res = @()
    for ($i = 1; $i -le 3; $i++) {
      $a = FRect 'Photos'; $b = FRect 'Bravo'
      [G]::MoveTo($a.x + 80, $a.y + 16); Start-Sleep -Milliseconds 200; MDown; Start-Sleep -Milliseconds 150
      Glide ($a.x + 80) ($a.y + 16) ($b.x + 120) ($b.y + 16) 24 25; Start-Sleep -Milliseconds 600
      [G]::Press(0x1B); Start-Sleep -Milliseconds 300
      MUp; Start-Sleep -Milliseconds 700
      Park; Start-Sleep -Milliseconds 1500
      $res += "try ${i}: inside=$(Inside 'Photos') tabHost='$((FenceGet 'Photos').tabHost)'"
      if ((FenceGet 'Photos').tabHost) { Pf fence detach Photos | Out-Null; Start-Sleep -Milliseconds 800 }
      Pf fence move Photos --rect '700,520,240,200' | Out-Null; Start-Sleep -Milliseconds 600
    }
    $p2 = FRect 'Photos'; Crop "s9-$style" ($p2.x - 10) ($p2.y - 10) ($p2.w + 20) 60 | Out-Null
    $stuck = @($res | Where-Object { $_ -match 'inside=true' }).Count
    Check "$style after Esc and leaving, the fence knows the pointer left" ($stuck -eq 0) "$stuck of 3 stuck: $($res -join ' | ')"
    $a = FRect 'Photos'; DragTo ($a.x + 80) ($a.y + 16) 1150 600; Park; Start-Sleep -Milliseconds 1500
    Check "$style after a normal move and leaving, too" ((Inside 'Photos') -ne 'true') "inside=$(Inside 'Photos')"
    Pf fence move Photos --rect '700,520,240,200' | Out-Null; Start-Sleep -Milliseconds 600
  }

  Case 'P5 Win+D with windows open'
  $rows = @()
  for ($k = 1; $k -le 3; $k++) {
    CloseWindows
    OpenNote "N$k" 100 100 700 500 | Out-Null
    OpenNote "M$k" 650 150 700 500 | Out-Null
    Start-Sleep -Milliseconds 800
    $in = ZsStats ([Zs]::Run(1600, 150, 0x44, [NullString]::Value, 0))
    Start-Sleep -Seconds 1
    $out = ZsStats ([Zs]::Run(1600, 150, 0x44, [NullString]::Value, 0))
    $rows += $in.covered, $out.over
    Check "Win+D #${k}: the fences stay in view and do not cover the restored windows" ($in.covered -le 30 -and $out.covered -eq 0 -and $out.over -le 30) "in: covered=$($in.covered) ms topmost=$($in.topmost); out: covered=$($out.covered) over windows=$($out.over) ms"
    Start-Sleep -Seconds 1
  }
  Case 'P5 Win+D on a bare desktop'
  CloseWindows; Start-Sleep -Milliseconds 800
  [G]::Click($free[0], $free[1]); Start-Sleep -Milliseconds 600
  for ($k = 1; $k -le 2; $k++) {
    $in = ZsStats ([Zs]::Run(1600, 150, 0x44, [NullString]::Value, 0))
    Start-Sleep -Seconds 1
    $out = ZsStats ([Zs]::Run(1600, 150, 0x44, [NullString]::Value, 0))
    Check "bare desktop Win+D #${k}: the fences stay in view" ($in.covered -le 30 -and $out.covered -eq 0) "in: covered=$($in.covered) ms; out: covered=$($out.covered) ms"
    Start-Sleep -Seconds 1
  }
}

if ($Part -match 'B') {
  # PfQuit ends the app the way a sign-out does; with a fence in front, Windows gives the
  # foreground to the taskbar, and a click on the desktop then brings it back without any
  # foreground event (Windows 11 24H2 and Server 2025).
  for ($k = 1; $k -le 5; $k++) {
    Case "S2 restart with a fence in front #$k"
    $wr = (PfFence 'Alpha').windowRect; [G]::Click($wr.x + 50, $wr.y + 96); Start-Sleep -Milliseconds 600
    $fg = Fg
    PfQuit; $gone = Fg
    PfStart -Script (DumpScript 250 25); Start-Sleep -Seconds 2
    [G]::Click($free[0], $free[1]); Start-Sleep -Milliseconds 700
    $q = QuickHideWorks
    Check "quick hide after a restart #$k" $q[0] "$($q[1]); fg before: $fg; when it was gone: $gone; now: $(Fg)"
    $m = PfLogAll | Where-Object { $_ -match 'in front at start' } | Select-Object -Last 1
    Log "start line: $m"
    if (-not $q[0]) { [G]::Click(200, 876); Start-Sleep -Milliseconds 500; [G]::Click($free[0], $free[1]); Start-Sleep -Milliseconds 500 }
  }
  Case 'S2 twenty Notepad rounds'
  $fails = @()
  for ($k = 1; $k -le 20; $k++) {
    OpenNote "R$k" 200 120 600 400 | Out-Null
    Start-Sleep -Milliseconds 500
    Chord 0x12 0x73; Start-Sleep -Milliseconds 900
    if (Get-Process notepad -ErrorAction SilentlyContinue) { Get-Process notepad | Stop-Process -Force; Start-Sleep -Milliseconds 500 }
    $q = QuickHideWorks
    if (-not $q[0]) { $fails += "round ${k}: $($q[1]) fg=$(Fg)" }
  }
  Check 'after Alt+F4 on Notepad the first double-click hides the fences, 20 rounds' ($fails.Count -eq 0) "$($fails.Count) of 20 failed: $($fails -join ' | ')"

  # Windows 10 often reports no foreground change when a click on the desktop comes from an
  # app's window (the Copilot window at sign-in did it in the guests).
  Case 'S2 a double-click on the desktop while Notepad is in front, 10 rounds'
  $fails = @()
  $n = OpenNote 'App' 150 120 500 350
  for ($k = 1; $k -le 10; $k++) {
    [G]::Click(400, 140); Start-Sleep -Milliseconds 700
    $fg = Fg
    $q = QuickHideWorks
    if (-not $q[0]) { $fails += "round ${k}: $($q[1]) fg before=$fg" }
  }
  CloseWindows
  Check 'with an app in front, the first double-click on the desktop hides the fences, 10 rounds' ($fails.Count -eq 0) "$($fails.Count) of 10 failed: $($fails -join ' | ')"
}

if ($Part -match 'C') {
  # The watchdog starts PecoFence again after a crash, but not within a minute of its start.
  PfQuit
  $env:PECOFENCE_INSTANCE = $PfInst; $env:RUST_LOG = 'pecofence=debug'
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $PfApp
  WaitWin 'PecoFence.Control' 20 -Hidden | Out-Null
  for ($k = 1; $k -le 2; $k++) {
    Case "S2 watchdog restart after a crash with a fence in front #$k"
    Start-Sleep -Seconds 62
    $wr = (PfFence 'Alpha').windowRect; [G]::Click($wr.x + 50, $wr.y + 96); Start-Sleep -Milliseconds 600
    $fg = Fg
    $old = (Get-Process pecofence | Select-Object -First 1).Id
    [Th]::Terminate($old, [uint32]3221225477) | Out-Null
    $gone = $null
    for ($i = 0; $i -lt 100; $i++) { Start-Sleep -Milliseconds 200; $np = Get-Process pecofence -ErrorAction SilentlyContinue | Where-Object { $_.Id -ne $old }; if (-not $gone -and -not (Get-Process -Id $old -ErrorAction SilentlyContinue)) { $gone = Fg }; if ($np) { break } }
    $h = WaitWin 'PecoFence.Control' 30 -Hidden
    Start-Sleep -Seconds 4
    $fgStart = Fg
    [G]::Click($free[0], $free[1]); Start-Sleep -Milliseconds 700
    $q = QuickHideWorks
    Check "quick hide after the watchdog restart #$k" $q[0] "$($q[1]); restarted=$($h -ne [IntPtr]::Zero); fg before the crash: $fg; when it was gone: $gone; after the restart: $fgStart"
  }
  PfQuit
  Get-Process pecofence-watchdog -ErrorAction SilentlyContinue | Stop-Process -Force
}

Case 'cleanup'
CloseWindows
PfQuit
foreach ($i in 1..6) { Remove-Item "$PfDesk\fx-in-$i.txt" -ErrorAction SilentlyContinue }
