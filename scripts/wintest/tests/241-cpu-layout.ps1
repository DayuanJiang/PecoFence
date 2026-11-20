# GitHub #241, the reporter's layout (Windows Server 2022, 200 %): three rolled fences along the
# top edge (hover to open), a folder fence and a fence with tabs at the right, the 「桌面」 fence
# arranged like the desktop, Liquid Glass (--var Part=left: the taskbar on the left, as there). Measures the processor time of the test instance at
# rest with the icons hidden and shown. Run it in a 200 % session (box.sh up with WINTEST_SCALE=200).
$PfAllowLogErrors = $true

function CpuOver([int]$Seconds) {
  $p = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
  if (-not $p) { return -1 }
  $c0 = $p.TotalProcessorTime.TotalMilliseconds
  Start-Sleep -Seconds $Seconds
  $p.Refresh()
  return [math]::Round(($p.TotalProcessorTime.TotalMilliseconds - $c0) / ($Seconds * 10), 1)
}

# The taskbar's side (byte 12 of StuckRects3: 0 left, 3 bottom); Explorer restarts to take it.
function TaskbarSide([int]$Side) {
  $k = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\StuckRects3'
  $v = (Get-ItemProperty $k).Settings
  if ($v[12] -eq $Side) { return }
  $v[12] = [byte]$Side
  Set-ItemProperty $k Settings $v
  Get-Process explorer -ErrorAction SilentlyContinue | Where-Object { $_.SessionId -eq (Get-Process -Id $PID).SessionId } | Stop-Process -Force
  for ($i = 0; $i -lt 30 -and -not (Get-Process explorer -ErrorAction SilentlyContinue | Where-Object { $_.SessionId -eq (Get-Process -Id $PID).SessionId }); $i++) { Start-Sleep -Milliseconds 500 }
  if (-not (Get-Process explorer -ErrorAction SilentlyContinue | Where-Object { $_.SessionId -eq (Get-Process -Id $PID).SessionId })) { Start-Process explorer.exe }
  Start-Sleep -Seconds 8
  Log "taskbar side $Side; work area $([Windows.Forms.Screen]::PrimaryScreen.WorkingArea)"
}

Case 'the reporter layout at rest'
if ($Part -match 'left') { TaskbarSide 0 }
$dir = 'C:\pf\cpu-sql'
New-Item -ItemType Directory -Force $dir | Out-Null
for ($i = 1; $i -le 9; $i++) { [IO.File]::WriteAllText("$dir\q$i.sql", "select $i") }
PfStart -LogLevel 'pecofence=debug'
Pf settings set themeStyle liquidGlass | Out-Null
$wa = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea
$b = [Windows.Forms.Screen]::PrimaryScreen.Bounds
Log "screen $($b.Width)x$($b.Height) work $($wa.X),$($wa.Y),$($wa.Width)x$($wa.Height)"
ClearFences
$x0 = $wa.X
Pf fence create --title TopA --rect "$x0,0,948,282" | Out-Null
Pf fence create --title TopB --rect "$($x0 + 964),0,480,282" | Out-Null
Pf fence create --title TopC --rect "$($x0 + 1460),0,480,282" | Out-Null
$rx = $wa.Right - 560
Pf fence create --portal $dir --rect "$rx,0,560,682" | Out-Null
Pf fence create --title TabOne --rect "$rx,698,560,480" | Out-Null
Pf fence create --title TabTwo --rect "$rx,1200,560,480" | Out-Null
Pf fence merge TabTwo --into TabOne | Out-Null
Start-Sleep -Seconds 2
Pf fence roll TopA | Out-Null
Pf fence roll TopB | Out-Null
Pf fence roll TopC | Out-Null
Start-Sleep -Seconds 12
$fl = @(PfJson fence list)
Log ("fences: " + (($fl | ForEach-Object { "$($_.title)[$($_.windowRect.x),$($_.windowRect.y) $($_.windowRect.w)x$($_.windowRect.h) rolled=$($_.rolledUp) desk=$($_.desktopLayout)]" }) -join ' '))
ParkMouse
$n0 = @(PfLogAll).Count
$h1 = CpuOver 20
$L = @(PfLogAll | Select-Object -Skip $n0)
$top = $L | ForEach-Object { if ($_ -match '^\S+\s+\w+\s+(\S+?):\s(.{0,50})') { "$($Matches[1]): $($Matches[2])" } } | Group-Object | Sort-Object Count -Descending | Select-Object -First 6 | ForEach-Object { "$($_.Count)x $($_.Name)" }
Log "hidden: $h1 % of a core; lines during it: $($L.Count): $($top -join ' | ')"
Pf settings set hideRealIcons false | Out-Null
Start-Sleep -Seconds 5
ParkMouse
$n1 = @(PfLogAll).Count
$s1 = CpuOver 20
$L = @(PfLogAll | Select-Object -Skip $n1)
Log "shown: $s1 % of a core; lines during it: $($L.Count)"
Pf settings set hideRealIcons true | Out-Null
Start-Sleep -Seconds 5
ParkMouse
$h2 = CpuOver 20
$row = "hidden $h1 %, shown $s1 %, hidden again $h2 % of a core"
Log "CPU $row"
Check 'with the icons hidden the reporter layout rests under 2 % of a core' ($h1 -ge 0 -and $h1 -lt 2 -and $h2 -lt 2) $row
Shot 'layout'
PfQuit
Remove-Item -Recurse -Force $dir -ErrorAction SilentlyContinue
if ($Part -match 'left') { TaskbarSide 3 }
