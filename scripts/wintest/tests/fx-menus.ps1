# Fence review fix round (menus): S14, P7, P9. About 5 minutes.
# S14: a popup menu opened while another program takes the foreground (a toast) still closes
#      with Esc or a click elsewhere, and PecoFence goes on answering. assets/foreground-steal.cs
#      is a toast-like window of its own process that takes the foreground on a signal.
# P7:  while a menu is open, another fence's animation goes on (it stopped half way before).
# P9:  the first item menu after a start opens about as fast as the later ones.
Add-Type @'
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
public static class MenuClock {
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr FindWindowExW(IntPtr p, IntPtr a, string c, string t);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] static extern void mouse_event(uint f, int dx, int dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
  // Right-click at x,y, then the ms until a visible popup menu of process pid appears (-1: none).
  public static int RClickToMenu(int x, int y, int pid, int timeoutMs) {
    SetCursorPos(x, y); System.Threading.Thread.Sleep(80);
    mouse_event(8, 0, 0, 0, UIntPtr.Zero); System.Threading.Thread.Sleep(40);
    mouse_event(16, 0, 0, 0, UIntPtr.Zero);
    var sw = Stopwatch.StartNew();
    while (sw.ElapsedMilliseconds < timeoutMs) {
      IntPtr h = IntPtr.Zero;
      while ((h = FindWindowExW(IntPtr.Zero, h, "#32768", null)) != IntPtr.Zero) {
        uint p; GetWindowThreadProcessId(h, out p);
        if (p == pid && IsWindowVisible(h)) return (int)sw.ElapsedMilliseconds;
      }
      System.Threading.Thread.Sleep(1);
    }
    return -1;
  }
}
'@

Case 'setup'
Add-Type -TypeDefinition (Get-Content 'C:\pf\kit\assets\foreground-steal.cs' -Raw) -ReferencedAssemblies System.Windows.Forms, System.Drawing -OutputAssembly "$PfRun\steal.exe" -OutputType WindowsApplication
Check 'foreground-steal.exe compiled' (Test-Path "$PfRun\steal.exe")
[Anim]::Set($true)
Get-ChildItem $PfDesk -Filter 'fxm-*' -Force -ErrorAction SilentlyContinue | Remove-Item -Force
Set-Content "$PfDesk\fxm-a.txt" 'fx menus' -Encoding ASCII
Set-Content "$PfDesk\fxm-b.txt" 'fx menus' -Encoding ASCII
PfStart
Pf fence create --title D --rect '200,300,300,200' | Out-Null
Pf fence create --title E --rect '700,300,300,200' | Out-Null
Pf fence create --title Photos --rect '200,560,300,236' | Out-Null
Pf fence create --title Work --rect '1050,300,300,200' | Out-Null
Start-Sleep -Seconds 2
Pf item move 'fxm-a.txt' 'fxm-b.txt' --to D | Out-Null
Start-Sleep -Seconds 1
$script:Names = @(@(PfJson item list --fence D) | Where-Object { $_ -and $_.name } | ForEach-Object { $_.name } | Sort-Object)
Check 'two items in D' ($script:Names.Count -eq 2) ($script:Names -join ', ')
$script:Ready = New-Object Threading.EventWaitHandle($false, [Threading.EventResetMode]::ManualReset, 'Local\PfStealReady')
$script:Go = New-Object Threading.EventWaitHandle($false, [Threading.EventResetMode]::AutoReset, 'Local\PfStealGo')

function PfMenu() {
  foreach ($m in [W]::Tops('#32768')) { if ([W]::ProcOf($m) -eq 'pecofence') { return $m } }
  return [IntPtr]::Zero
}
function Timed() {
  $a = @($args | ForEach-Object { $_ })
  $sw = [Diagnostics.Stopwatch]::StartNew()
  $null = & "$PfApp\pecofence-cli.exe" @a 2>&1 | Out-String
  return "$([int]$sw.ElapsedMilliseconds) ms exit $LASTEXITCODE"
}
function FenceHwnd($t) { foreach ($h in [W]::Tops('PecoFence.Fence')) { if ([W]::Title($h) -eq $t) { return $h } }; return [IntPtr]::Zero }
function WinH($h) { $r = Rect $h; return $r.B - $r.T }
# Gets PecoFence out of a stuck menu: Esc posted to the menu window, else a click on its first row.
function Unstick() {
  $m = PfMenu
  if ($m -eq [IntPtr]::Zero) { return '' }
  [W]::PostMessageW($m, 0x100, [IntPtr]0x1B, [IntPtr]0x00010001) | Out-Null
  [W]::PostMessageW($m, 0x101, [IntPtr]0x1B, [IntPtr]0xC0010001) | Out-Null
  Start-Sleep -Milliseconds 600
  if ((PfMenu) -ne [IntPtr]::Zero) { $r = Rect $m; [G]::Click($r.L + 40, $r.T + 12); Start-Sleep -Milliseconds 800; Pf fence unroll D | Out-Null; return 'unstuck by a click on its first row' }
  return 'unstuck by a posted Esc'
}
$script:OpenFence = { $r = FenceGet 'D'; $wr = $r.windowRect; [G]::RClick($wr.x + 150, $wr.y + 14) }
$script:OpenItem = { $pt = ItemPt 'D' $script:Names[0]; [G]::RClick($pt[0], $pt[1]) }
$script:OpenTray = {
  $ctl = [G]::FindWindowW('PecoFence.Control', [NullString]::Value)
  [W]::PostMessageW($ctl, 0x8003, [IntPtr](1500 -bor (830 -shl 16)), [IntPtr](0x7B -bor (1 -shl 16))) | Out-Null
}
$script:OpenWork = { $r = Rect (FenceHwnd 'Work'); [G]::RClick($r.L + 150, $r.T + 14) }

# ---- S14
$script:TryNo = 0
# One try: the stealer waits for the signal sent right after $open (the right-click), sleeps
# $delay ms, then takes the foreground ($duration ms long, at least once) and closes $hide ms
# later. Then, as a user who wants the menu gone: Esc, a click on the empty desktop, Esc.
function StealTry($what, $delay, $duration, $hide, [scriptblock]$open) {
  $script:TryNo++
  $script:Ready.Reset() | Out-Null
  $slog = "$PfRun\steal-$($script:TryNo).log"
  $p = Start-Process "$PfRun\steal.exe" -ArgumentList $delay, $duration, $hide, $slog -PassThru
  $null = $script:Ready.WaitOne(10000)
  Start-Sleep -Milliseconds 300
  & $open
  $script:Go.Set() | Out-Null
  $sw = [Diagnostics.Stopwatch]::StartNew()
  $seen = @(); $fgs = @()
  while ($sw.ElapsedMilliseconds -lt 1500) {
    if ((PfMenu) -ne [IntPtr]::Zero) { $seen += [int]$sw.ElapsedMilliseconds }
    $f = [L]::Cls([L]::GetForegroundWindow()); if ($fgs.Count -eq 0 -or $fgs[-1] -ne $f) { $fgs += $f }
    Start-Sleep -Milliseconds 40
  }
  $span = if ($seen.Count) { "$($seen[0])..$($seen[-1]) ms" } else { 'never seen' }
  $p.WaitForExit(5000) | Out-Null
  $fgAfter = Fg
  $openBeforeEsc = (PfMenu) -ne [IntPtr]::Zero
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  $openAfterEsc = (PfMenu) -ne [IntPtr]::Zero
  if ($openBeforeEsc -and $openAfterEsc) { $script:EscIgnored++ }
  [G]::Click(1000, 800); Start-Sleep -Milliseconds 800
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  if ($script:TryNo -eq 1) { Shot 's14-try1-after-esc-and-desktop-click' }
  $still = (PfMenu) -ne [IntPtr]::Zero
  $cli = Timed --timeout 3000 status
  if ($still) { $script:Stuck++; $how = Unstick } else { $how = '' }
  $st = (Get-Content $slog -ErrorAction SilentlyContinue) -join ' / '
  $line = "try $($script:TryNo) [$what delay=$delay]: menu seen $span; foreground $($fgs -join ' > '); after the toast: $fgAfter; open before Esc $openBeforeEsc, after Esc $openAfterEsc; still open after Esc + desktop click + Esc: $still; CLI status $cli; $how; steal: $st"
  Log $line
  $script:Lines += $line
  Start-Sleep -Milliseconds 500
}
function Series($name, $delays, $duration, $hide, [scriptblock]$open) {
  $script:Stuck = 0; $script:EscIgnored = 0; $script:Lines = @()
  foreach ($d in $delays) { StealTry $name $d $duration $hide $open }
  Check "$name : the menu closes with Esc or a click elsewhere" ($script:Stuck -eq 0) "$($script:Stuck) of $(@($delays).Count) stuck; $(($script:Lines | Select-Object -First 2) -join ' || ')"
  Check "$name : a menu still open when Esc is pressed closes with it" ($script:EscIgnored -eq 0) "$($script:EscIgnored) of $(@($delays).Count) ignored Esc"
}

Case 'S14 fence menu: a toast takes the foreground once, right at the right-click'
Series 'fence menu, once' @(0, 2, 5, 8, 10, 15, 20, 30, 45, 60, 90, 150) 0 300 $script:OpenFence
Case 'S14 fence menu: a toast takes the foreground once the menu is open'
Series 'fence menu, menu open' @(400, 400, 400) 0 300 $script:OpenFence
Case 'S14 fence menu: another program keeps taking the foreground for 0.8 s'
Series 'fence menu, keeps taking' @(0, 0, 0) 800 300 $script:OpenFence
Case 'S14 item menu: a toast takes the foreground'
Series 'item menu' @(0, 10, 20, 40, 400) 0 300 $script:OpenItem
Case 'S14 tray menu: a toast takes the foreground'
Series 'tray menu' @(0, 5, 10, 20, 40, 400) 0 300 $script:OpenTray
Case 'S14 menus without a toast still work'
& $script:OpenFence; $m = MenuWin 3; $t = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
Check 'fence menu opens' ($t -ne '') $t
[G]::Press(0x1B); Start-Sleep -Milliseconds 500
Check 'Esc closes it' ((PfMenu) -eq [IntPtr]::Zero)
& $script:OpenTray; $m = MenuWin 3; $t = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
Check 'tray menu opens' ($t -ne '') $t
[G]::Click(1000, 800); Start-Sleep -Milliseconds 600
Check 'a desktop click closes it' ((PfMenu) -eq [IntPtr]::Zero)
& $script:OpenItem; $m = MenuWin 3; $t = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
Check 'item menu opens' ($t -ne '') $t
[G]::Click(1000, 800); Start-Sleep -Milliseconds 600
Check 'a desktop click closes it' ((PfMenu) -eq [IntPtr]::Zero)
& $script:OpenFence; $m = MenuWin 3
$p = MenuPoint $m 'Roll up'; if ($p) { [G]::Click($p[0], $p[1]) }; Start-Sleep -Milliseconds 900
Check 'an item chosen in the fence menu still runs (Roll up)' ((FenceGet 'D').rolledUp -eq $true) "menu point $p"
Pf fence unroll D | Out-Null; Start-Sleep -Milliseconds 600
Log ("menu log lines: " + ((@(PfLogAll | Where-Object { $_ -match 'popup menu' }) | Select-Object -Last 40) -join ' || '))

# ---- P7
$script:Ph = FenceHwnd 'Photos'
Check 'Photos window found' ($script:Ph -ne [IntPtr]::Zero)
# One try: Photos rolled up, the pointer onto its title, and as soon as it starts to grow the
# menu opens; heights sampled every 50 ms for 1.5 s with the menu open, a CLI status sent then.
function ExpandTry($what, [scriptblock]$open) {
  [G]::MoveTo(1100, 830); Start-Sleep -Milliseconds 800
  Pf fence unroll Photos | Out-Null; Start-Sleep -Milliseconds 500
  Pf fence roll Photos | Out-Null; Start-Sleep -Milliseconds 1200
  $rolled = WinH $script:Ph
  $r = Rect $script:Ph
  $logStart = @(PfLogAll).Count
  [G]::MoveTo($r.L + 80, $r.T + 14)
  $grow = 0
  for ($i = 0; $i -lt 400; $i++) { if ((WinH $script:Ph) -gt $rolled + 4) { $grow = $i; break }; Start-Sleep -Milliseconds 5 }
  $atOpen = WinH $script:Ph
  & $open
  $hs = @()
  $sw = [Diagnostics.Stopwatch]::StartNew()
  $cli = Start-Job -ScriptBlock { param($exe, $inst) $env:PECOFENCE_INSTANCE = $inst; $t = [Diagnostics.Stopwatch]::StartNew(); & $exe --timeout 8000 status | Out-Null; $t.ElapsedMilliseconds } -ArgumentList "$PfApp\pecofence-cli.exe", $PfInst
  while ($sw.ElapsedMilliseconds -lt 1500) { $hs += WinH $script:Ph; Start-Sleep -Milliseconds 50 }
  $menuOpen = (PfMenu) -ne [IntPtr]::Zero
  Shot "p7-$($what -replace '\W','_')-menu-open"
  Start-Sleep -Milliseconds 1100
  $stillOpen = (PfMenu) -ne [IntPtr]::Zero
  $menuMs = [int]$sw.ElapsedMilliseconds
  CloseMenus
  $cliMs = Receive-Job $cli -Wait; Remove-Job $cli
  $after = @(); for ($i = 0; $i -lt 10; $i++) { $after += WinH $script:Ph; Start-Sleep -Milliseconds 50 }
  $max = ($hs | Measure-Object -Maximum).Maximum
  $min = ($hs | Measure-Object -Minimum).Minimum
  $runs = @(PfLogAll | Select-Object -Skip $logStart | Where-Object { $_ -match 'animation run|animations ran while|popup menu closed' } | ForEach-Object { ($_ -replace '^\S+\s+\S+\s+\S+\s+', '') })
  $line = "$what : rolled $rolled px, grew after $($grow * 5) ms, $atOpen px when the menu opened; heights with the menu open (50 ms): $($hs -join ','); menu open at 1.5 s $menuOpen and at 3 s $stillOpen; after it closed: $($after -join ','); CLI status sent with the menu open took $cliMs ms (menu open $menuMs ms); log: $($runs -join ' | ')"
  Log $line
  $script:Lines += $line
  $script:Max += $max
  $script:Moved += ($max - $min)
}

Case 'P7 a fence keeps expanding while the tray menu is open'
$script:Lines = @(); $script:Max = @(); $script:Moved = @()
foreach ($k in 1, 2, 3) { ExpandTry "tray menu $k" $script:OpenTray }
$full = @($script:Max | Where-Object { $_ -ge 230 }).Count
Check 'Photos reaches its full height (236) with the tray menu open' ($full -eq 3) "$full of 3; $($script:Lines -join ' || ')"

Case 'P7 a fence keeps animating while another fence menu is open'
$script:Lines = @(); $script:Max = @(); $script:Moved = @()
foreach ($k in 1, 2, 3) { ExpandTry "fence menu $k" $script:OpenWork }
$moving = @($script:Moved | Where-Object { $_ -ge 20 }).Count
Check 'Photos goes on moving (20 px or more) with the Work fence menu open' ($moving -eq 3) "$moving of 3 (moved $($script:Moved -join ', ') px); $($script:Lines -join ' || ')"
[G]::MoveTo(1100, 830); Start-Sleep -Milliseconds 800
Pf fence unroll Photos | Out-Null

# ---- P9
Case 'P9 first item menu after a start (8 s after startup done)'
$first = @(); $second = @(); $logs = @(); $build1 = @(); $build2 = @()
foreach ($k in 1..5) {
  PfStart
  $done = PfWaitLog 'startup done' 60
  Start-Sleep -Seconds 8
  $proc = Get-Process pecofence | Select-Object -First 1
  $a = ItemPt 'D' $script:Names[0]; $b = ItemPt 'D' $script:Names[1]
  $mods0 = $proc.Modules.Count
  $t1 = [MenuClock]::RClickToMenu($a[0], $a[1], $proc.Id, 5000)
  $proc.Refresh(); $mods1 = $proc.Modules.Count
  Start-Sleep -Milliseconds 500; [G]::Press(0x1B); Start-Sleep -Milliseconds 700
  $t2 = [MenuClock]::RClickToMenu($b[0], $b[1], $proc.Id, 5000)
  Start-Sleep -Milliseconds 500; [G]::Press(0x1B); Start-Sleep -Milliseconds 700
  $first += $t1; $second += $t2
  $logs += (@(PfLogAll | Where-Object { $_ -match 'prewarm' }) | Select-Object -Last 1 | ForEach-Object { $_ -replace '^\S+\s+', '' })
  # The build time each menu logs (this start's log only).
  $built = @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match 'item menu built.* ms=(\d+)' } | ForEach-Object { if ($_ -match ' ms=(\d+)') { [int]$Matches[1] } })
  if ($built.Count -ge 2) { $build1 += $built[0]; $build2 += $built[1] }
  Log "start ${k}: first item menu $t1 ms, second $t2 ms; built in $($built -join ', ') ms; modules loaded $mods0 before the first menu, $mods1 with it open"
}
Log "P9 prewarm lines: $($logs -join ' || ')"
$med = ($first | Sort-Object)[2]
$med2 = ($second | Sort-Object)[2]
Check 'first item menu after a start: median within 35 ms of the second' (($med - $med2) -le 35) "first $($first -join ', ') ms (median $med); second $($second -join ', ') ms (median $med2)"
$slow = @(for ($i = 0; $i -lt $build1.Count; $i++) { if ($build1[$i] -gt 2 * $build2[$i] + 10) { $i } }).Count
Check 'the first item menu is built about as fast as the second (handlers loaded in the background)' ($build1.Count -eq 5 -and $slow -eq 0) "first $($build1 -join ', ') ms, second $($build2 -join ', ') ms"

Case 'cleanup'
[G]::MoveTo(1100, 830)
PfQuit
Get-ChildItem $PfDesk -Filter 'fxm-*' -Force -ErrorAction SilentlyContinue | Remove-Item -Force
