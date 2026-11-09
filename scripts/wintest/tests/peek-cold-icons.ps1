# The Peek search list and cold files (needs Everything: smart.ps1). A program's own icon reads
# the file, which on a cold or sleeping drive takes from a tenth of a second to seconds; the list
# must show the type's icon at once and swap the own icon in from a worker, so typing never
# waits for a disk. About 1300 fresh .exe copies (System32 and SysWOW64; robocopy /J leaves them
# out of the file cache; a .dll shows its type's icon and reads nothing), in three groups renamed
# to start with q, j or x and dated now, are the newest hits of the one-letter queries q, j, x.
# A probe thread sends WM_NULL to the control window every few ms and keeps the longest wait of
# each keystroke: q, Backspace, j, Backspace, x (each shows other cold files), then q again
# (warm). EC2's disks answer in a millisecond or two, so a debug build then stands in for a slow
# or sleeping drive: PECOFENCE_TEST_SLOW_READ_MS makes each own icon take 400 ms, and the box
# must still answer at once and swap the icons in later; the same for shortcut targets under
# the Programs tab. English UI, debug build.
$Cold = "C:\pf-cold-$(Get-Date -Format 'MMddHHmmss')"
Add-Type @"
using System; using System.Collections.Generic; using System.Diagnostics; using System.Threading; using System.Runtime.InteropServices;
public static class UiProbe {
  [DllImport("user32.dll")] static extern IntPtr SendMessageTimeoutW(IntPtr h, uint m, IntPtr w, IntPtr l, uint f, uint t, out IntPtr r);
  static Thread th; static volatile bool run; static List<double> waits = new List<double>();
  public static void Start(IntPtr h) {
    lock (waits) waits.Clear(); run = true;
    th = new Thread(() => { var sw = new Stopwatch(); IntPtr r; while (run) { sw.Restart(); SendMessageTimeoutW(h, 0, IntPtr.Zero, IntPtr.Zero, 0, 10000, out r); double ms = sw.Elapsed.TotalMilliseconds; lock (waits) waits.Add(ms); Thread.Sleep(2); } });
    th.IsBackground = true; th.Start();
  }
  public static double[] Stop() { run = false; th.Join(); lock (waits) return waits.ToArray(); }
}
"@
# The longest wait (ms) while $action runs and 2.5 s after it.
function Waited($ctl, [scriptblock]$action) {
  [UiProbe]::Start($ctl); & $action; Start-Sleep -Milliseconds 2500
  $s = @([UiProbe]::Stop() | Sort-Object)
  if ($s.Count -eq 0) { return -1 }
  return [math]::Round($s[-1], 1)
}
# Deletes the cold copies of this and earlier runs (PowerShell 5.1's Remove-Item -Recurse can
# leave empty folders behind; Directory.Delete does not).
function ClearColdCopies() {
  foreach ($d in @(Get-ChildItem -Path 'C:\' -Directory -Filter 'pf-cold-*' -ErrorAction SilentlyContinue)) {
    try { [IO.Directory]::Delete($d.FullName, $true) } catch { Log "cold copies: $($_.Exception.Message)" }
  }
}

Case 'setup'
CloseWindows
# Copies an earlier run left behind (a run that broke off).
ClearColdCopies
Check 'Everything runs' (EvStart) "installed=$(EvInstalled)"
$t = [Diagnostics.Stopwatch]::StartNew()
foreach ($src in 'System32', 'SysWOW64') {
  robocopy "C:\Windows\$src" "$Cold\$src" *.exe /J /COPY:D /R:0 /W:0 /NFL /NDL /NJH /NJS | Out-Null
}
$now = Get-Date
$n = 0
foreach ($f in @(Get-ChildItem -LiteralPath $Cold -File -Recurse)) {
  # Renaming and dating read no file data: the copies stay cold.
  $to = Join-Path $f.DirectoryName "$('qjx'[$n % 3])$($f.Name)"
  try { $f.Attributes = 'Normal'; [IO.File]::Move($f.FullName, $to); [IO.File]::SetLastWriteTime($to, $now.AddSeconds(-($n % 600))); $n++ } catch { }
}
Check 'about 1300 cold program copies named q…, j…, x…' ($n -ge 600) "files=$n in $([int]$t.Elapsed.TotalSeconds) s"
Start-Sleep -Seconds 5
PfStart -Lang en
Pf settings set tools.everything true | Out-Null
Start-Sleep -Seconds 3
$ctl = [G]::FindWindowW('PecoFence.Control', [NullString]::Value)
Check 'the control window is there' ($ctl -ne [IntPtr]::Zero) "$ctl"

Case 'typing over cold files'
DeskClick 1500 820 | Out-Null
PeekKey; Start-Sleep -Milliseconds 800
$coldKeys = @()
foreach ($k in 'q', 'j', 'x') {
  $coldKeys += Waited $ctl { [G]::Type($k) }
  if ($k -eq 'q') { Shot 'c01-cold-q' }
  [G]::Press(0x08); Start-Sleep -Milliseconds 800
}
$warm = Waited $ctl { [G]::Type('q') }
Shot 'c02-warm-q'
Log "longest UI wait per keystroke, cold q / j / x: $($coldKeys -join ' / ') ms; warm q: $warm ms"
$slow = @(PfLogAll | Where-Object { $_ -match 'a stage was slow stage="own_icon"' -and $_ -notmatch 'omitted' })
$swapped = @(PfLogAll | Where-Object { $_ -match "rows' own icons swapped in taken=[1-9]" })
Check 'the rows show Everything''s cold programs' (@(PfLogAll | Where-Object { $_ -match 'search list laid out rows=[1-9]' }).Count -ge 1) ''
Check 'the own icons came from the worker and were swapped in' ($swapped.Count -ge 1) "swapped=$($swapped.Count) slow=$($slow.Count) $(($slow | Select-Object -Last 1) -replace '^.*search: ', '')"
Check 'no keystroke over cold files keeps the UI thread over 100 ms' (($coldKeys | Measure-Object -Maximum).Maximum -lt 100) "cold q / j / x: $($coldKeys -join ' / ') ms; warm q: $warm ms"
for ($i = 0; $i -lt 3; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 400 }

Case 'a slow drive: each own icon takes 400 ms'
PfQuit
$env:PECOFENCE_TEST_SLOW_READ_MS = '400'
PfStart -Lang en
Remove-Item Env:\PECOFENCE_TEST_SLOW_READ_MS
Start-Sleep -Seconds 3
$ctl = [G]::FindWindowW('PecoFence.Control', [NullString]::Value)
DeskClick 1500 820 | Out-Null
PeekKey; Start-Sleep -Milliseconds 800
$n0 = @(PfLogAll | Where-Object { $_ -match "rows' own icons swapped in taken=[1-9]" }).Count
$slowKeys = @()
foreach ($k in 'q', 'j') {
  $slowKeys += Waited $ctl { [G]::Type($k) }
  if ($k -eq 'q') { Shot 'c03-slow-q-at-once' }
  [G]::Press(0x08); Start-Sleep -Milliseconds 800
}
[G]::Type('x'); Start-Sleep -Milliseconds 300
Shot 'c04-slow-x-type-icons'
Start-Sleep -Seconds 4
Shot 'c05-slow-x-own-icons'
$late = @(PfLogAll | Where-Object { $_ -match 'a stage was slow stage="own_icon" ms=(\d+) us=\d+ drive=C:' -and [int]$Matches[1] -ge 400 })
$n1 = @(PfLogAll | Where-Object { $_ -match "rows' own icons swapped in taken=[1-9]" }).Count
Log "a slow drive, longest UI wait per keystroke q / j: $($slowKeys -join ' / ') ms; slow icons logged: $($late.Count)"
Check 'with 400 ms per icon the box still answers each key at once' (($slowKeys | Measure-Object -Maximum).Maximum -lt 100) "q / j: $($slowKeys -join ' / ') ms"
Check 'the slow icons are logged with the time and the drive only, and swapped in later' ($late.Count -ge 3 -and $n1 -gt $n0) "slow=$($late.Count) swapped $n0 -> $n1; $(($late | Select-Object -Last 1) -replace '^.*search: ', '')"
for ($i = 0; $i -lt 3; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 400 }

Case 'the 程序 tab with slow shortcut reads'
# A folder fence of shortcuts, 8 to Notepad (programs) and 8 to win.ini: with every target read
# taking 400 ms off the UI thread, 程序 holds the shortcuts back until their targets are known,
# then shows the 8 programs; typing and the tab keys never wait for the reads.
PfQuit
$Lnk = Join-Path $Cold 'lnk'
New-Item -ItemType Directory -Force $Lnk | Out-Null
$sh = New-Object -ComObject WScript.Shell
foreach ($i in 1..8) {
  $a = $sh.CreateShortcut((Join-Path $Lnk ('pflnk-tool-{0}.lnk' -f $i))); $a.TargetPath = "$env:WINDIR\notepad.exe"; $a.Save()
  $b = $sh.CreateShortcut((Join-Path $Lnk ('pflnk-note-{0}.lnk' -f $i))); $b.TargetPath = "$env:WINDIR\win.ini"; $b.Save()
}
$env:PECOFENCE_TEST_SLOW_READ_MS = '400'
PfStart -Lang en
Remove-Item Env:\PECOFENCE_TEST_SLOW_READ_MS
Pf fence create --portal $Lnk --title 'Shortcuts' --rect '24,40,420,300' | Out-Null
Start-Sleep -Seconds 3
$ctl = [G]::FindWindowW('PecoFence.Control', [NullString]::Value)
DeskClick 1500 820 | Out-Null
PeekKey; Start-Sleep -Milliseconds 600
$keys = @()
$keys += Waited $ctl { [G]::Type('pflnk') }
# Ctrl+Tab seven times: 全部 → … → 程序.
foreach ($i in 1..7) { $keys += Waited $ctl { Chord 0x11 0x09 } }
$prog = $null
for ($i = 0; $i -lt 40; $i++) {
  $prog = @(PfLogAll | Where-Object { $_ -match 'search list laid out' -and $_ -match 'tab="Programs"' -and $_ -notmatch 'omitted' }) | Select-Object -Last 1
  if ("$prog" -match ' total=8 ') { break }
  Start-Sleep -Milliseconds 500
}
Shot 'c06-programs'
$reads = @(PfLogAll | Where-Object { $_ -match 'a stage was slow stage="lnk_target" ms=(\d+) us=\d+ drive=C:' -and [int]$Matches[1] -ge 400 } | Sort-Object -Unique)
Check 'with 400 ms per shortcut target the box answers each key at once' (($keys | Measure-Object -Maximum).Maximum -lt 100) "keys: $($keys -join ' / ') ms"
Check 'Programs shows the 8 shortcuts to Notepad once their targets are read' ("$prog" -match ' total=8 ' -and $reads.Count -ge 8) "reads=$($reads.Count) $(("$prog") -replace '^.*search list laid out ', '')"
for ($i = 0; $i -lt 3; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 400 }
FocusFence 'Shortcuts'; Chord 0x11 0x46; Start-Sleep -Milliseconds 800
[G]::Type('kind:programs '); Start-Sleep -Milliseconds 900
Check 'the filter box''s Programs chip: the 8 programs' (WaitFilterShown 'Shortcuts' 8 10) ((FilterOf 'Shortcuts') | ConvertTo-Json -Compress)
[G]::Press(0x1B); Start-Sleep -Milliseconds 600

Case 'cleanup'
PfQuit
# The folder itself can stay a while after its files go (Defender still scanning the copies
# holds it, delete pending); the next run's setup takes what is left.
ClearColdCopies
$left = @(Get-ChildItem -LiteralPath $Cold -Recurse -File -ErrorAction SilentlyContinue).Count
Check 'the cold copies are deleted' ($left -eq 0) "left=$left files; folder there: $(Test-Path -LiteralPath $Cold)"

