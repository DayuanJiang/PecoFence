# Owner round 2026-11-12, package frame-pacing. Every check states the fixed behaviour; on a build
# from before the fix (run.sh --bin <main's CI build>) a FAIL is the finding showing.
#   run.sh --var Part=A   the frame clock: a light animation draws one frame per refresh. The
#                         uneven pace this guards against needs a 60 Hz display with a GPU (1.1
#                         to 1.5 frames per refresh there before the fix); on a box without a GPU
#                         the checks keep the timer's frame between two compositor ticks
#   run.sh --var Part=B   a fence that rolls up, its shadow and the fence that follows it move in
#                         one step; while it opens, the fence below is pushed out from under it.
#                         Nothing is seen apart: no bare strip between the two fences, no shadow
#                         beyond the plate. B reads the windows' rectangles: what the app asks
#                         Windows for. On a box without a GPU the screen itself still shows the
#                         plate's new edge one composed frame before the moved windows, with
#                         either build (Part=R records it); on a PC with a GPU the screen
#                         follows the windows
#   run.sh --var Part=R   recordings of B's roll (no checks)
# No files are put on the desktop: the fences are empty, made and deleted by the steps.
$PfAllowLogErrors = $true
[Anim]::Set($true)
if (-not $Part) { $Part = 'AB' }

Add-Type @'
using System; using System.Collections.Generic; using System.Diagnostics; using System.Runtime.InteropServices; using System.Threading;
public static class OwnerPace {
  [StructLayout(LayoutKind.Sequential)] struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("winmm.dll")] static extern uint timeBeginPeriod(uint ms);
  [DllImport("winmm.dll")] static extern uint timeEndPeriod(uint ms);
  [StructLayout(LayoutKind.Sequential, Pack=1)] struct DWM_TIMING_INFO {
    public uint cbSize; public uint rateRefreshNum, rateRefreshDen; public ulong qpcRefreshPeriod; public uint rateComposeNum, rateComposeDen; public ulong qpcVBlank;
    [MarshalAs(UnmanagedType.ByValArray, SizeConst=256)] public byte[] rest; }
  [DllImport("dwmapi.dll")] static extern int DwmGetCompositionTimingInfo(IntPtr hwnd, ref DWM_TIMING_INFO info);
  // The refresh interval Windows reports for the desktop, in ms (0 when it reports none).
  public static double RefreshMs() {
    var info = new DWM_TIMING_INFO(); info.cbSize = (uint)Marshal.SizeOf(typeof(DWM_TIMING_INFO));
    if (DwmGetCompositionTimingInfo(IntPtr.Zero, ref info) != 0 || info.rateRefreshNum == 0) return 0;
    return 1000.0 * info.rateRefreshDen / info.rateRefreshNum;
  }
  static Thread thread; static volatile bool on; static List<int[]> rows;
  // Reads the three windows about every millisecond and keeps each state they were seen in:
  // top and bottom of the rolling fence, of its shadow and of the fence below, and when it was
  // first seen (tenths of a millisecond). A state is kept only when a second reading 0.3 ms
  // later is the same: a batch that Windows is just applying, or a move that lands between two
  // reads, is not a state the screen had.
  public static void Watch(IntPtr top, IntPtr shadow, IntPtr low) {
    rows = new List<int[]>(); on = true;
    thread = new Thread(() => {
      timeBeginPeriod(1);
      var clock = Stopwatch.StartNew();
      int[] last = null;
      while (on) {
        RECT a, s, b, a2, s2, b2;
        GetWindowRect(top, out a); GetWindowRect(shadow, out s); GetWindowRect(low, out b);
        long until = Stopwatch.GetTimestamp() + Stopwatch.Frequency * 3 / 10000;
        while (Stopwatch.GetTimestamp() < until) { }
        GetWindowRect(top, out a2); GetWindowRect(shadow, out s2); GetWindowRect(low, out b2);
        if (a.T == a2.T && a.B == a2.B && s.T == s2.T && s.B == s2.B && b.T == b2.T && b.B == b2.B) {
          var row = new int[] { a.T, a.B, s.T, s.B, b.T, b.B, (int)(clock.Elapsed.TotalMilliseconds * 10) };
          bool same = last != null;
          for (int i = 0; same && i < 6; i++) same = row[i] == last[i];
          if (!same) { rows.Add(row); last = row; }
        }
        Thread.Sleep(1);
      }
      timeEndPeriod(1);
    });
    thread.IsBackground = true; thread.Priority = ThreadPriority.AboveNormal; thread.Start();
  }
  public static int[][] Stop() { on = false; thread.Join(3000); return rows.ToArray(); }
}
'@

# A rolled-up fence with a second fence 8 px under its title row: opening the first pushes the
# second down, rolling it up lets it come back. Returns both windows and their rects at rest.
function PushPair() {
  # Nothing but the four values may leave this function: what a step prints would join them.
  PfFresh | Out-Null
  ClearFences | Out-Null
  Pf fence create --title '会卷起来的栅栏' --rect '520,80,440,320' | Out-Null
  Start-Sleep -Milliseconds 1500
  Pf fence roll '会卷起来的栅栏' | Out-Null
  Start-Sleep -Milliseconds 900
  $top = [G]::FindWindowW('PecoFence.Fence', '会卷起来的栅栏')
  $tr = Rect $top
  Pf fence create --title '下面被推的' --rect "520,$($tr.B + 8),440,180" | Out-Null
  Start-Sleep -Milliseconds 1500
  $low = [G]::FindWindowW('PecoFence.Fence', '下面被推的')
  return $top, $low, $tr, (Rect $low)
}

if ($Part -match 'A') {
  Case 'A: a light animation draws one frame per refresh'
  PfStart
  ClearFences
  # The pace the frames should have: the display's refresh interval, or half of it (a quarter)
  # for a compositor slower than 60 Hz, which gets a frame from the timer between two ticks.
  $refresh = [OwnerPace]::RefreshMs()
  $pace = if ($refresh -gt 0) { $refresh } else { 1000 / 60 }
  while ($pace -gt 18) { $pace = $pace / 2 }
  # A fence fading in and out is the lightest animation there is (one alpha per frame): its
  # frames were the ones a fixed 16 ms timer started ahead of a 60 Hz display's 16.67 ms tick.
  for ($i = 1; $i -le 8; $i++) {
    Pf fence create --title "淡入淡出的栅栏$i" --rect '240,160,360,240' | Out-Null
    Start-Sleep -Milliseconds 900
    Pf fence delete "淡入淡出的栅栏$i" | Out-Null
    Start-Sleep -Milliseconds 900
  }
  # The app writes its log in blocks: the whole of it is there once the app has ended.
  PfQuit
  $L = @(PfLogAll)
  $runs = @()
  foreach ($line in $L) {
    if ($line -match 'animation run( stuttered)? frames=(\d+) total_ms=([\d.]+) max_gap_ms=([\d.]+) max_frame_ms=([\d.]+) timer_frames=(\d+)') {
      $f = [int]$Matches[2]; $ms = [double]$Matches[3]
      if ($f -ge 5 -and $ms -gt 100) { $runs += [pscustomobject]@{ frames = $f; ms = $ms; timer = [int]$Matches[6]; per = [math]::Round(($f - 1) / ($ms / $pace), 2) } }
    }
  }
  $per = @($runs | ForEach-Object per | Sort-Object)
  $med = if ($per.Count) { $per[[int][math]::Floor($per.Count / 2)] } else { -1 }
  $d = "Windows reports a refresh every $([math]::Round($refresh, 2)) ms, frames are due every $([math]::Round($pace, 2)) ms; $($runs.Count) runs, frames per such interval: $($per -join ' '); frames/ms/by the timer: $((@($runs | ForEach-Object { "$($_.frames)/$([int]$_.ms)/$($_.timer)" })) -join ' ')"
  Log "A $d"
  Check 'A: the steps saw at least 12 animation runs of 100 ms or more' ($runs.Count -ge 12) $d
  Check 'A: the middle run draws one frame per interval (0.95 to 1.05; 1.1 to 1.5 with the 16 ms timer on a 60 Hz display with a GPU)' ($runs.Count -ge 12 -and $med -ge 0.95 -and $med -le 1.05) "median $med; $d"
  Check 'A: no run draws under 0.88 or over 1.15 frames per interval' ($runs.Count -ge 12 -and $per[0] -ge 0.88 -and $per[-1] -le 1.15) "from $($per[0]) to $($per[-1]); $d"
  $ri = @($L | Where-Object { $_ -match 'frame clock: refresh interval' })
  $us = if ($ri.Count -and $ri[-1] -match 'timer_us=(\d+)') { [int]$Matches[1] } else { 0 }
  Check 'A: the frame clock logs the interval it times a missing tick by, and it is the one above' ($us -gt 0 -and [math]::Abs($us / 1000 - $pace) -lt 0.4) "lines: $($ri.Count); last: $(if ($ri.Count) { $ri[-1] -replace '^.*frame clock:', 'frame clock:' })"
}

if ($Part -match 'B') {
  Case 'B: a fence that rolls up and the fence below it move in one step; nothing is seen apart'
  $top, $low, $tr, $lr = PushPair
  # The shadow window of the rolling fence: the smallest shadow window around it.
  $shadow = [IntPtr]::Zero; $best = [int]::MaxValue
  foreach ($h in @([W]::Tops('PecoFence.Shadow'))) {
    $r = Rect $h
    if ($r.L -lt $tr.L -and $r.R -gt $tr.R -and $r.T -lt $tr.T -and $r.B -gt $tr.B -and ($r.B - $r.T) -lt $best) { $shadow = $h; $best = $r.B - $r.T }
  }
  Check 'B: the rolled fence, its shadow window and the fence below it are there' ($top -ne [IntPtr]::Zero -and $low -ne [IntPtr]::Zero -and $shadow -ne [IntPtr]::Zero -and $lr.T -eq $tr.B + 8) "fence $($tr.T)..$($tr.B), below at $($lr.T), shadow window $shadow"
  $gap0 = $lr.T - $tr.B
  $pad0 = (Rect $shadow).B - $tr.B
  # One frame = one height of the rolling fence. At a height the fence below is apart when it
  # was seen there further down than that height puts it: a bare strip of wallpaper between
  # the two (the fence rolled up and the one below had not followed yet). It is under the
  # plate when it was seen further up: the plate has grown and pushes it out a moment later,
  # which is how an opening goes, before and after the fix. The shadow is beyond the plate
  # when its lower edge was further down than that height puts it (the old outline left
  # standing on the wallpaper while the fence rolls up, or a shadow that runs ahead while it
  # opens); it is inside when its lower edge was further up, behind the plate, where it does
  # not show. A new shadow bitmap cannot land in the same instant as the resize, so the app
  # keeps the shadow inside: it shrinks just before the fence does and grows just after.
  $stat = @{ open = @(0, 0, 0, 0); roll = @(0, 0, 0, 0) }
  $ms = @{ low = 0.0; under = 0.0; beyond = 0.0; inside = 0.0 }
  $worstLow = 0; $worstBeyond = 0; $underAt = 0
  for ($n = 1; $n -le 6; $n++) {
    foreach ($act in 'open', 'roll') {
      [OwnerPace]::Watch($top, $shadow, $low)
      Start-Sleep -Milliseconds 100
      if ($act -eq 'open') { Pf fence unroll '会卷起来的栅栏' | Out-Null } else { Pf fence roll '会卷起来的栅栏' | Out-Null }
      Start-Sleep -Milliseconds 900
      $rows = [OwnerPace]::Stop()
      $frames = @{}; $offLow = @{}; $under = @{}; $beyond = @{}; $inside = @{}
      for ($k = 0; $k -lt $rows.Count; $k++) {
        $r = $rows[$k]
        $frames[$r[1]] = 1
        # How long the windows stayed in this state.
        $held = if ($k + 1 -lt $rows.Count) { ($rows[$k + 1][6] - $r[6]) / 10.0 } else { 0 }
        $dl = ($r[4] - $r[1]) - $gap0; $ds = ($r[3] - $r[1]) - $pad0
        if ($dl -gt 0) {
          $offLow[$r[1]] = 1; $ms.low += $held; if ($dl -gt $worstLow) { $worstLow = $dl }
          # The first few go to the job log with the states around them (fence top, bottom,
          # shadow top, bottom, fence below top, bottom, tenths of a ms).
          if ($offLow.Count -le 3) { Log "B apart, $act ${n}: state $k of $($rows.Count): $(($rows[[math]::Max(0, $k - 2)..[math]::Min($rows.Count - 1, $k + 2)] | ForEach-Object { $_ -join ',' }) -join ' | ')" }
        }
        if ($dl -lt 0) { $under[$r[1]] = 1; $ms.under += $held }
        if ($ds -gt 0) { $beyond[$r[1]] = 1; $ms.beyond += $held; if ($ds -gt $worstBeyond) { $worstBeyond = $ds } }
        if ($ds -lt 0) { $inside[$r[1]] = 1; $ms.inside += $held }
      }
      $stat[$act][0] += $frames.Count; $stat[$act][1] += $offLow.Count; $stat[$act][2] += $beyond.Count; $stat[$act][3] += $inside.Count
      $underAt += $under.Count
    }
  }
  $d = "opening: $($stat.open[0]) heights seen, fence below apart at $($stat.open[1]), shadow beyond the plate at $($stat.open[2]), inside at $($stat.open[3]); rolling up: $($stat.roll[0]) heights, fence below apart at $($stat.roll[1]), shadow beyond the plate at $($stat.roll[2]), inside at $($stat.roll[3]); in all: fence below apart for $([math]::Round($ms.low)) ms (up to $worstLow px) and under the plate at $underAt heights for $([math]::Round($ms.under)) ms, shadow beyond the plate for $([math]::Round($ms.beyond)) ms (up to $worstBeyond px), inside for $([math]::Round($ms.inside)) ms"
  Log "B $d"
  $all = $stat.open[0] + $stat.roll[0]
  Check 'B: the steps saw the fence at 60 heights or more over 6 openings and 6 roll-ups' ($all -ge 60) $d
  Check 'B: the fence below is never seen apart from the fence above it (a bare strip between them while it rolled up)' ($all -ge 60 -and ($stat.open[1] + $stat.roll[1]) -eq 0) $d
  Check 'B: the shadow is never seen beyond the plate (its old outline stood on the wallpaper while the fence rolled up)' ($all -ge 60 -and ($stat.open[2] + $stat.roll[2]) -eq 0) $d
  $tr2 = Rect $top; $lr2 = Rect $low
  Check 'B: afterwards both fences rest where they were' ($tr2.T -eq $tr.T -and $tr2.B -eq $tr.B -and $lr2.T -eq $lr.T -and $lr2.B -eq $lr.B) "fence $($tr2.T)..$($tr2.B) (was $($tr.T)..$($tr.B)), below $($lr2.T)..$($lr2.B) (was $($lr.T)..$($lr.B))"
  ClearFences
  PfQuit
}

if ($Part -match 'R') {
  . C:\pf\kit\rec.ps1
  Case 'R: recordings of the roll with a pushed fence'
  $top, $low, $tr, $lr = PushPair
  GreyWallpaper
  Start-Sleep -Milliseconds 800
  foreach ($n in 1, 2, 3) {
    RecStart "push$n" -X ($tr.L - 60) -Y ($tr.T - 50) -W ($tr.R - $tr.L + 120) -H 640 -Fps 60
    Start-Sleep -Milliseconds 400
    RecMark '展开上面的栅栏，下面的栅栏被推下去'
    Pf fence unroll '会卷起来的栅栏' | Out-Null
    Start-Sleep -Milliseconds 900
    RecMark '卷起上面的栅栏，下面的栅栏跟着回来'
    Pf fence roll '会卷起来的栅栏' | Out-Null
    Start-Sleep -Milliseconds 900
    RecMark '停下'
    Start-Sleep -Milliseconds 300
    RecStop
  }
  Check 'R: recorded' $true (Get-Content "$PfShots\rec-push1\stats.txt")
  ClearFences
  PfQuit
}
