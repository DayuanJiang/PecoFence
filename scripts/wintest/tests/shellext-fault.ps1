# Another program's shell extension that faults inside PecoFence (GitHub #167, #178). The test
# extension (tools/shellext-fault, built by build.sh into target/wintest-shellext) is a menu
# handler for files and folders and a copy hook for folders. It writes every call to calls.log
# beside the DLL and faults in the entry point named in fault.txt, only inside pecofence.exe.
# The steps register it for pf and remove it again.
#   run.sh --extra target/wintest-shellext scripts/wintest/tests/shellext-fault.ps1
#   D  Delete calls no menu extension: with the extension set to fault in its first call,
#      Delete, Shift+Delete, the item menu's Delete, several files at once, a folder fence and
#      a folder still delete, and Ctrl+Z brings a deleted file back
#   C  Ctrl+C and Ctrl+X call no menu extension: with the extension set to fault in its first
#      call they put the files on the clipboard as Explorer does (copy 5, cut 2), a cut file
#      pastes into a folder fence and a copy pastes into an Explorer window
#   F  a fault while the item menu is built (the shell loads every menu extension for it): the
#      crash line and the next start's notice name the DLL, the process is gone within half a
#      second of that line, the watchdog starts PecoFence again within a second of the click
#      without Windows' own icons showing in between, and the layout is as before
#   M  after that crash the item menus leave out Windows' part: the notice says so, the same
#      right-click shows PecoFence's entries with the extension still set to fault, among
#      them a Cut and a Copy of its own that work, the next start builds no menu in the
#      background, and the tray menu's entry brings Windows' part back
#   A  an extension that faults whenever its menu is built: that first happens two seconds
#      after a start, in the menu built in the background; the run is started again with the
#      Windows part of the item menus off, and a right-click then loads nothing. Before that:
#      a fault while the DLL is loaded ends nothing, Windows only leaves the extension out
#   H  a menu extension that never returns: the frozen run says why, ends itself after about
#      35 s and is started again; that start leaves the Windows part of the menus out
#   S  a 45 s wait in PecoFence's own code (the test script's `stall`, a debug build:
#      run.sh --bin target/wintest-bin-debug --var Part=S) does not end the run
#   O  a crash in PecoFence's own code (the test script's `crash` line) is left to Windows
#      Error Reporting as before: Windows records it, the process ends when that is done
#   L  the log says once per run which operation loaded the extension, and the log a user
#      sends with a report keeps the DLL's name
# Parts: run.sh --var Part=D ... (default: all). About 4 minutes.
$PfAllowLogErrors = $true
if (-not $Part) { $Part = 'LDCFMAHO' }
trap { PfTrap $_; continue }

Add-Type @'
using System; using System.Text; using System.Threading; using System.Diagnostics; using System.Runtime.InteropServices;
public static class SxMon {
  delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  [DllImport("kernel32.dll")] static extern IntPtr OpenProcess(uint access, bool inherit, uint id);
  [DllImport("kernel32.dll")] static extern uint WaitForSingleObject(IntPtr h, uint ms);
  [DllImport("kernel32.dll")] static extern bool GetExitCodeProcess(IntPtr h, out uint code);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr FindWindowW(string cls, string title);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr FindWindowExW(IntPtr parent, IntPtr after, string cls, string title);
  // Windows' own desktop icons on screen (Explorer's list under SHELLDLL_DefView).
  static bool WindowsIcons() {
    IntPtr view = FindWindowExW(FindWindowW("Progman", null), IntPtr.Zero, "SHELLDLL_DefView", null);
    if (view == IntPtr.Zero) EnumWindows((h, l) => { var d = FindWindowExW(h, IntPtr.Zero, "SHELLDLL_DefView", null); if (d != IntPtr.Zero) { view = d; return false; } return true; }, IntPtr.Zero);
    IntPtr list = FindWindowExW(view, IntPtr.Zero, "SysListView32", null);
    return list != IntPtr.Zero && IsWindowVisible(list);
  }
  static int VisibleFences(uint pid) {
    int n = 0;
    EnumWindows((h, l) => {
      uint p; GetWindowThreadProcessId(h, out p);
      if (p == pid && IsWindowVisible(h)) { var s = new StringBuilder(64); GetClassNameW(h, s, 64); if (s.ToString() == "PecoFence.Fence") n++; }
      return true;
    }, IntPtr.Zero);
    return n;
  }
  // Watches a PecoFence process until a new one shows fences, `seconds` at most. Milliseconds
  // since the call: exit (and the exit code), born = a new pecofence.exe after that exit (while
  // Windows Error Reporting works, a copy of the crashed process runs under the same name),
  // fences = the new one shows a fence, icons = Windows' own desktop icons were on screen.
  // settle > 0: stop that many ms after the exit when no new PecoFence has come.
  public static string Watch(int pid, int seconds) { return Watch(pid, seconds, 0); }
  public static string Watch(int pid, int seconds, int settle) {
    var sw = Stopwatch.StartNew();
    IntPtr hp = OpenProcess(0x00100000 | 0x1000, false, (uint)pid);
    long exited = -1, born = -1, shown = -1, icons = -1; uint code = 0; int newPid = 0;
    while (sw.ElapsedMilliseconds < seconds * 1000L && shown < 0) {
      long t = sw.ElapsedMilliseconds;
      if (exited < 0 && (hp == IntPtr.Zero || WaitForSingleObject(hp, 0) == 0)) { exited = t; GetExitCodeProcess(hp, out code); }
      if (exited >= 0 && newPid == 0) foreach (var p in Process.GetProcessesByName("pecofence")) if (p.Id != pid) { newPid = p.Id; born = t; }
      if (newPid != 0 && VisibleFences((uint)newPid) > 0) shown = t;
      if (icons < 0 && WindowsIcons()) icons = t;
      if (settle > 0 && exited >= 0 && newPid == 0 && t - exited > settle) break;
      Thread.Sleep(20);
    }
    return string.Format("exit={0} code={1:X8} new_pid={2} born={3} fences={4} icons={5}", exited, code, newPid, born, shown, icons);
  }
}
'@

$SxDir = 'C:\pf\sx-folder'
$SxExplorer = 'C:\pf\sx-explorer'
$ExtDir = "$PfRun\ext"
$ExtClsid = '{7C0F1E5A-51B6-4E0C-9C55-70F5C0F3A167}'
$ExtParents = @('HKCU\Software\Classes\*', 'HKCU\Software\Classes\Directory')
$ExtKeys = @(
  'HKCU\Software\Classes\*\shellex\ContextMenuHandlers\PfFaultTest',
  'HKCU\Software\Classes\Directory\shellex\ContextMenuHandlers\PfFaultTest',
  'HKCU\Software\Classes\Directory\shellex\CopyHookHandlers\PfFaultTest')
$Stamp = "$env:LOCALAPPDATA\PecoFence\icons-hidden.$PfInst.restarted"

function SxClean() {
  Get-ChildItem -LiteralPath $PfDesk -Force -ErrorAction SilentlyContinue | Where-Object { $_.Name -like 'sx *' } | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
  if (Test-Path $SxDir) { Remove-Item -LiteralPath $SxDir -Recurse -Force -ErrorAction SilentlyContinue }
  if (Test-Path $SxExplorer) { Remove-Item -LiteralPath $SxExplorer -Recurse -Force -ErrorAction SilentlyContinue }
  Clear-RecycleBin -Force -ErrorAction SilentlyContinue
}
function RegHas($key) { & reg.exe query $key 2>$null | Out-Null; return ($LASTEXITCODE -eq 0) }
# The extension for pf only (HKCU): a menu handler for files and folders, a copy hook for folders.
function ExtRegister() {
  New-Item -ItemType Directory -Force $ExtDir | Out-Null
  Copy-Item "$PfExtra\pf_shellext_fault.dll" $ExtDir -Force
  $script:ExtHad = @{}
  foreach ($k in $ExtParents) { $script:ExtHad[$k] = (RegHas $k) }
  & reg.exe add "HKCU\Software\Classes\CLSID\$ExtClsid" /ve /d 'PecoFence fault test' /f | Out-Null
  & reg.exe add "HKCU\Software\Classes\CLSID\$ExtClsid\InprocServer32" /ve /d "$ExtDir\pf_shellext_fault.dll" /f | Out-Null
  & reg.exe add "HKCU\Software\Classes\CLSID\$ExtClsid\InprocServer32" /v ThreadingModel /d Apartment /f | Out-Null
  foreach ($k in $ExtKeys) { & reg.exe add $k /ve /d $ExtClsid /f | Out-Null }
}
# Its keys go again, and the parent keys ExtRegister had to make (they hold nothing else).
function ExtUnregister() {
  foreach ($k in $ExtKeys) { & reg.exe delete $k /f 2>$null | Out-Null }
  & reg.exe delete "HKCU\Software\Classes\CLSID\$ExtClsid" /f 2>$null | Out-Null
  foreach ($k in $ExtParents) { if ($script:ExtHad -and -not $script:ExtHad[$k]) { & reg.exe delete $k /f 2>$null | Out-Null } }
  return @(@($ExtKeys) + @("HKCU\Software\Classes\CLSID\$ExtClsid") | Where-Object { RegHas $_ }).Count
}
# From ExtArm on, that entry point faults inside PecoFence; the calls so far are forgotten.
function ExtArm($entry) { Remove-Item "$ExtDir\calls.log" -ErrorAction SilentlyContinue; Set-Content "$ExtDir\fault.txt" $entry -Encoding ASCII }
function ExtDisarm() { Remove-Item "$ExtDir\fault.txt" -ErrorAction SilentlyContinue }
# The entry points PecoFence called since ExtArm, each once.
function ExtCalls() {
  if (-not (Test-Path "$ExtDir\calls.log")) { return @() }
  return @(Get-Content "$ExtDir\calls.log" | Where-Object { $_ -match 'host=pecofence\.exe' } | ForEach-Object { if ($_ -match 'tid=\d+ (\S+)') { $Matches[1] } } | Sort-Object -Unique)
}
function SxFence($title, $rect, $names) {
  foreach ($n in $names) { DeskFile $n | Out-Null }
  Start-Sleep -Seconds 2
  Pf fence create --title $title --rect $rect | Out-Null
  Pf item move @($names | ForEach-Object { Join-Path $PfDesk $_ }) --to $title | Out-Null
  Pf fence set $title sort name | Out-Null
  Start-Sleep -Seconds 2
}
function SxSelect($fence, $i) { $p = CellPt $fence $i; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600; ParkMouse; Start-Sleep -Milliseconds 200 }
# PecoFence as a user runs it (no test script): the watchdog starts it again after a crash.
function SxPlain() {
  $env:PECOFENCE_INSTANCE = $PfInst
  $env:RUST_LOG = 'pecofence=debug'
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $PfApp
  $null = WaitWin 'PecoFence.Control' 20 -Hidden
}
# The last line of the running instance's log that matches.
function SxNewLog($regex) { return (@(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match $regex }) | Select-Object -Last 1) }
# While this file exists, item menus leave out Windows' part.
$SxMarker = "$PfApp\config\windows-menu-off.marker"
# Windows' question of a delete (a dialog of the PecoFence process), zero when none shows.
function SxQuestion($sec = 6) {
  for ($i = 0; $i -lt $sec * 5; $i++) {
    $d = @([W]::Tops('#32770') | Where-Object { [W]::ProcOf($_) -eq 'pecofence' }) | Select-Object -First 1
    if ($d) { return $d }
    Start-Sleep -Milliseconds 200
  }
  return [IntPtr]::Zero
}
# One row after a delete: the files are gone, PecoFence is the same process and logged no crash.
function SxGone($what, $names, $dir = $PfDesk) {
  $left = @()
  for ($i = 0; $i -lt 100; $i++) {
    $left = @($names | Where-Object { Test-Path -LiteralPath (Join-Path $dir $_) })
    if ($left.Count -eq 0) { break }
    Start-Sleep -Milliseconds 200
  }
  Start-Sleep -Milliseconds 1000
  $p = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
  $same = [bool]($p -and $p.Id -eq $script:SxPid)
  $crash = @(PfLogAll | Where-Object { $_ -cmatch 'CRASH \(|panicked' }).Count
  Check "D ${what}: the files are gone and PecoFence still runs" ($left.Count -eq 0 -and $same -and $crash -eq 0) "left $($left.Count) of $(@($names).Count); same process=$same; crash lines=$crash; extension calls: $((ExtCalls) -join ', ')"
}

Case 'setup'
CloseWindows
SxClean
Remove-Item $Stamp -ErrorAction SilentlyContinue
$dll = Test-Path "$PfExtra\pf_shellext_fault.dll"
Check 'the test extension is on the box' $dll 'C:\pf\extra\pf_shellext_fault.dll: build.sh makes it, run.sh brings it with --extra target/wintest-shellext'
if ($dll) { ExtRegister }

if ($dll -and $Part -match 'L') {
  Case 'L the log names the extension'
  DeskFile 'sx l1 文件.txt' | Out-Null
  PfStart
  Pf settings set featureTips false | Out-Null
  ClearFences | Out-Null
  # The background build of the item menu, two seconds after the start, loads it first;
  # without that build the right-click below does.
  $null = PfWaitLog 'shell menu (pre|not pre)warmed' 20
  SxFence 'SxL' '300,200,520,170' @('sx l2 报告.txt', 'sx l3 预算.txt')
  $p = CellPt 'SxL' 0
  [G]::RClick($p[0], $p[1])
  $m = MenuWin 8
  CloseMenus
  [G]::RClick($p[0], $p[1])
  $m = MenuWin 8
  CloseMenus
  Start-Sleep -Milliseconds 500
  $lines = @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match 'shell extension loaded' -and $_ -match 'pf_shellext_fault' })
  Check 'L the log says once which operation loaded the extension' ($lines.Count -eq 1 -and $lines[0] -match ' by="?(Prewarm|ItemMenu)"?') "lines: $($lines.Count); $(if ($lines.Count) { if ($lines[0] -match '(shell extension loaded.*)$') { $Matches[1] } })"
  $shared = @(Get-Content (ShareLog) -Encoding UTF8 | Where-Object { $_ -match 'shell extension loaded' })
  Check 'L the log sent with a report keeps the DLL''s name' ($shared.Count -eq 1 -and $shared[0] -match 'module=pf_shellext_fault\.dll by=(Prewarm|ItemMenu)') "shareable log: $(if ($shared.Count) { if ($shared[0] -match '(shell extension loaded.*)$') { $Matches[1] } })"
  PfQuit
  SxClean
}

if ($dll -and $Part -match 'D') {
  Case 'D Delete with a menu extension that faults'
  DeskFile 'sx a0.txt' | Out-Null
  PfStart
  Pf settings set featureTips false | Out-Null
  ClearFences | Out-Null
  # The first item menu is built in the background two seconds after the start: only after
  # that does the extension fault.
  $warm = PfWaitLog 'shell menu (pre|not pre)warmed' 20
  $dn = @('sx d1 报告.txt', 'sx d2 预算.txt', 'sx d3 合同.txt', 'sx d4 清单.txt', 'sx d5 发票.txt', 'sx d6 备忘.txt')
  SxFence 'SxD' '300,200,560,170' $dn
  $script:SxPid = (Get-Process pecofence | Select-Object -First 1).Id
  $menuCalls = (ExtCalls) -join ', '
  ExtArm 'Initialize'
  Log "armed; before: $menuCalls; $warm"

  SxSelect 'SxD' 0
  [G]::Press(0x2E)
  SxGone 'Delete key' @($dn[0])
  Check 'D Delete key: the file is in the Recycle Bin' ((BinNames) -match 'sx d1') "bin: $(BinNames)"
  # The fence still has the keyboard: Ctrl+Z undoes the delete.
  Chord 0x11 0x5A; Start-Sleep -Milliseconds 2500
  Check 'D Ctrl+Z brings the deleted file back' (Test-Path -LiteralPath (Join-Path $PfDesk $dn[0])) "there=$(Test-Path -LiteralPath (Join-Path $PfDesk $dn[0])); bin: $(BinNames)"

  SxSelect 'SxD' 1
  Chord 0x10 0x2E
  $q = SxQuestion
  if ($q -ne [IntPtr]::Zero) { [G]::Press(0x59) }
  SxGone 'Shift+Delete, answered Yes' @($dn[1])
  Check 'D Shift+Delete: Windows asked first, and the file is not in the Recycle Bin' ($q -ne [IntPtr]::Zero -and (BinNames) -notmatch 'sx d2') "question=$q; bin: $(BinNames)"

  # The item menu loads the extension (it must not fault there), its Delete does not.
  ExtDisarm
  $p = CellPt 'SxD' 1
  [G]::RClick($p[0], $p[1])
  $m = MenuWin 8
  $texts = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
  $menuCalls = (ExtCalls) -join ', '
  ExtArm 'Initialize'
  MenuPick '^Delete' $m | Out-Null
  SxGone "the item menu's Delete" @($dn[2])
  Check 'D the item menu itself shows the extension''s entry' ($texts -match 'PF fault test' -and $menuCalls -match 'QueryContextMenu') "extension calls for the menu: $menuCalls; menu: $texts"

  # Three files at once, for good: one question for all of them.
  SxSelect 'SxD' 1
  Chord 0x11 0x41; Start-Sleep -Milliseconds 500
  $left0 = @($dn | Where-Object { Test-Path -LiteralPath (Join-Path $PfDesk $_) })
  Chord 0x10 0x2E
  $q = SxQuestion
  $asked = if ($q -ne [IntPtr]::Zero) { ([W]::Statics($q) -replace '\s+', ' ') } else { '' }
  if ($q -ne [IntPtr]::Zero) { [G]::Press(0x59) }
  SxGone "Ctrl+A and Shift+Delete on $($left0.Count) files" $left0
  Check 'D several files: Windows asks once, for all of them' ($asked -match "$($left0.Count) items" -and (SxQuestion 1) -eq [IntPtr]::Zero) "question: $asked"

  ClearFences | Out-Null
  New-Item -ItemType Directory -Force $SxDir | Out-Null
  $pn = @('sx p1 合同.txt', 'sx p2 报价.txt')
  foreach ($n in $pn) { Set-Content -LiteralPath (Join-Path $SxDir $n) -Value 'x' -Encoding ASCII }
  Pf fence create --portal $SxDir --title 'SxP' --rect '300,200,520,170' | Out-Null
  Start-Sleep -Seconds 3
  Pf fence set SxP sort name | Out-Null
  Start-Sleep -Seconds 1
  SxSelect 'SxP' 0
  [G]::Press(0x2E)
  SxGone 'Delete in a folder fence' @($pn[0]) $SxDir

  # A folder: Windows asks its copy hooks (the extension is one), no menu extension.
  $folder = Join-Path $PfDesk 'sx f1 资料夹'
  New-Item -ItemType Directory -Force $folder | Out-Null
  Set-Content -LiteralPath (Join-Path $folder 'in.txt') -Value 'x' -Encoding ASCII
  Start-Sleep -Seconds 2
  Pf fence create --title 'SxF' --rect '300,440,300,150' | Out-Null
  Pf item move $folder --to SxF | Out-Null
  Start-Sleep -Seconds 2
  SxSelect 'SxF' 0
  [G]::Press(0x2E)
  SxGone 'Delete of a folder' @('sx f1 资料夹')

  $calls = ExtCalls
  Check 'D no Delete called the menu extension' (@($calls | Where-Object { $_ -match 'Initialize|QueryContextMenu|InvokeCommand' }).Count -eq 0) "calls from PecoFence since the extension was set to fault: $($calls -join ', ')"
  ExtDisarm
  PfQuit
  SxClean
}

if ($dll -and $Part -match 'C') {
  Case 'C copy and cut with a menu extension that faults'
  DeskFile 'sx a0.txt' | Out-Null
  PfStart
  Pf settings set featureTips false | Out-Null
  ClearFences | Out-Null
  $null = PfWaitLog 'shell menu (pre|not pre)warmed' 20
  $cn = @('sx c1 文档.txt', 'sx c2 表格.txt', 'sx c3 图片.txt')
  SxFence 'SxC' '300,200,520,170' $cn
  New-Item -ItemType Directory -Force $SxDir | Out-Null
  Pf fence create --portal $SxDir --title 'SxV' --rect '300,440,520,170' | Out-Null
  Start-Sleep -Seconds 3
  $script:SxPid = (Get-Process pecofence | Select-Object -First 1).Id
  ExtArm 'Initialize'
  [Windows.Forms.Clipboard]::Clear()
  SxSelect 'SxC' 0
  Chord 0x11 0x43; Start-Sleep -Milliseconds 800
  $c = @(ClipFiles | ForEach-Object { Split-Path $_ -Leaf }); $e = ClipEffect
  Check 'C Ctrl+C puts the file on the clipboard as a copy' ($c.Count -eq 1 -and $c[0] -eq $cn[0] -and $e -eq 5 -and (PfAlive)) "clipboard: $($c -join ' | ') effect=$e; extension calls: $((ExtCalls) -join ', ')"
  Chord 0x11 0x41; Start-Sleep -Milliseconds 500
  Chord 0x11 0x43; Start-Sleep -Milliseconds 800
  $c = @(ClipFiles)
  Check 'C Ctrl+C on three files copies the three' ($c.Count -eq 3 -and (ClipEffect) -eq 5) "clipboard: $(($c | ForEach-Object { Split-Path $_ -Leaf }) -join ' | ') effect=$(ClipEffect)"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300
  SxSelect 'SxC' 1
  Chord 0x11 0x58; Start-Sleep -Milliseconds 800
  $c = @(ClipFiles | ForEach-Object { Split-Path $_ -Leaf }); $e = ClipEffect
  Check 'C Ctrl+X puts the file on the clipboard as a cut' ($c.Count -eq 1 -and $c[0] -eq $cn[1] -and $e -eq 2) "clipboard: $($c -join ' | ') effect=$e"
  # The cut file pastes into the folder fence: it moves into that folder.
  $v = (FenceGet 'SxV').windowRect
  [G]::Click([int]($v.x + $v.w / 2), [int]($v.y + $v.h - 30)); Start-Sleep -Milliseconds 500
  Chord 0x11 0x56
  $moved = $false
  for ($i = 0; $i -lt 40 -and -not $moved; $i++) { Start-Sleep -Milliseconds 200; $moved = (Test-Path -LiteralPath (Join-Path $SxDir $cn[1])) -and -not (Test-Path -LiteralPath (Join-Path $PfDesk $cn[1])) }
  Check 'C the cut file pastes into a folder fence: it moves there' $moved "in the folder=$(Test-Path -LiteralPath (Join-Path $SxDir $cn[1])); still on the desktop=$(Test-Path -LiteralPath (Join-Path $PfDesk $cn[1]))"
  # A copy made in a fence pastes into an Explorer window.
  New-Item -ItemType Directory -Force $SxExplorer | Out-Null
  SxSelect 'SxC' 0
  Chord 0x11 0x43; Start-Sleep -Milliseconds 800
  $copied = @(ClipFiles | ForEach-Object { Split-Path $_ -Leaf }) -join ' | '
  $had = @([W]::Tops('CabinetWClass'))
  Start-Process explorer.exe $SxExplorer
  $fw = $null
  for ($k = 0; $k -lt 30 -and -not $fw; $k++) { Start-Sleep -Milliseconds 300; $fw = @([W]::Tops('CabinetWClass') | Where-Object { $had -notcontains $_ }) | Select-Object -First 1 }
  if ($fw) { [L]::MoveWindow($fw, 880, 420, 640, 400, $true) | Out-Null }
  Start-Sleep -Seconds 2
  # The empty file list of that window, then Ctrl+V.
  [G]::Click(1320, 700); Start-Sleep -Milliseconds 500
  Chord 0x11 0x56
  $pasted = $false
  for ($i = 0; $i -lt 40 -and -not $pasted; $i++) { Start-Sleep -Milliseconds 200; $pasted = Test-Path -LiteralPath (Join-Path $SxExplorer $copied) }
  Check 'C a copy made in a fence pastes into an Explorer window' ($fw -and $copied -and $pasted -and (Test-Path -LiteralPath (Join-Path $PfDesk $copied))) "copied: $copied; in the Explorer folder=$pasted; still on the desktop=$(Test-Path -LiteralPath (Join-Path $PfDesk $copied))"
  CloseWindows
  $p = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
  $calls = ExtCalls
  $crash = @(PfLogAll | Where-Object { $_ -cmatch 'CRASH \(|panicked' }).Count
  Check 'C no copy, cut or paste called the menu extension, and PecoFence still runs' ($calls.Count -eq 0 -and $p -and $p.Id -eq $script:SxPid -and $crash -eq 0) "calls from PecoFence since the extension was set to fault: $($calls -join ', '); same process=$($p -and $p.Id -eq $script:SxPid); crash lines=$crash"
  ExtDisarm
  [Windows.Forms.Clipboard]::Clear()
  PfQuit
  SxClean
}

if ($dll -and $Part -match 'F') {
  Case 'F a fault while the item menu is built'
  # A first run makes the fences and ends normally; the run under test has no test script, so
  # the watchdog starts it again after a crash, as for a user. (Part D may have ended early
  # with the extension still set to fault.)
  ExtDisarm
  PfStart
  Pf settings set featureTips false | Out-Null
  ClearFences | Out-Null
  $fn = @('sx m1 报告.txt', 'sx m2 预算.txt')
  SxFence 'SxM' '300,200,520,170' $fn
  PfQuit
  Start-Sleep -Seconds 2
  # No restart may be on record: a crash this soon after a start is answered only once in
  # ten minutes.
  Remove-Item $Stamp -ErrorAction SilentlyContinue
  $env:PECOFENCE_INSTANCE = $PfInst
  $env:RUST_LOG = 'pecofence=debug'
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $PfApp
  $null = WaitWin 'PecoFence.Control' 20 -Hidden
  Start-Sleep -Seconds 9
  $p1 = Get-Process pecofence | Select-Object -First 1
  # A change a second before the fault must still be there afterwards.
  Pf fence create --title 'SxKeep' --rect '900,500,300,150' | Out-Null
  Start-Sleep -Milliseconds 300
  $p = CellPt 'SxM' 0
  ExtArm 'QueryContextMenu'
  # One clock: t0 is the moment the right button is let go (RClick moves, waits 80 ms,
  # presses, waits 40 ms, releases); the watch's times are moved onto it.
  $t0 = [DateTime]::UtcNow.AddMilliseconds(120)
  [G]::RClick($p[0], $p[1])
  $off = [int]([DateTime]::UtcNow - $t0).TotalMilliseconds
  $watch = [SxMon]::Watch($p1.Id, 90)
  ExtDisarm
  Log "watch (its clock starts $off ms after the click): $watch"
  $w = @{}; foreach ($kv in ($watch -split ' ')) { $a = $kv -split '=', 2; $w[$a[0]] = $a[1] }
  foreach ($k in 'exit', 'born', 'fences') { if ([int]$w[$k] -ge 0) { $w[$k] = [int]$w[$k] + $off } }
  $toast = ToastOcr 'crashed' 8
  $restarted = [int]$w['new_pid'] -ne 0
  $prev = "$env:LOCALAPPDATA\PecoFence\pecofence.$PfInst.prev.log"
  $crashLog = if ($restarted -and (Test-Path $prev)) { $prev } else { PfLogPath }
  $crashLine = @(Get-Content $crashLog -Encoding UTF8 | Where-Object { $_ -cmatch 'CRASH \(unhandled exception\)' }) | Select-Object -Last 1
  $crashMs = if ($crashLine) { [int]((LogTime $crashLine) - $t0).TotalMilliseconds } else { -1 }
  Check 'F the crash line names the extension as another program''s file' ($crashLine -match ' at=pf_shellext_fault\.dll\+0x' -and $crashLine -match 'module_owner="?other') "crash line $crashMs ms after the click: $(if ($crashLine -match '(code=.*?) dump=') { $Matches[1] })"
  Check 'F the process ends with the exception''s code and the watchdog starts PecoFence again' ($w['code'] -eq 'C0000005' -and $restarted -and [int]$w['fences'] -ge 0) "after the click: exit $($w['exit']) ms (code $($w['code'])), new process $($w['born']) ms, its first fence $($w['fences']) ms"
  # The watchdog starts PecoFence first, and that run takes over the hidden icons: Windows'
  # own icons do not show in between (they did for 0.3 s), and the fences are back sooner.
  Check 'F the fences are back within a second of the click, and Windows'' own icons never show in between' ([int]$w['icons'] -lt 0 -and [int]$w['fences'] -ge 0 -and [int]$w['fences'] -lt 1000) "first fence $($w['fences']) ms after the click; Windows' icons seen at $($w['icons']) ms (-1 = never)"
  # Left to Windows Error Reporting, the dead process stayed 1 to 2 s on this box (about 50 s
  # on the PC of GitHub #167).
  $dead = [int]$w['exit'] - $crashMs
  Check 'F the process is gone within half a second of its crash line' ($crashMs -ge 0 -and [int]$w['exit'] -ge 0 -and $dead -lt 500) "crash line at $crashMs ms, exit at $($w['exit']) ms: $dead ms"
  if ($restarted) {
    $new = @(Get-Content (PfLogPath) -Encoding UTF8)
    $said = @($new | Where-Object { $_ -match 'the crash was inside another program''s module' }) | Select-Object -Last 1
    $warm = @($new | Where-Object { $_ -match 'shell menu (pre|not pre)warmed' }) | Select-Object -Last 1
    Check 'F the next start names the file in its notice' ($said -match 'pf_shellext_fault' -and $toast -match 'pf_shellext_fault') "log: $(if ($said -match '(the crash was.*)$') { $Matches[1] }); notice: $toast"
    Check 'F the next start does not load the menu extensions in the background' ($warm -match 'not prewarmed') "log: $(if ($warm -match '(shell menu.*)$') { $Matches[1] })"
    $titles = @(@(PfJson fence list) | ForEach-Object { $_.title }) -join ', '
    Check 'F the fences are as before the fault, the newest one too' ($titles -match 'SxM' -and $titles -match 'SxKeep') "fences: $titles"
  }
  if (PfAlive) { PfQuit }
  Get-Process pecofence, pecofence-watchdog -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  Start-Sleep -Seconds 3
  if (-not [W]::IconsVisible()) { [W]::ToggleIcons(); Start-Sleep -Seconds 1 }
  # The restarted run switched the Windows part of the item menus off: on again for the rest.
  Remove-Item $Stamp, $SxMarker -ErrorAction SilentlyContinue
  Remove-Item "$env:LOCALAPPDATA\PecoFence\crash-*.dmp" -ErrorAction SilentlyContinue
}

if ($dll -and $Part -match 'M') {
  Case 'M after a crash in a menu extension'
  ExtDisarm
  Remove-Item $SxMarker -ErrorAction SilentlyContinue
  PfStart
  Pf settings set featureTips false | Out-Null
  ClearFences | Out-Null
  SxFence 'SxN' '300,200,520,170' @('sx n1 报告.txt', 'sx n2 预算.txt')
  PfQuit
  Start-Sleep -Seconds 2
  Remove-Item $Stamp -ErrorAction SilentlyContinue
  SxPlain
  Start-Sleep -Seconds 9
  $p1 = Get-Process pecofence | Select-Object -First 1
  $p = CellPt 'SxN' 0
  ExtArm 'QueryContextMenu'
  [G]::RClick($p[0], $p[1])
  $watch = [SxMon]::Watch($p1.Id, 60)
  Log "watch: $watch"
  $toast = ToastOcr 'Windows entries' 8
  Start-Sleep -Seconds 3
  $said = SxNewLog 'item menus are built without the Windows part'
  Check 'M the next start turns the Windows part of the item menus off and says so' ($said -and (Test-Path $SxMarker) -and $toast -match 'pf_shellext_fault' -and $toast -match 'Windows entries') "log: $(if ($said -match '(item menus.*)$') { $Matches[1] }); marker there=$(Test-Path $SxMarker); notice: $toast"
  # The same right-click, the extension still set to fault.
  $p2 = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
  Remove-Item "$ExtDir\calls.log" -ErrorAction SilentlyContinue
  [G]::RClick($p[0], $p[1])
  $m = MenuWin 8
  $texts = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
  Shot 'M-menu-without-windows-part'
  CloseMenus
  Start-Sleep -Milliseconds 500
  $p3 = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
  Check 'M the same right-click again shows PecoFence''s entries, and PecoFence stays' ($texts -match 'Delete' -and $texts -match 'Rename' -and $texts -notmatch 'PF fault test' -and $p2 -and $p3 -and $p2.Id -eq $p3.Id -and @(ExtCalls).Count -eq 0) "same process=$($p2 -and $p3 -and $p2.Id -eq $p3.Id); extension calls: $((ExtCalls) -join ', '); menu: $texts"
  # Windows' part had Cut and Copy: the menu has PecoFence's own now, with their keys, and
  # they do what Ctrl+X and Ctrl+C do.
  [Windows.Forms.Clipboard]::Clear()
  [G]::RClick($p[0], $p[1])
  $m = MenuWin 8
  $picked = MenuPick '^Copy ~ Ctrl\+C$' $m
  Start-Sleep -Milliseconds 800
  $copied = "$((ClipFiles | ForEach-Object { Split-Path $_ -Leaf }) -join ' | ') effect=$(ClipEffect)"
  [G]::RClick($p[0], $p[1])
  $m = MenuWin 8
  $picked2 = MenuPick '^Cut ~ Ctrl\+X$' $m
  Start-Sleep -Milliseconds 800
  $cut = "$((ClipFiles | ForEach-Object { Split-Path $_ -Leaf }) -join ' | ') effect=$(ClipEffect)"
  [Windows.Forms.Clipboard]::Clear()
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300
  Check 'M that menu has PecoFence''s own Cut and Copy, and they work' ($texts -match 'Cut ~ Ctrl\+X \| Copy ~ Ctrl\+C \| Properties' -and $picked -and $picked2 -and $copied -eq 'sx n1 报告.txt effect=5' -and $cut -eq 'sx n1 报告.txt effect=2' -and @(ExtCalls).Count -eq 0) "Copy: $copied; Cut: $cut; extension calls: $((ExtCalls) -join ', ')"
  # A normal exit and start: no menu is built in the background while the part is off.
  PfQuit
  Start-Sleep -Seconds 2
  SxPlain
  Start-Sleep -Seconds 8
  $warm = SxNewLog 'shell menu (pre|not pre)warmed'
  Check 'M while it is off, a start builds no menu in the background' ($warm -match 'the Windows part of item menus is off' -and @(ExtCalls).Count -eq 0) "log: $(if ($warm -match '(shell menu.*)$') { $Matches[1] }); extension calls: $((ExtCalls) -join ', ')"
  # The tray menu's entry brings the Windows part back.
  ExtDisarm
  $picked = TrayPick 'Turn Windows entries'
  Start-Sleep -Milliseconds 800
  Check 'M the tray menu has an entry that turns the Windows part back on' ($picked -and -not (Test-Path $SxMarker)) "picked: $picked; marker there=$(Test-Path $SxMarker); tray menu: $script:LastTrayMenu"
  [G]::RClick($p[0], $p[1])
  $m = MenuWin 8
  $texts = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
  CloseMenus
  Check 'M after that the item menu has Windows'' entries again' ($texts -match 'PF fault test' -and $texts -match 'Send to') "menu: $texts"
  $m = TrayMenu
  $tray = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
  CloseMenus
  Check 'M and the tray entry is gone' ($tray -match 'Settings|Exit' -and $tray -notmatch 'Turn Windows entries') "tray menu: $tray"
  if (PfAlive) { PfQuit }
  Get-Process pecofence, pecofence-watchdog -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  Start-Sleep -Seconds 3
  if (-not [W]::IconsVisible()) { [W]::ToggleIcons(); Start-Sleep -Seconds 1 }
  Remove-Item $Stamp, $SxMarker -ErrorAction SilentlyContinue
  Remove-Item "$env:LOCALAPPDATA\PecoFence\crash-*.dmp" -ErrorAction SilentlyContinue
  SxClean
}

if ($dll -and $Part -match 'A') {
  Case 'A a menu extension that faults in the menu built in the background'
  ExtDisarm
  Remove-Item $SxMarker -ErrorAction SilentlyContinue
  PfStart
  Pf settings set featureTips false | Out-Null
  ClearFences | Out-Null
  SxFence 'SxA' '300,200,520,170' @('sx g1 报告.txt', 'sx g2 预算.txt')
  PfQuit
  Start-Sleep -Seconds 2
  Remove-Item $Stamp -ErrorAction SilentlyContinue
  # A fault while the DLL is loaded: Windows catches it and fails the load, so PecoFence goes
  # on and the menu comes without that extension's entry.
  ExtArm 'DllMain'
  SxPlain
  Start-Sleep -Seconds 8
  $p0 = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
  $p = CellPt 'SxA' 0
  [G]::RClick($p[0], $p[1])
  $m = MenuWin 8
  $texts = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
  CloseMenus
  Start-Sleep -Milliseconds 500
  $p0b = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
  $crash = @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -cmatch 'CRASH \(' }).Count
  Check 'A a fault while the DLL is loaded ends nothing: the menu comes without that extension' ($p0 -and $p0b -and $p0.Id -eq $p0b.Id -and $crash -eq 0 -and $texts -match 'Send to' -and $texts -notmatch 'PF fault test' -and ((ExtCalls) -join ',') -eq 'DllMain') "same process=$($p0 -and $p0b -and $p0.Id -eq $p0b.Id); crash lines=$crash; extension calls: $((ExtCalls) -join ', '); menu: $texts"
  PfQuit
  Start-Sleep -Seconds 2
  # Set to fault whenever its menu is built, before the start: the menu built in the background
  # two seconds after the start is the first one.
  ExtArm 'QueryContextMenu'
  SxPlain
  $p1 = Get-Process pecofence | Select-Object -First 1
  $watch = [SxMon]::Watch($p1.Id, 60)
  Log "watch: $watch"
  $w = @{}; foreach ($kv in ($watch -split ' ')) { $a = $kv -split '=', 2; $w[$a[0]] = $a[1] }
  $toast = ToastOcr 'Windows entries' 8
  Start-Sleep -Seconds 3
  $prev = "$env:LOCALAPPDATA\PecoFence\pecofence.$PfInst.prev.log"
  $crashLog = if ([int]$w['new_pid'] -ne 0 -and (Test-Path $prev)) { $prev } else { PfLogPath }
  $crashLine = @(Get-Content $crashLog -Encoding UTF8 | Where-Object { $_ -cmatch 'CRASH \(unhandled exception\)' }) | Select-Object -Last 1
  Check 'A the crash line names the extension and the menu built in the background' ($crashLine -match ' at=pf_shellext_fault\.dll\+0x' -and $crashLine -match 'command="?Prewarm' -and $crashLine -match 'module_owner="?other') "$(if ($crashLine -match '(code=.*?) dump=') { $Matches[1] })"
  $said = SxNewLog 'item menus are built without the Windows part'
  Check 'A PecoFence is started again with the Windows part of the item menus off' ($w['code'] -eq 'C0000005' -and [int]$w['fences'] -ge 0 -and $said -and (Test-Path $SxMarker) -and $toast -match 'pf_shellext_fault') "exit $($w['exit']) ms after the start was seen, code $($w['code']); new fences $($w['fences']) ms; log: $(if ($said -match '(item menus.*)$') { $Matches[1] }); marker there=$(Test-Path $SxMarker); notice: $toast"
  # A right-click now, the extension still set to fault.
  $p2 = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
  Remove-Item "$ExtDir\calls.log" -ErrorAction SilentlyContinue
  $p = CellPt 'SxA' 0
  [G]::RClick($p[0], $p[1])
  $m = MenuWin 8
  $texts = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
  CloseMenus
  Start-Sleep -Milliseconds 500
  $p3 = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
  Check 'A a right-click then shows PecoFence''s entries and loads nothing' ($texts -match 'Delete' -and $p2 -and $p3 -and $p2.Id -eq $p3.Id -and @(ExtCalls).Count -eq 0) "same process=$($p2 -and $p3 -and $p2.Id -eq $p3.Id); extension calls: $((ExtCalls) -join ', '); menu: $texts"
  ExtDisarm
  if (PfAlive) { PfQuit }
  Get-Process pecofence, pecofence-watchdog -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  Start-Sleep -Seconds 3
  if (-not [W]::IconsVisible()) { [W]::ToggleIcons(); Start-Sleep -Seconds 1 }
  Remove-Item $Stamp, $SxMarker -ErrorAction SilentlyContinue
  Remove-Item "$env:LOCALAPPDATA\PecoFence\crash-*.dmp" -ErrorAction SilentlyContinue
  SxClean
}

if ($dll -and $Part -match 'H') {
  Case 'H a menu extension that never returns'
  ExtDisarm
  Remove-Item $SxMarker -ErrorAction SilentlyContinue
  PfStart
  Pf settings set featureTips false | Out-Null
  ClearFences | Out-Null
  SxFence 'SxH' '300,200,520,170' @('sx h1 报告.txt', 'sx h2 预算.txt')
  PfQuit
  Start-Sleep -Seconds 2
  Remove-Item $Stamp -ErrorAction SilentlyContinue
  SxPlain
  Start-Sleep -Seconds 9
  $p1 = Get-Process pecofence | Select-Object -First 1
  $p = CellPt 'SxH' 0
  ExtArm 'QueryContextMenu hang'
  $t0 = [DateTime]::UtcNow.AddMilliseconds(120)
  [G]::RClick($p[0], $p[1])
  $off = [int]([DateTime]::UtcNow - $t0).TotalMilliseconds
  $watch = [SxMon]::Watch($p1.Id, 90)
  ExtDisarm
  Log "watch (its clock starts $off ms after the click): $watch"
  $w = @{}; foreach ($kv in ($watch -split ' ')) { $a = $kv -split '=', 2; $w[$a[0]] = $a[1] }
  foreach ($k in 'exit', 'born', 'fences') { if ([int]$w[$k] -ge 0) { $w[$k] = [int]$w[$k] + $off } }
  $toast = ToastOcr 'Windows entries' 8
  $prev = "$env:LOCALAPPDATA\PecoFence\pecofence.$PfInst.prev.log"
  $hungLog = if ([int]$w['new_pid'] -ne 0 -and (Test-Path $prev)) { $prev } else { PfLogPath }
  $line = @(Get-Content $hungLog -Encoding UTF8 | Where-Object { $_ -match 'HANG: ending to restart' }) | Select-Object -Last 1
  Check 'H the frozen run says where it is stuck and ends itself after about 35 s' ($line -match ' at=pf_shellext_fault\.dll\+0x' -and $line -match 'command="?ItemMenu' -and $line -match ' frame=\d+ ' -and $w['code'] -eq 'CFFFFFFF' -and [int]$w['exit'] -ge 30000 -and [int]$w['exit'] -le 45000) "exit $($w['exit']) ms after the click, code $($w['code']); log: $(if ($line -match '(HANG: ending.*?) stack=') { $Matches[1] })"
  Check 'H the watchdog starts PecoFence again' ([int]$w['new_pid'] -ne 0 -and [int]$w['fences'] -ge 0) "new process $($w['born']) ms, its first fence $($w['fences']) ms after the click"
  Start-Sleep -Seconds 3
  $said = SxNewLog 'item menus are built without the Windows part'
  Check 'H that start leaves the Windows part of the item menus out and says so' ($said -and (Test-Path $SxMarker) -and $toast -match 'pf_shellext_fault') "log: $(if ($said -match '(item menus.*)$') { $Matches[1] }); marker there=$(Test-Path $SxMarker); notice: $toast"
  if (PfAlive) { PfQuit }
  Get-Process pecofence, pecofence-watchdog -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  Start-Sleep -Seconds 3
  if (-not [W]::IconsVisible()) { [W]::ToggleIcons(); Start-Sleep -Seconds 1 }
  Remove-Item $Stamp, $SxMarker -ErrorAction SilentlyContinue
  SxClean
}

if ($Part -match 'S') {
  Case 'S a long wait in PecoFence''s own code'
  if (-not $PfDebugBuild) { Check 'S runs on a debug build (the test script''s stall line)' $false 'run.sh --bin target/wintest-bin-debug --var Part=S' }
  PfStart -Script @('sleep 10000', 'stall 45000', 'sleep 3600000')
  $p1 = Get-Process pecofence | Select-Object -First 1
  # The stall starts about 5 s from here and lasts 45 s; the hang watch looks again after
  # 36 s of it.
  Start-Sleep -Seconds 58
  $p2 = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
  $log = @(Get-Content (PfLogPath) -Encoding UTF8)
  $left = @($log | Where-Object { $_ -match 'still not responding: not inside another program''s module, left alone' }).Count
  $back = @($log | Where-Object { $_ -match 'UI thread responding again' }) | Select-Object -Last 1
  $ended = @($log | Where-Object { $_ -match 'HANG: ending to restart' }).Count
  Check 'S a 45 s wait in PecoFence''s own code does not end the run' ($p1 -and $p2 -and $p1.Id -eq $p2.Id -and $left -ge 1 -and $ended -eq 0 -and $back) "same process=$($p1 -and $p2 -and $p1.Id -eq $p2.Id); left-alone lines=$left; ending lines=$ended; $(if ($back -match '(UI thread responding again.*)$') { $Matches[1] })"
  PfQuit
}

if ($Part -match 'O') {
  Case 'O a crash in PecoFence''s own code'
  # The test script's `crash` line writes to address 0 in pecofence.exe itself.
  $t0 = [DateTime]::UtcNow
  PfStart -Script @('sleep 12000', 'crash', 'sleep 3600000')
  $p1 = Get-Process pecofence | Select-Object -First 1
  $tw = [DateTime]::UtcNow
  $watch = [SxMon]::Watch($p1.Id, 60, 3000)
  Log "watch: $watch"
  $w = @{}; foreach ($kv in ($watch -split ' ')) { $a = $kv -split '=', 2; $w[$a[0]] = $a[1] }
  $crashLine = @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -cmatch 'CRASH \(unhandled exception\)' }) | Select-Object -Last 1
  $exitAt = $tw.AddMilliseconds([int]$w['exit'])
  $dead = if ($crashLine -and [int]$w['exit'] -ge 0) { [int]($exitAt - (LogTime $crashLine)).TotalMilliseconds } else { -1 }
  Start-Sleep -Seconds 2
  $ev = @(Get-WinEvent -FilterHashtable @{ LogName = 'Application'; ProviderName = 'Application Error'; StartTime = $t0.ToLocalTime() } -ErrorAction SilentlyContinue | Where-Object { $_.Message -match 'pecofence\.exe' })
  Check 'O the crash line says the crash was in PecoFence''s own file' ($crashLine -match ' at=pecofence\.exe\+0x' -and $crashLine -match 'module_owner="?pecofence') "$(if ($crashLine -match '(code=.*?) dump=') { $Matches[1] })"
  Check 'O the crash is left to Windows Error Reporting: Windows records it and the process ends after that' ($ev.Count -ge 1 -and $w['code'] -eq 'C0000005' -and $dead -ge 500) "Application Error events: $($ev.Count); exit $dead ms after the crash line, code $($w['code'])"
  Get-Process pecofence, pecofence-watchdog -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  Start-Sleep -Seconds 3
  if (-not [W]::IconsVisible()) { [W]::ToggleIcons(); Start-Sleep -Seconds 1 }
  Remove-Item "$env:LOCALAPPDATA\PecoFence\crash-*.dmp" -ErrorAction SilentlyContinue
}

Case 'cleanup'
if ($dll) {
  $left = ExtUnregister
  Check 'the test extension is unregistered again' ($left -eq 0) "keys left: $left"
}
SxClean
