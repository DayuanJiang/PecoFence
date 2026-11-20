# GitHub #241: when PecoFence keeps the processor busy, its log says what did it (the threads,
# what the UI thread handled most, the frames drawn). The steps keep it busy for a minute with
# fast pointer moves over a fence's icons (a mouse that reports thousands of moves a second)
# and look for the line. On main there is no such line. About 3 minutes.
$PfAllowLogErrors = $true

Add-Type @'
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
public static class BusyMover {
  [DllImport("user32.dll")] static extern void mouse_event(uint f, int dx, int dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] static extern int GetSystemMetrics(int i);
  public static int Zigzag(int x0, int y0, int w, int h, int seconds, int perSecond) {
    int sw0 = GetSystemMetrics(0), sh0 = GetSystemMetrics(1), n = 0;
    var sw = Stopwatch.StartNew();
    double step = 1000.0 / perSecond;
    while (sw.ElapsedMilliseconds < seconds * 1000L) {
      int x = x0 + (n * 7) % w, y = y0 + (n * 3) % h;
      mouse_event(0x8001, x * 65535 / (sw0 - 1), y * 65535 / (sh0 - 1), 0, UIntPtr.Zero);
      n++;
      while (sw.Elapsed.TotalMilliseconds < n * step) { System.Threading.Thread.SpinWait(20); }
    }
    return n;
  }
}
'@

Case 'a busy minute leaves a line that says what kept PecoFence busy'
$dir = 'C:\pf\busy-icons'
New-Item -ItemType Directory -Force $dir | Out-Null
for ($i = 1; $i -le 30; $i++) { [IO.File]::WriteAllText("$dir\file $i.txt", "busy $i") }
PfStart
Pf fence create --portal $dir --rect '200,100,700,600' | Out-Null
Start-Sleep -Seconds 4
$f = @(@(PfJson fence list) | Where-Object { $_.title -eq 'busy-icons' })[0]
$r = $f.windowRect
Log "fence $($r.x),$($r.y) $($r.w)x$($r.h)"
# The first minute check only takes the start; the load begins after it (about 60 s after start).
$p = Get-Process pecofence | Select-Object -First 1
# --var Part=shown: Windows' icons shown, 1000 moves a second over the bare desktop instead
# (a look at what moves over Windows' own desktop cost PecoFence).
if ($Part -eq 'shown') {
  Pf settings set hideRealIcons false | Out-Null
  Start-Sleep -Seconds 4
  $spot = FreeSpot
  [G]::Click($spot[0], $spot[1])
  Log "front: $(Fg) spot $($spot -join ',')"
}
# --var Part=other: another program in front, 1000 moves a second over the bare desktop (what
# following the desktop's clicks all the time costs on Windows 10 and Server 2022).
if ($Part -eq 'other') {
  $spot = FreeSpot
  $np = Start-Process notepad -PassThru
  Start-Sleep -Seconds 2
  Log "front: $(Fg) spot $($spot -join ',')"
}
Start-Sleep -Seconds 58
$c0 = $p.TotalProcessorTime.TotalMilliseconds
if ($Part -eq 'other') { $n = [BusyMover]::Zigzag($spot[0] - 30, $spot[1] - 30, 60, 60, 70, 1000); Log "front after: $(Fg)"; Stop-Process -Id $np.Id -ErrorAction SilentlyContinue }
# --var Part=still: a pointer that keeps reporting while it rests (a remote or virtual machine's
# pointer): 1000 reports a second of the same point, over an icon of the fence.
elseif ($Part -eq 'still') { $n = [BusyMover]::Zigzag($r.x + 80, $r.y + 100, 1, 1, 70, 1000) }
elseif ($Part -eq 'shown') { $n = [BusyMover]::Zigzag($spot[0] - 30, $spot[1] - 30, 60, 60, 70, 1000) }
else { $n = [BusyMover]::Zigzag($r.x + 40, $r.y + 60, $r.w - 80, $r.h - 120, 70, 8000) }
$p.Refresh()
$pct = [math]::Round(($p.TotalProcessorTime.TotalMilliseconds - $c0) / 700, 1)
Log "pointer moves: $n over 70 s; PecoFence used $pct % of a core"
ParkMouse
$line = PfWaitLog 'busy: PecoFence used the processor' 70
Log "line: $line"
Check 'the load kept PecoFence above 10 % of a core (else this case tests nothing)' ($pct -ge 10) "$pct %"
Check 'a busy line names the threads and what the UI thread handled' ($line -match 'busy: PecoFence used the processor' -and $line -match 'threads=' -and $line -match 'ui_thread_handled=') "$line"
PfQuit
Remove-Item -Recurse -Force $dir -ErrorAction SilentlyContinue
