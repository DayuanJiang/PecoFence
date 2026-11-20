# GitHub #241: with Windows' desktop icons hidden by PecoFence (hideRealIcons) and the 「桌面」
# fence arranged like the desktop, PecoFence kept part of a processor core busy on Windows Server
# 2022 (Task Manager 6.5 %); with hideRealIcons off it went to 0.4 %. Each phase measures the
# processor time of the test instance over 15 s, at rest and while the pointer moves over the
# bare desktop (a mouse sends hundreds to thousands of moves a second). About 4 minutes.
$PfAllowLogErrors = $true

Add-Type @'
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
public static class Mover {
  [DllImport("user32.dll")] static extern void mouse_event(uint f, int dx, int dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] static extern int GetSystemMetrics(int i);
  // Moves the pointer around a small circle at (cx, cy) with `perSecond` absolute moves a second.
  public static int Circle(int cx, int cy, int r, int seconds, int perSecond) {
    int w = GetSystemMetrics(0), h = GetSystemMetrics(1), n = 0;
    var sw = Stopwatch.StartNew();
    double step = 1000.0 / perSecond;
    while (sw.ElapsedMilliseconds < seconds * 1000L) {
      double a = n * 0.05;
      int x = cx + (int)(r * Math.Cos(a)), y = cy + (int)(r * Math.Sin(a));
      mouse_event(0x8001, x * 65535 / (w - 1), y * 65535 / (h - 1), 0, UIntPtr.Zero);
      n++;
      while (sw.Elapsed.TotalMilliseconds < n * step) { System.Threading.Thread.SpinWait(50); }
    }
    return n;
  }
}
'@

# Percent of one core over $Seconds, at rest or (with $Move) while the pointer circles at $At.
function CpuOver([int]$Seconds, [switch]$Move, $At) {
  $p = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
  if (-not $p) { return -1 }
  $c0 = $p.TotalProcessorTime.TotalMilliseconds
  if ($Move) { $n = [Mover]::Circle($At[0], $At[1], 30, $Seconds, 1000); Log "pointer moves: $n" }
  else { Start-Sleep -Seconds $Seconds }
  $p.Refresh()
  $c1 = $p.TotalProcessorTime.TotalMilliseconds
  return [math]::Round(($c1 - $c0) / ($Seconds * 10), 1)
}

Case 'CPU with the Windows icons hidden and the Desktop fence arranged like the desktop'
PfStart -LogLevel 'pecofence=info'
Pf settings set themeStyle liquidGlass | Out-Null
Start-Sleep -Seconds 15
$s = PfJson settings get
$desk = @(@(PfJson fence list) | Where-Object { $_ -and $_.desktopLayout })
$spot = FreeSpot
Log "hideRealIcons=$($s.hideRealIcons) desktopLayout fences=$($desk.Count) spot=$($spot -join ',')"
# The desktop in front, as after a click on it (its clicks are followed then on every Windows).
[G]::Click($spot[0], $spot[1]); Start-Sleep -Seconds 2
Log "front: $(Fg)"
$restH = CpuOver 15
$moveH = CpuOver 15 -Move -At $spot
Pf settings set hideRealIcons false | Out-Null
Start-Sleep -Seconds 4
[G]::Click($spot[0], $spot[1]); Start-Sleep -Seconds 2
$restS = CpuOver 15
$moveS = CpuOver 15 -Move -At $spot
Pf settings set hideRealIcons true | Out-Null
Start-Sleep -Seconds 4
[G]::Click($spot[0], $spot[1]); Start-Sleep -Seconds 2
$moveH2 = CpuOver 15 -Move -At $spot
$row = "hidden: rest $restH %, moving $moveH %; shown: rest $restS %, moving $moveS %; hidden again: moving $moveH2 % (of one core)"
Log "CPU $row"
"$row" | Add-Content "$PfRun\cpu.tsv" -Encoding UTF8
Check 'the Desktop fence is arranged like the desktop (else this case tests nothing)' ($desk.Count -ge 1) "desktopLayout fences: $($desk.Count)"
Check 'at rest with the icons hidden PecoFence uses under 2 % of a core' ($restH -ge 0 -and $restH -lt 2) $row
PfQuit
