# The shell menu prewarm: 2 s after a start PecoFence builds the Windows menu of one desktop
# file on a worker thread and drops it, so the first right-click does not wait for the menu
# extensions of the installed programs. It is made for a shortcut (the first in path order). A
# shortcut's menu is built from its target's: these steps put a shortcut first in path order
# that points to a program (the usual case), to a folder, to a file on a share that does not
# answer, and to a file on a drive that is not there, start PecoFence each time and read what
# the prewarm did: its time, the modules it loaded, how long a right-click 3 s after the start
# waited, how long the UI thread took to answer.
# PecoFence leaves such a shortcut alone and takes the next one: the log line
# `shell menu prewarm: the file it is made for` says why (`reason=target is a folder`).
#   run.sh tests/owner8-prewarm.ps1        about 90 seconds, release build
# Before each run, as Administrator (pf cannot write to the public desktop; the steps delete
# the file at the end):
#   box.sh ssh <box> 'New-Item -ItemType File -Force "C:\Users\Public\Desktop\!a.lnk";
#                     icacls "C:\Users\Public\Desktop\!a.lnk" /grant "pf:(M)"'
Add-Type @'
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
public static class MenuClock8 {
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
# Paths are taken in ordinal order: C:\Users\Public\Desktop comes before C:\Users\pf\Desktop, so
# the shortcut that is to be first goes on the public desktop (where Microsoft Edge's is).
$First = '!a.lnk'
$PubDesk = [Environment]::GetFolderPath('CommonDesktopDirectory')
function Link8($name, $target, $dir = $PfDesk) {
  $p = Join-Path $dir $name
  if ($dir -eq $PfDesk) { Remove-Item -LiteralPath $p -Force -ErrorAction SilentlyContinue }
  $s = (New-Object -ComObject WScript.Shell).CreateShortcut($p); $s.TargetPath = $target; $s.Save()
  return $p
}
# This start's log lines that match.
function LogNow($regex) { return @(Get-Content (PfLogPath) -Encoding UTF8 -ErrorAction SilentlyContinue | Where-Object { $_ -match $regex }) }

Case 'setup'
CloseWindows
$made = @()
foreach ($n in @('季度报告.txt', '预算表 2026.txt')) { $made += (DeskFile $n) }
$made += (Link8 'pf8-notepad.lnk' "$env:WINDIR\notepad.exe")
Check "the shortcut that is to be first can be written (see the head of this file)" (Test-Path -LiteralPath (Join-Path $PubDesk $First)) (Join-Path $PubDesk $First)
PfStart
ClearFences | Out-Null
Pf fence create --title D --rect '200,300,300,200' | Out-Null
Start-Sleep -Seconds 2
Pf item move '季度报告.txt' '预算表 2026.txt' --to D | Out-Null
Start-Sleep -Seconds 1
$names = @(@(PfJson item list --fence D) | Where-Object { $_ -and $_.name } | ForEach-Object { $_.name })
Check 'two files in the fence D' ($names.Count -eq 2) ($names -join ', ')
PfQuit

# One start with the shortcut `!a.lnk` -> $target first in path order.
function Start8($case, $target) {
  trap { PfTrap $_; continue }
  $sw = [Diagnostics.Stopwatch]::StartNew()
  $lnk = Link8 $First $target $PubDesk
  $saved = $sw.ElapsedMilliseconds
  PfStart
  $done = PfWaitLog 'startup done' 60
  # The shortcuts PecoFence knows, in the order the prewarm takes them (by path, ordinal).
  $paths = [string[]]@(@(PfJson item list) | Where-Object { $_ -and $_.path -match '\.lnk$' } | ForEach-Object { $_.path })
  [Array]::Sort($paths, [StringComparer]::Ordinal)
  $firstLnk = if ($paths.Count) { Split-Path -Leaf $paths[0] } else { '(none)' }
  $ctl = [G]::FindWindowW('PecoFence.Control', [NullString]::Value)
  $proc = Get-Process pecofence | Select-Object -First 1
  $pt = ItemPt 'D' '季度报告'
  if (-not $pt) { $pt = ItemPt 'D' '季度报告.txt' }
  Start-Sleep -Milliseconds 2600
  # About 3 s after the start: the prewarm began a second ago.
  $click = [MenuClock8]::RClickToMenu($pt[0], $pt[1], $proc.Id, 30000)
  Start-Sleep -Milliseconds 300; [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  CloseMenus
  $ping = [Th]::MaxPing($ctl, 6000)
  $line = PfWaitLog 'shell menu prewarmed|shell menu prewarm failed|shell menu not prewarmed' 120
  Start-Sleep -Milliseconds 500
  $ms = if ("$line" -match ' ms=(\d+)') { [int]$Matches[1] } else { -1 }
  $mods = @(LogNow 'shell extension loaded.*by=Prewarm' | ForEach-Object { if ($_ -match 'module=(\S+)') { $Matches[1] } })
  $allMods = @(LogNow 'shell extension loaded' | ForEach-Object { if ($_ -match 'module=(\S+).*by=(\S+)') { "$($Matches[1])/$($Matches[2])" } })
  $hang = @(LogNow 'UI thread not responding|UI thread stack')
  $choice = @(LogNow 'shell menu prewarm: ') | ForEach-Object { $_ -replace '^\S+\s+\S+\s+\S+\s+', '' }
  $r = [pscustomobject]@{ case = $case; firstLnk = $firstLnk; click = $click; ping = $ping; ms = $ms; line = ("$line" -replace '^\S+\s+', ''); mods = $mods; hang = $hang.Count; choice = ($choice -join ' || ') }
  Log "PREWARM $case target=$target saved_in=${saved}ms first_lnk=$firstLnk right_click_at_3s=${click}ms max_ping=${ping}ms prewarm_ms=$ms hang_lines=$($hang.Count) modules_by_prewarm=[$($mods -join ',')] all=[$($allMods -join ',')]"
  Log "PREWARM $case line: $($r.line)"
  Log "PREWARM $case choice: $($r.choice)"
  PfQuit
  return $r
}

Case 'a shortcut to a program first in path order'
$prog = Start8 'program' "$env:WINDIR\System32\mspaint.exe"
Check 'the usual case: the menu is prewarmed for the shortcut' ($prog.firstLnk -eq $First -and $prog.ms -ge 0 -and $prog.line -match 'shell menu prewarmed') "first=$($prog.firstLnk) $($prog.line); right-click at 3 s $($prog.click) ms, UI answered within $($prog.ping) ms"

Case 'a shortcut to a folder first in path order'
$folder = Start8 'folder' 'C:\Users'
Check 'a folder shortcut is passed over: the log says so, and the menu is prewarmed for another file' ($folder.firstLnk -eq $First -and $folder.choice -match 'folder' -and $folder.line -match 'shell menu prewarmed') "first=$($folder.firstLnk) choice: $($folder.choice) | $($folder.line)"
Check 'folder shortcut first: the right-click at 3 s and the UI thread do not wait' ($folder.click -ge 0 -and $folder.click -le ($prog.click + 300) -and $folder.ping -le 500 -and $folder.hang -eq 0) "right-click $($folder.click) ms (program shortcut: $($prog.click) ms), UI answered within $($folder.ping) ms, hang lines $($folder.hang), prewarm $($folder.ms) ms, modules [$($folder.mods -join ',')]"

Case 'a shortcut to a file on a share that does not answer'
$share = Start8 'share' '\\no-such-host-pf\share\a.exe'
Check 'a shortcut to a network share is passed over: the log says so, and the menu is prewarmed for another file' ($share.firstLnk -eq $First -and $share.choice -match 'network|share' -and $share.line -match 'shell menu prewarmed') "first=$($share.firstLnk) choice: $($share.choice) | $($share.line)"
Check 'share shortcut first: the right-click at 3 s and the UI thread do not wait' ($share.click -ge 0 -and $share.click -le ($prog.click + 300) -and $share.ping -le 500 -and $share.hang -eq 0) "right-click $($share.click) ms (program shortcut: $($prog.click) ms), UI answered within $($share.ping) ms, hang lines $($share.hang), prewarm $($share.ms) ms, modules [$($share.mods -join ',')]"

Case 'a shortcut to a file on a drive that is not there'
$drive = Start8 'drive' 'Q:\none\a.exe'
Check 'a shortcut to a missing drive is passed over: the log says so, and the menu is prewarmed for another file' ($drive.firstLnk -eq $First -and $drive.choice -match 'drive' -and $drive.line -match 'shell menu prewarmed') "first=$($drive.firstLnk) choice: $($drive.choice) | $($drive.line)"
Check 'missing drive shortcut first: the right-click at 3 s and the UI thread do not wait' ($drive.click -ge 0 -and $drive.click -le ($prog.click + 300) -and $drive.ping -le 500 -and $drive.hang -eq 0) "right-click $($drive.click) ms (program shortcut: $($prog.click) ms), UI answered within $($drive.ping) ms, hang lines $($drive.hang), prewarm $($drive.ms) ms, modules [$($drive.mods -join ',')]"
Check 'the odd shortcuts cost the prewarm no more than the program shortcut' (@($folder, $share, $drive | Where-Object { $_.ms -lt 0 -or $_.ms -gt ($prog.ms * 2 + 200) }).Count -eq 0) "program $($prog.ms) ms, folder $($folder.ms) ms, share $($share.ms) ms, drive $($drive.ms) ms"

Case 'cleanup'
PfQuit
Remove-Item -LiteralPath (Join-Path $PubDesk $First) -Force -ErrorAction SilentlyContinue
foreach ($p in $made) { Remove-Item -LiteralPath $p -Force -ErrorAction SilentlyContinue }
Check 'the test files are gone from both desktops' (-not (Test-Path -LiteralPath $made[0]) -and -not (Test-Path -LiteralPath (Join-Path $PubDesk $First))) "$($made.Count) files; public desktop: $((Get-ChildItem $PubDesk -Force | ForEach-Object { $_.Name }) -join ', ')"
