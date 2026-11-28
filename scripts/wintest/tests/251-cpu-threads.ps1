# GitHub #251 / #241: where PecoFence's processor time goes on a quiet desktop, per thread (by its
# description; "ui" is the first thread), in the reporter's layout of #251 (three fences at the
# right, 「桌面」 arranged like the desktop, 16 items; --var Part=left also moves the taskbar to the
# left, as there). Phases: at rest with the desktop in front and with another program in front,
# with every fence hidden, with the Windows desktop on, with 100, 1,000 and 10,000 pointer reports
# a second (relative 1 px moves back and forth, another program in front), then Liquid Glass at rest
# and at 1,000 a second. Every row goes to the job log and to cpu.tsv. About 9 minutes.
# The one check: at rest PecoFence uses under 1 % of a core. Manual in suite.txt: it measures, and
# its rows mean something next to the same run on another Windows (Server 2022 with --var Part=left,
# Server 2025). --var Part=pointer runs only the resting-pointer phases at the end of this file.
$PfAllowLogErrors = $true

Add-Type @'
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
public static class Thr {
  [DllImport("kernel32.dll")] static extern IntPtr OpenThread(uint access, bool inherit, uint id);
  [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
  [DllImport("kernel32.dll")] static extern int GetThreadDescription(IntPtr h, out IntPtr desc);
  [DllImport("kernel32.dll")] static extern IntPtr LocalFree(IntPtr p);
  [DllImport("user32.dll")] static extern void mouse_event(uint f, int dx, int dy, uint d, UIntPtr e);
  public static string Name(int id) {
    IntPtr h = OpenThread(0x0800, false, (uint)id);
    if (h == IntPtr.Zero) return "";
    IntPtr p; string s = "";
    if (GetThreadDescription(h, out p) >= 0 && p != IntPtr.Zero) { s = Marshal.PtrToStringUni(p); LocalFree(p); }
    CloseHandle(h);
    return s;
  }
  [DllImport("user32.dll")] static extern int GetSystemMetrics(int i);
  // `perSecond` absolute reports of the same point (x, y), for `seconds`: a pointer that keeps
  // reporting while it rests, as a virtual machine's pointer can.
  public static int Still(int x, int y, int seconds, int perSecond) {
    int w = GetSystemMetrics(0), h = GetSystemMetrics(1), nx = x * 65535 / (w - 1), ny = y * 65535 / (h - 1);
    var sw = Stopwatch.StartNew();
    double step = 1000.0 / perSecond;
    int n = 0;
    while (sw.ElapsedMilliseconds < seconds * 1000L) {
      mouse_event(0x8001, nx, ny, 0, UIntPtr.Zero);
      n++;
      while (sw.Elapsed.TotalMilliseconds < n * step) { System.Threading.Thread.SpinWait(20); }
    }
    return n;
  }
  // `perSecond` relative pointer moves of 1 px, back and forth, for `seconds`.
  public static int Jiggle(int seconds, int perSecond) {
    var sw = Stopwatch.StartNew();
    double step = 1000.0 / perSecond;
    int n = 0;
    while (sw.ElapsedMilliseconds < seconds * 1000L) {
      mouse_event(1, (n & 1) == 0 ? 1 : -1, 0, 0, UIntPtr.Zero);
      n++;
      while (sw.Elapsed.TotalMilliseconds < n * step) { System.Threading.Thread.SpinWait(20); }
    }
    return n;
  }
}
'@

# Processor milliseconds of each thread of the test instance: id -> @(ms, start time).
function ThreadSnap() {
  $p = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
  $h = @{}
  if (-not $p) { return $h }
  foreach ($t in $p.Threads) {
    try { $h[$t.Id] = @($t.TotalProcessorTime.TotalMilliseconds, $t.StartTime) } catch { }
  }
  return $h
}

# Runs $Load (or sleeps $Seconds) and logs the share of one core of the whole process and of each
# group of threads with the same description.
function Phase([string]$Label, [int]$Seconds, [scriptblock]$Load) {
  $p = Get-Process pecofence | Select-Object -First 1
  $a = ThreadSnap
  $c0 = $p.TotalProcessorTime.TotalMilliseconds
  $sw = [Diagnostics.Stopwatch]::StartNew()
  $extra = ''
  if ($Load) { $extra = & $Load } else { Start-Sleep -Seconds $Seconds }
  $ms = $sw.Elapsed.TotalMilliseconds
  $p.Refresh()
  $c1 = $p.TotalProcessorTime.TotalMilliseconds
  $b = ThreadSnap
  $first = ($b.GetEnumerator() | Sort-Object { $_.Value[1] } | Select-Object -First 1).Key
  $groups = @{}
  foreach ($id in $b.Keys) {
    $was = if ($a.ContainsKey($id)) { $a[$id][0] } else { 0 }
    $name = if ($id -eq $first) { 'ui' } else { [Thr]::Name($id) }
    if (-not $name) { $name = 'unnamed' }
    $groups[$name] = $groups[$name] + ($b[$id][0] - $was)
  }
  $parts = $groups.GetEnumerator() | Where-Object { $_.Value -gt 0 } | Sort-Object Value -Descending |
    Select-Object -First 5 | ForEach-Object { "$($_.Key) $([math]::Round($_.Value * 100 / $ms, 2)) %" }
  $total = [math]::Round(($c1 - $c0) * 100 / $ms, 2)
  $row = "$Label`t$total %`t$($parts -join ', ')`t$extra"
  Log "CPU $row"
  $row | Add-Content "$PfRun\cpu.tsv" -Encoding UTF8
  return $total
}

# The taskbar's side (byte 12 of StuckRects3: 0 left, 3 bottom); Explorer restarts to take it.
function TaskbarSide([int]$Side) {
  $k = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\StuckRects3'
  $v = (Get-ItemProperty $k).Settings
  if ($v[12] -eq $Side) { return }
  $v[12] = [byte]$Side
  Set-ItemProperty $k Settings $v
  $me = (Get-Process -Id $PID).SessionId
  Get-Process explorer -ErrorAction SilentlyContinue | Where-Object { $_.SessionId -eq $me } | Stop-Process -Force
  for ($i = 0; $i -lt 30 -and -not (Get-Process explorer -ErrorAction SilentlyContinue | Where-Object { $_.SessionId -eq $me }); $i++) { Start-Sleep -Milliseconds 500 }
  if (-not (Get-Process explorer -ErrorAction SilentlyContinue | Where-Object { $_.SessionId -eq $me })) { Start-Process explorer.exe }
  Start-Sleep -Seconds 8
  Log "taskbar side $Side; work area $([Windows.Forms.Screen]::PrimaryScreen.WorkingArea)"
}

if ($Part -notmatch 'pointer') {
Case 'processor time per thread in the reporter layout of #251'
if ($Part -match 'left') { TaskbarSide 0 }
$desk = [Environment]::GetFolderPath('Desktop')
$made = @()
for ($i = 1; $i -le 16; $i++) { $f = "$desk\cpu251 $i.txt"; [IO.File]::WriteAllText($f, "item $i"); $made += $f }
PfStart -LogLevel 'pecofence=info'
ClearFences | Out-Null
$wa = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea
$b = [Windows.Forms.Screen]::PrimaryScreen.Bounds
Log "screen $($b.Width)x$($b.Height) work $($wa.X),$($wa.Y),$($wa.Width)x$($wa.Height) dpi $([G]::GetSystemMetrics(88))"
$w = [math]::Min(480, [int]($wa.Width / 4)); $x = $wa.Right - $w - 16
$h1 = [int]($wa.Height * 0.5); $h2 = [int]($wa.Height * 0.18)
Pf fence create --title RightA --rect "$x,$($wa.Y + 16),$w,$h1" | Out-Null
Pf fence create --title RightB --rect "$x,$($wa.Y + $h1 + 32),$w,$h2" | Out-Null
Pf fence create --title RightC --rect "$x,$($wa.Y + $h1 + $h2 + 48),$w,$h2" | Out-Null
$inbox = @(@(PfJson fence list) | Where-Object { $_.kind -eq 'inbox' })[0]
Pf fence set $inbox.id desktopLayout true | Out-Null
Start-Sleep -Seconds 10
$fl = @(PfJson fence list)
Log ("fences: " + (($fl | ForEach-Object { "$($_.title)[$($_.windowRect.x),$($_.windowRect.y) $($_.windowRect.w)x$($_.windowRect.h) desk=$($_.desktopLayout) items=$($_.itemCount)]" }) -join ' '))
$spot = FreeSpot
[G]::Click($spot[0], $spot[1]); Start-Sleep -Seconds 2
ParkMouse
Log "front: $(Fg)"
$rest = Phase 'rest, desktop in front' 60
$np = Start-Process notepad -PassThru
Start-Sleep -Seconds 3
Log "front: $(Fg)"
Phase 'rest, another program in front' 60 | Out-Null
Pf fence hide-all | Out-Null
Start-Sleep -Seconds 3
Phase 'every fence hidden' 60 | Out-Null
Pf fence show-all | Out-Null
Pf windows-desktop start | Out-Null
Start-Sleep -Seconds 3
Phase 'Windows desktop on' 60 | Out-Null
Pf windows-desktop end | Out-Null
Start-Sleep -Seconds 3
foreach ($rate in 100, 1000, 10000) {
  Phase "pointer $rate reports/s, another program in front" 0 { $n = [Thr]::Jiggle(30, $rate); "moves $n" } | Out-Null
}
Pf settings set themeStyle liquidGlass | Out-Null
Start-Sleep -Seconds 8
Phase 'Liquid Glass, rest' 60 | Out-Null
Phase 'Liquid Glass, pointer 1000 reports/s' 0 { $n = [Thr]::Jiggle(30, 1000); "moves $n" } | Out-Null
Stop-Process -Id $np.Id -ErrorAction SilentlyContinue
Check 'at rest PecoFence uses under 1 % of a core' ($rest -ge 0 -and $rest -lt 1) "$rest %"
PfQuit
$made | ForEach-Object { Remove-Item $_ -ErrorAction SilentlyContinue }
if ($Part -match 'left') { TaskbarSide 3 }
}

# --var Part=pointer: where a resting pointer's reports cost. 1,000 reports a second of the same
# point over the bare desktop (with Windows' icons hidden that is the window of 「桌面」 arranged
# like the desktop), the same with Windows' icons shown, over another program's window, and
# 1,000 one-pixel moves a second over the bare desktop.
if ($Part -match 'pointer') {
Case 'a resting pointer that keeps reporting, over the desktop and over another window'
$desk = [Environment]::GetFolderPath('Desktop')
$made = @()
for ($i = 1; $i -le 16; $i++) { $f = "$desk\cpu251 $i.txt"; [IO.File]::WriteAllText($f, "item $i"); $made += $f }
PfStart -LogLevel 'pecofence=info'
ClearFences | Out-Null
$inbox = @(@(PfJson fence list) | Where-Object { $_.kind -eq 'inbox' })[0]
Pf fence set $inbox.id desktopLayout true | Out-Null
Start-Sleep -Seconds 8
$spot = FreeSpot 1000 600
[G]::Click($spot[0], $spot[1]); Start-Sleep -Seconds 2
[G]::MoveTo($spot[0], $spot[1]); Start-Sleep -Seconds 1
Log "front: $(Fg); under the pointer: $([W]::RootClassAt($spot[0], $spot[1])) at $($spot -join ',')"
$restP = Phase 'rest, pointer still over the bare desktop' 30
Phase 'icons hidden: 1000 reports/s of one point over the bare desktop' 0 { $n = [Thr]::Still($spot[0], $spot[1], 30, 1000); "reports $n" } | Out-Null
Phase 'icons hidden: 1000 one-pixel moves/s over the bare desktop' 0 { [G]::MoveTo($spot[0], $spot[1]); $n = [Thr]::Jiggle(30, 1000); "moves $n" } | Out-Null
$np = Start-Process notepad -PassThru
Start-Sleep -Seconds 3
$nr = New-Object G+RECT
[G]::GetWindowRect((Get-Process -Id $np.Id).MainWindowHandle, [ref]$nr) | Out-Null
$ox = [int](($nr.L + $nr.R) / 2); $oy = [int](($nr.T + $nr.B) / 2)
Log "front: $(Fg); under the pointer: $([W]::RootClassAt($ox, $oy)) at $ox,$oy"
Phase 'icons hidden: 1000 reports/s of one point over another program' 0 { $n = [Thr]::Still($ox, $oy, 30, 1000); "reports $n" } | Out-Null
Stop-Process -Id $np.Id -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2
Pf settings set hideRealIcons false | Out-Null
Start-Sleep -Seconds 4
[G]::Click($spot[0], $spot[1]); Start-Sleep -Seconds 2
Log "front: $(Fg); under the pointer: $([W]::RootClassAt($spot[0], $spot[1]))"
Phase 'icons shown: 1000 reports/s of one point over the bare desktop' 0 { $n = [Thr]::Still($spot[0], $spot[1], 30, 1000); "reports $n" } | Out-Null
Phase 'icons shown: 1000 one-pixel moves/s over the bare desktop' 0 { [G]::MoveTo($spot[0], $spot[1]); $n = [Thr]::Jiggle(30, 1000); "moves $n" } | Out-Null
Pf settings set hideRealIcons true | Out-Null
Check 'at rest PecoFence uses under 1 % of a core' ($restP -ge 0 -and $restP -lt 1) "$restP %"
PfQuit
$made | ForEach-Object { Remove-Item $_ -ErrorAction SilentlyContinue }
}
