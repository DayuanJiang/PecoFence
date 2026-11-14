# Show desktop (Win+D), then a program's window that shows without taking the foreground.
# Explorer leaves such a window above the raised desktop; the fences, floating in the topmost
# band, lay over it until it took the foreground. With a black window the screen looks like a
# wallpaper gone black with only the icons and the taskbar left, which is what #187 reports
# after opening a file with WPS Office (WPS itself was not tested: whether its window shows
# this way is an assumption from the report).
# The stand-in here is a black window over the whole work area, shown with SW_SHOWNOACTIVATE by
# another process; it never takes the foreground.
. C:\pf\kit\zorder.ps1

# The stand-in window's process: black, over the work area, shown without activation, always on
# top with -Top 1; it closes when nofocus.stop appears (or after 60 s).
Set-Content "$PfRun\nofocus.ps1" -Encoding ASCII -Value @'
param([int]$Top = 0)
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type 'using System; using System.Runtime.InteropServices; public static class NF { [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int c); [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int w, int hh, uint f); }'
$f = New-Object Windows.Forms.Form
$f.Text = 'PFNoFocus'
$f.StartPosition = 'Manual'
$f.Bounds = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea
$f.BackColor = [Drawing.Color]::Black
$f.Add_FormClosed({ [Windows.Forms.Application]::ExitThread() })
$t = New-Object Windows.Forms.Timer
$t.Interval = 200
$n = 0
$t.Add_Tick({ $script:n++; if ((Test-Path C:\pf\run\nofocus.stop) -or $script:n -gt 300) { $t.Stop(); $f.Close() } })
$null = $f.Handle
[void][NF]::ShowWindow($f.Handle, 4)
# Always on top without activation: HWND_TOPMOST with SWP_NOMOVE, SWP_NOSIZE, SWP_NOACTIVATE.
if ($Top) { [void][NF]::SetWindowPos($f.Handle, [IntPtr](-1), 0, 0, 0, 0, 0x13) }
$t.Start()
[Windows.Forms.Application]::Run()
'@
function NoFocusWin([int]$Top) {
  Remove-Item "$PfRun\nofocus.stop" -ErrorAction SilentlyContinue
  Start-Process powershell.exe -WindowStyle Hidden -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-WindowStyle', 'Hidden', '-File', "$PfRun\nofocus.ps1", '-Top', $Top
  for ($i = 0; $i -lt 100; $i++) {
    $h = [G]::FindWindowW([NullString]::Value, 'PFNoFocus')
    if ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) { return $h }
    Start-Sleep -Milliseconds 100
  }
  return [IntPtr]::Zero
}
function NoFocusGone() {
  Set-Content "$PfRun\nofocus.stop" 1
  for ($i = 0; $i -lt 50 -and [G]::FindWindowW([NullString]::Value, 'PFNoFocus') -ne [IntPtr]::Zero; $i++) { Start-Sleep -Milliseconds 100 }
}
# The z-order signature ([Zs]::Sig: top first, F a floating fence, f a fence at the desktop,
# A the stand-in window, a the same always on top, P / W the desktop) once it matches, else the
# last one after $sec seconds.
function SigWait($regex, $sec = 2) {
  $sw = [Diagnostics.Stopwatch]::StartNew()
  do { $s = [Zs]::Sig(); if ($s -cmatch $regex) { return $s }; Start-Sleep -Milliseconds 50 } while ($sw.Elapsed.TotalSeconds -lt $sec)
  return $s
}

Case 'show desktop, then a window without the foreground'
PfStart
Start-Sleep -Seconds 4
Start-Process notepad.exe
Start-Sleep -Seconds 2
Chord 0x5B 0x44
$sig = SigWait '^F' 3
Check 'show desktop: the fences float over the raised desktop' ($sig -cmatch '^F\d* ') $sig
$h = NoFocusWin 0
if ($h -ne [IntPtr]::Zero) { [void][Zs]::Apps.Add([G]::Cls($h)) }
Check 'the stand-in window shows and the desktop keeps the foreground' ($h -ne [IntPtr]::Zero -and (Fg) -match '^(Progman|WorkerW|PecoFence)') "window=$h fg=$(Fg)"
$sig = SigWait '^A f' 2
Check 'the fences go under that window within 2 s and stay above the desktop' ($sig -cmatch '^A f\d* ((W|w|m) )*P') $sig
Shot 'window-without-foreground'
NoFocusGone
$sig = SigWait '^F' 3
Check 'that window gone, the fences float over the raised desktop again' ($sig -cmatch '^F\d* ') $sig

Case 'show desktop with an always-on-top window'
$h = NoFocusWin 1
if ($h -ne [IntPtr]::Zero) { [void][Zs]::Apps.Add([G]::Cls($h)) }
Start-Sleep -Milliseconds 1500
$sig = [Zs]::Sig()
Check 'an always-on-top window that does not take the foreground leaves the fences floating' ($h -ne [IntPtr]::Zero -and $sig -cmatch '(^| )a ' -and $sig -cmatch '(^| )F' -and $sig -cnotmatch '(^| )f\d* ' -and $sig -cmatch 'fg=(Progman|WorkerW)') $sig
NoFocusGone
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
