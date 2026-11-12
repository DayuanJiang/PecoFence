# While Windows writes a large file to a slow disk, every other write to that disk waits, however
# small: on the test boxes a 100 byte line for a log waits about 40 s while a 6 GB file is
# copied. PecoFence wrote its configuration and its log on the thread that owns the fences, so
# every fence stopped for as long (found 2026-11-12). Both are written on threads of their own
# since.
# Part A: another program copies a 6 GB file onto the desktop with the shell's copy engine, as
#   a paste in File Explorer does (12 GB free on the box). Until the disk is quiet again a
#   fence answers, follows a drag and the command line can read; afterwards the dragged place
#   and the new file are in config.json and the log has no "UI thread not responding".
# Part B: what a save still has to do: a fence dragged with the mouse is in config.json two
#   seconds later; one dragged right before PecoFence quits is in the file once the process is
#   gone; a command-line change is in the file when the command returns.
# Parts: run.sh --var Part=A (about 2 min) or B (about 1 min); default both.
# --var Rec=before (or after) also records part A's screen for a before/after clip (rec.ps1).
if (-not $Part) { $Part = 'AB' }
trap { PfTrap $_; continue }
if ($Rec) { . C:\pf\kit\rec.ps1 }

# A thread that appends 100 bytes to a file again and again, as a line of a log does: how long
# the disk keeps such a write waiting.
Add-Type @'
using System; using System.IO; using System.Threading; using System.Diagnostics;
public static class BcDisk {
  static volatile bool stop; static long began; public static int MaxMs; public static int Writes;
  public static void Start(string path) {
    stop = false; MaxMs = 0; Writes = 0; Interlocked.Exchange(ref began, 0);
    var t = new Thread(() => {
      var line = new byte[100];
      using (var f = new FileStream(path, FileMode.Create, FileAccess.Write, FileShare.Read)) {
        while (!stop) {
          var sw = Stopwatch.StartNew(); Interlocked.Exchange(ref began, Stopwatch.GetTimestamp());
          try { f.Write(line, 0, line.Length); f.Flush(false); } catch { }
          Interlocked.Exchange(ref began, 0);
          MaxMs = Math.Max(MaxMs, (int)sw.ElapsedMilliseconds); Writes++;
          Thread.Sleep(250);
        }
      }
    });
    t.IsBackground = true; t.Start();
  }
  // How long the write under way has waited so far (0 between writes).
  public static int WaitingMs() {
    long b = Interlocked.Read(ref began);
    return b == 0 ? 0 : (int)((Stopwatch.GetTimestamp() - b) * 1000 / Stopwatch.Frequency);
  }
  public static void Stop() { stop = true; }
}
'@

$BcCfg = "$PfApp\config\config.json"
function BcClean() {
  Remove-Item -LiteralPath "$PfDesk\bc-big.bin" -Force -ErrorAction SilentlyContinue
  Remove-Item -Recurse -Force "$PfRun\bc-src" -ErrorAction SilentlyContinue
}
# The fence's saved place (DIPs) in config.json; $null while the file does not have the fence.
function BcSaved($title) {
  $c = Get-Content -LiteralPath $BcCfg -Raw -Encoding UTF8 | ConvertFrom-Json
  $f = @($c.layouts | ForEach-Object { $_.fences } | Where-Object { $_.title -eq $title }) | Select-Object -First 1
  if ($f) { return $f.geometry }
  return $null
}
function BcWaitFor([scriptblock]$ok, $sec) {
  for ($k = 0; $k -lt $sec * 5; $k++) { if (& $ok) { return $true }; Start-Sleep -Milliseconds 200 }
  return [bool](& $ok)
}
function BcTitleDrag($title, $dx, $dy) {
  $w = (FenceGet $title).windowRect
  DragTo ($w.x + 120) ($w.y + 18) ($w.x + 120 + $dx) ($w.y + 18 + $dy) 10
}

BcClean
PfStart
ClearFences | Out-Null

if ($Part -match 'A') {
  Case 'A a 6 GB file copied onto the desktop'
  New-Item -ItemType Directory -Force "$PfRun\bc-src" | Out-Null
  # No data is written to make the file; the copy writes all of it.
  $f = [IO.File]::Create("$PfRun\bc-src\bc-big.bin"); $f.SetLength([int64]6 * 1GB); $f.Close()
  # A fence to drag while the disk is busy, with room to its right.
  $r = FreeRect 620 300 420 200
  Pf fence create --title 'BcDrag' --rect "$($r[0] + 20),$($r[1] + 40),300,220" | Out-Null
  Start-Sleep -Seconds 2
  $g0 = BcSaved 'BcDrag'
  $w = (FenceGet 'BcDrag').windowRect
  $fw = [W]::Tops('PecoFence.Fence') | Where-Object { $q = Rect $_; $q.L -le ($w.x + 120) -and $q.R -ge ($w.x + 120) -and $q.T -le ($w.y + 18) -and $q.B -ge ($w.y + 18) } | Select-Object -First 1
  [G]::MoveTo($w.x + 420, $w.y + 150)
  if ($Rec) { RecStart "copy-$Rec" -X $r[0] -Y $r[1] -W 620 -H 300 -Fps 10 -InMemory; RecMark '另一个程序开始往桌面复制 6 GB 的文件' }
  [BcDisk]::Start("$PfRun\bc-src\probe.log")
  $cmd = "Add-Type -AssemblyName Microsoft.VisualBasic; [Microsoft.VisualBasic.FileIO.FileSystem]::CopyFile('$PfRun\bc-src\bc-big.bin', '$PfDesk\bc-big.bin', 'AllDialogs')"
  $copy = Start-Process powershell -PassThru -WindowStyle Hidden -ArgumentList '-NoProfile', '-Command', $cmd
  $busy = [Diagnostics.Stopwatch]::StartNew()
  # Until the copy is over and the disk is quiet again: the longest the fence took to answer,
  # and the longest a command-line read took. Once the disk has kept a small write waiting
  # for 5 s, the fence is dragged 200 px to the right. Nothing here writes a file (a line for
  # job.log would wait for the disk as well): the notes are logged afterwards.
  $maxPing = 0; $maxRead = 0; $reads = 0; $quiet = 0; $notes = @(); $drag = $null
  while ($busy.Elapsed.TotalSeconds -lt 180) {
    $waiting = [BcDisk]::WaitingMs()
    if (-not $drag -and $waiting -ge 5000) {
      if ($Rec) { RecMark '硬盘繁忙：一行小写入已经等了 5 秒。拖动栅栏' }
      $q0 = Rect $fw
      DragTo ($q0.L + 120) ($q0.T + 18) ($q0.L + 320) ($q0.T + 18) 10
      $q1 = Rect $fw
      $drag = @{ at = [int]$busy.Elapsed.TotalSeconds; from = $q0.L; to = $q1.L; waited = $waiting; still = [BcDisk]::WaitingMs() }
      if ($Rec) { RecMark '松开鼠标，等硬盘写完' }
    }
    $ping = [Th]::MaxPing($fw, 1000)
    $t = [Diagnostics.Stopwatch]::StartNew()
    $out = & "$PfApp\pecofence-cli.exe" item list 2>&1 | Out-String
    $read = [int]$t.ElapsedMilliseconds
    $maxPing = [Math]::Max($maxPing, $ping); $maxRead = [Math]::Max($maxRead, $read); $reads++
    $notes += "at $([int]$busy.Elapsed.TotalSeconds) s: the fence answered within $ping ms, item list took $read ms ($($out.Length) chars), the small write has waited $waiting ms, copy running: $(-not $copy.HasExited)"
    if ([BcDisk]::WaitingMs() -lt 300 -and $copy.HasExited) { $quiet++ } else { $quiet = 0 }
    if ($quiet -ge 5) { break }
  }
  $busySec = [int]$busy.Elapsed.TotalSeconds
  [BcDisk]::Stop()
  if ($Rec) { RecMark '硬盘写完了'; Start-Sleep -Seconds 3; RecStop }
  $notes | ForEach-Object { Log $_ }
  Shot 'A-after-copy'
  $copied = (Test-Path -LiteralPath "$PfDesk\bc-big.bin") -and $copy.HasExited
  # With a disk that never kept a write waiting the checks below prove nothing.
  Check 'A setup: the file is copied and the disk kept a small write waiting 5 s or longer' ($copied -and $fw -and [BcDisk]::MaxMs -ge 5000 -and $busySec -lt 180) "file on the desktop, copy over: $copied; fence window: $fw; the longest of $([BcDisk]::Writes) small writes took $([BcDisk]::MaxMs) ms; disk quiet again after $busySec s"
  Check 'A the fence answers within 500 ms until the disk is quiet' ($fw -and $maxPing -lt 500) "longest wait for an answer: $maxPing ms in $busySec s"
  Check 'A the command line reads within 3 s until the disk is quiet' ($reads -ge 5 -and $maxRead -lt 3000) "$reads reads, the longest took $maxRead ms"
  Check 'A a fence dragged while the disk is busy follows the pointer' ($drag -and [Math]::Abs(($drag.to - $drag.from) - 200) -le 30 -and $drag.still -ge 3000) "dragged at $($drag.at) s, when the small write had waited $($drag.waited) ms (after the drag: $($drag.still) ms): window left $($drag.from) -> $($drag.to)"
  $saved = BcWaitFor { $g = BcSaved 'BcDrag'; $g -and [Math]::Abs(($g.x - $g0.x) - 200) -le 30 } 60
  $g1 = BcSaved 'BcDrag'
  Start-Sleep -Seconds 2
  $log = @(PfLogAll)
  $hasFile = (Get-Content -LiteralPath $BcCfg -Raw -Encoding UTF8) -match 'bc-big\.bin'
  Check 'A the place it was dragged to is in config.json once the disk is quiet' ($saved -and $hasFile) "x in the file: $($g0.x) before, $($g1.x) after; the copied file is in it: $hasFile"
  $slow = @($log | Where-Object { $_ -match 'config saved; the disk kept the save waiting' })
  Check 'A that save waited for the disk on its own thread' ($slow.Count -ge 1) "$($slow.Count) lines: $($slow | Select-Object -First 1)"
  $hangs = @($log | Where-Object { $_ -match 'UI thread not responding' })
  Check 'A the log has no "UI thread not responding"' ($hangs.Count -eq 0) "$($hangs.Count) lines: $($hangs | Select-Object -First 1)"
  Remove-Item -LiteralPath "$PfDesk\bc-big.bin" -Force -ErrorAction SilentlyContinue
  Pf fence delete 'BcDrag' | Out-Null
}

if ($Part -match 'B') {
  Case 'B a fence dragged with the mouse'
  # Room for the fence and both drags; a neighbour's edge may pull it a few pixels.
  $r = FreeRect 560 340 700 160
  Pf fence create --title 'BcMove' --rect "$($r[0]),$($r[1]),300,220" | Out-Null
  Start-Sleep -Seconds 2
  $g0 = BcSaved 'BcMove'
  BcTitleDrag 'BcMove' 200 0
  $moved = BcWaitFor { $g = BcSaved 'BcMove'; $g -and [Math]::Abs(($g.x - $g0.x) - 200) -le 30 } 2
  $g1 = BcSaved 'BcMove'
  Check 'B a fence dragged with the mouse is in config.json two seconds later' $moved "x in the file: $($g0.x) before, $($g1.x) after a drag of 200 px"

  Case 'B a fence dragged right before the quit'
  $w = (FenceGet 'BcMove').windowRect
  # The button goes up and the quit follows at once: the save a second after a change has
  # not run yet.
  [G]::MoveTo($w.x + 120, $w.y + 18); Start-Sleep -Milliseconds 150
  MDown; Start-Sleep -Milliseconds 200
  foreach ($s in 1..10) { [G]::MoveTo($w.x + 120, $w.y + 18 + $s * 8); Start-Sleep -Milliseconds 40 }
  Start-Sleep -Milliseconds 300
  MUp
  PfQuit
  $g2 = BcSaved 'BcMove'
  $tmp = Test-Path -LiteralPath "$BcCfg.tmp"
  Check 'B the place it was dragged to is in config.json once the process is gone' ($g2 -and [Math]::Abs(($g2.y - $g1.y) - 80) -le 30 -and -not $tmp -and -not (PfAlive)) "y in the file: $($g1.y) before, $($g2.y) after a drag of 80 px; temp file left: $tmp"
  $last = @(Get-Content (PfLogPath) -Encoding UTF8 | Select-Object -Last 3) -join ' / '
  Check 'B the log ends with the line of a normal exit' ($last -match 'process exiting') "last lines: $last"
  PfStart
  $w2 = (FenceGet 'BcMove').windowRect
  Check 'B the next start shows the fence there' ($w2 -and [Math]::Abs($w2.y - ($w.y + 80)) -le 30 -and [Math]::Abs($w2.x - $w.x) -le 30) "window at $($w.x),$($w.y) before the drag, $($w2.x),$($w2.y) after the restart"

  Case 'B a command-line change'
  Pf fence create --title 'BcCli' --rect '300,460,300,200' | Out-Null
  $made = [bool](BcSaved 'BcCli')
  Pf fence move BcCli --x 340 --y 480 | Out-Null
  $g3 = BcSaved 'BcCli'
  Pf fence delete 'BcCli' | Out-Null
  $gone = -not (BcSaved 'BcCli')
  Check 'B a command-line change is in config.json when the command returns' ($made -and $g3 -and $gone) "created: $made; moved: x=$($g3.x) y=$($g3.y); deleted: $gone"
  Pf fence delete 'BcMove' | Out-Null
}

Case 'cleanup'
PfQuit
BcClean
