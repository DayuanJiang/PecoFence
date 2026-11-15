# Feedback #92 (Windows 10 21H1): fences now and then stay above every window and the rest of the
# screen is black. Windows 10 moves the desktop icons between Progman and a WorkerW at each show
# desktop (Win+D); the app logs that as "icon host changed" and ends its show-desktop state.
# Twenty rounds: Win+D with Notepad and Explorer open, then either Win+D again or a switch to
# Notepad (its taskbar button, or Alt+Tab where the job cannot find the button); after each step the z-order (zorder.ps1) says whether a fence is
# still topmost or above the window in front, and a crop of bare desktop says whether it is black.
#   run-hyperv.sh --vm PF-Win10 tests/issues-vm-win10-ontop.ps1      (about 4 minutes)
. C:\pf\kit\zorder.ps1
. C:\pf\kit\pixels.ps1
$PfAllowLogErrors = $true
if (-not $Rounds) { $Rounds = 20 }

function OpenNote($tag, $x, $y, $w, $h) {
  $f = "$PfRun\$tag.txt"; Set-Content $f "window $tag" -Encoding ASCII
  $p = Start-Process notepad.exe -ArgumentList $f -PassThru
  for ($i = 0; $i -lt 50; $i++) { Start-Sleep -Milliseconds 200; $p.Refresh(); if ($p.MainWindowHandle -ne 0) { break } }
  if ($p.MainWindowHandle -ne 0) { [L]::MoveWindow([IntPtr]$p.MainWindowHandle, $x, $y, $w, $h, $true) | Out-Null }
  return [IntPtr]$p.MainWindowHandle
}
# The taskbar button whose name matches, by UI Automation.
function TaskbarPoint($regex) {
  $tray = [Windows.Automation.AutomationElement]::FromHandle([G]::FindWindowW('Shell_TrayWnd', [NullString]::Value))
  $cond = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::Button)
  $names = @()
  foreach ($b in @($tray.FindAll([Windows.Automation.TreeScope]::Descendants, $cond))) {
    $names += $b.Current.Name
    if ($b.Current.Name -match $regex) { $r = $b.Current.BoundingRectangle; return @([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2)) }
  }
  Log "no taskbar button matches '$regex' among: $($names -join ' | ')"
  return $null
}
# The z-order signatures for $ms after now, one per change.
function Sigs($ms = 1500) {
  $out = @(); $last = $null; $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.ElapsedMilliseconds -lt $ms) { $s = [Zs]::Sig(); if ($s -ne $last) { $out += "$($sw.ElapsedMilliseconds) $s"; $last = $s }; Start-Sleep -Milliseconds 10 }
  return $out
}
# From a signature: is a fence topmost (F), and is a fence above an app window that is itself
# above the desktop (the window the user works in, covered by a fence)?
function SigBad($sig) {
  $kinds = @(($sig -split ' fg=')[0] -split ' ' | Where-Object { $_ -and $_ -notmatch '^m\d+$' } | ForEach-Object { $_.Substring(0, 1) })
  $desk = $kinds.Count; for ($k = $kinds.Count - 1; $k -ge 0; $k--) { if ($kinds[$k] -ceq 'P' -or $kinds[$k] -ceq 'W') { $desk = $k } }
  $firstFence = -1; $lastApp = -1
  for ($k = 0; $k -lt $kinds.Count; $k++) {
    if (($kinds[$k] -ceq 'f' -or $kinds[$k] -ceq 'F') -and $firstFence -lt 0) { $firstFence = $k }
    if (($kinds[$k] -ceq 'A' -or $kinds[$k] -ceq 'a') -and $k -lt $desk) { $lastApp = $k }
  }
  return [pscustomobject]@{ over = ($firstFence -ge 0 -and $lastApp -gt $firstFence); top = ($kinds -ccontains 'F') }
}
# Mean colour of a 60x60 crop of bare desktop (no fence, no window there).
function DeskMean($tag) {
  $c = MeanRgb (Crop "desk-$tag" 1450 700 60 60)
  return "$($c[0]),$($c[1]),$($c[2])"
}

Case 'setup'
CloseWindows
foreach ($i in 1..6) { Set-Content "$PfDesk\ot-file-$i.txt" "ot $i" }
PfStart
Pf settings set themeStyle liquidGlass | Out-Null
Pf settings set theme light | Out-Null
Pf fence create --title Alpha --rect '60,60,320,240' | Out-Null
Pf fence create --title Bravo --rect '420,60,320,240' | Out-Null
Pf fence create --title Charlie --rect '780,60,320,240' | Out-Null
Pf item move "$PfDesk\ot-file-1.txt" "$PfDesk\ot-file-2.txt" --to Alpha | Out-Null
Pf item move "$PfDesk\ot-file-3.txt" "$PfDesk\ot-file-4.txt" --to Bravo | Out-Null
Pf item move "$PfDesk\ot-file-5.txt" "$PfDesk\ot-file-6.txt" --to Charlie | Out-Null
Start-Sleep -Seconds 2
$build = [Environment]::OSVersion.Version.Build
Log "windows build $build; settings showDesktop=$((Pf settings get showDesktop).Trim()) hideRealIcons=$((Pf settings get hideRealIcons).Trim())"
Log "desktop before: $([Zs]::Sig()) wallpaper mean $(DeskMean 'start')"
# run-hyperv.sh --elevated: the app runs as administrator, as in the report.
$started = @(PfLogAll | Where-Object { $_ -match 'process started' }) | Select-Object -Last 1
Log "app start: $(if ($started -match 'elevated=\S+') { $Matches[0] } else { '(no line)' })"
ParkMouse

Case 'Win+D then Win+D or a taskbar click, with Notepad and Explorer open'
$note = OpenNote 'OT-note' 200 160 760 520
Start-Process explorer.exe "`"$PfRun`""
Start-Sleep -Seconds 2
foreach ($x in @((New-Object -ComObject Shell.Application).Windows())) { try { [L]::MoveWindow([IntPtr]$x.HWND, 520, 260, 760, 480, $true) | Out-Null } catch {} }
TitleClick $note
Start-Sleep -Milliseconds 800
$bad = @(); $black = @(); $hostLines0 = @(PfLogAll | Where-Object { $_ -match 'icon host changed|show desktop ended' }).Count
for ($k = 1; $k -le $Rounds; $k++) {
  $in = @([Zs]::Run(1600, 150, 0x44, [NullString]::Value, 0))
  $sigIn = $in[-1]
  $mean = DeskMean "in-$k"
  $how = if ($k % 2) { 'win+d' } else { 'taskbar' }
  if ($how -eq 'win+d') {
    $out = @([Zs]::Run(1600, 150, 0x44, [NullString]::Value, 0) | Select-Object -Skip 1)
  } else {
    # A click on Notepad's taskbar button when UI Automation finds it (Windows 11), else
    # Alt+Tab (the job's UI Automation sees no taskbar buttons on Windows 10).
    $p = TaskbarPoint '^Notepad|OT-note'
    if ($p) { [G]::Click($p[0], $p[1]) } else { $how = 'alt+tab'; Chord 0x12 0x09 }
    $out = @(Sigs 1500)
  }
  $end = ($out[-1] -split ' ', 2)[1]
  $b = SigBad $end
  $anyOver = @($out | Where-Object { (SigBad (($_ -split ' ', 2)[1])).over }).Count
  Log "ROUND $k ${how}: in='$sigIn' mean=$mean; out end='$end' over=$($b.over) top=$($b.top) changes=$($out.Count) over_samples=$anyOver"
  if ($b.over -or ($b.top -and $end -notmatch 'fg=(Progman|WorkerW)')) { $bad += "round $k (${how}): $end"; Shot "bad-$k" }
  $c = $mean -split ','; if ([int]$c[0] + [int]$c[1] + [int]$c[2] -lt 30) { $black += "round ${k}: $mean"; Shot "black-$k" }
  if ($k -eq 1 -or $k -eq $Rounds) { Shot "round-$k" }
  # Every round starts with the windows shown and Notepad in front: a desktop still shown
  # (the switch did not bring a window) gets Win+D once more.
  if ([Zs]::Sig() -match 'fg=(Progman|WorkerW)') { Log "round ${k}: the desktop still shows; Win+D to bring the windows back"; Chord 0x5B 0x44; Start-Sleep -Milliseconds 1200 }
  if ([L]::GetForegroundWindow() -ne $note) { TitleClick $note }
  Start-Sleep -Milliseconds 700
}
$hostLines = @(PfLogAll | Where-Object { $_ -match 'icon host changed|show desktop ended' } | Select-Object -Skip $hostLines0)
Log "host lines: $($hostLines.Count); first: $(($hostLines | Select-Object -First 4) -join ' || ')"
Check "$Rounds rounds: no fence stays above the window in front or topmost over it" ($bad.Count -eq 0) "$($bad.Count) bad: $($bad -join ' | ')"
Check "$Rounds rounds: the bare desktop is never black while it shows" ($black.Count -eq 0) "$($black -join ' | ')"
Check 'the log has the Progman / WorkerW moves (Windows 10)' ($hostLines.Count -gt 0) "$($hostLines.Count) lines"

Case 'cleanup'
CloseWindows
PfQuit
foreach ($i in 1..6) { Remove-Item -LiteralPath "$PfDesk\ot-file-$i.txt" -Force -ErrorAction SilentlyContinue }
