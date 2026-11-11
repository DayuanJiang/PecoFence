# Another program's shell extension that faults inside PecoFence (GitHub #167, #178). The test
# extension (tools/shellext-fault, built by build.sh into target/wintest-shellext) is a menu
# handler for files and folders and a copy hook for folders. It writes every call to calls.log
# beside the DLL and faults in the entry point named in fault.txt, only inside pecofence.exe.
# The steps register it for pf and remove it again.
#   run.sh --extra target/wintest-shellext scripts/wintest/tests/shellext-fault.ps1
#   D  Delete calls no menu extension: with the extension set to fault in its first call,
#      Delete, Shift+Delete, the item menu's Delete, several files at once, a folder fence and
#      a folder still delete, and Ctrl+Z brings a deleted file back
#   F  a fault while the item menu is built (the shell loads every menu extension for it): the
#      crash line and the next start's notice name the DLL, the watchdog starts PecoFence again
#      and the layout is as before
# Parts: run.sh --var Part=D ... (default: all). About 3 minutes.
$PfAllowLogErrors = $true
if (-not $Part) { $Part = 'DF' }
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
  // fences = the new one shows a fence.
  public static string Watch(int pid, int seconds) {
    var sw = Stopwatch.StartNew();
    IntPtr hp = OpenProcess(0x00100000 | 0x1000, false, (uint)pid);
    long exited = -1, born = -1, shown = -1; uint code = 0; int newPid = 0;
    while (sw.ElapsedMilliseconds < seconds * 1000L && shown < 0) {
      long t = sw.ElapsedMilliseconds;
      if (exited < 0 && (hp == IntPtr.Zero || WaitForSingleObject(hp, 0) == 0)) { exited = t; GetExitCodeProcess(hp, out code); }
      if (exited >= 0 && newPid == 0) foreach (var p in Process.GetProcessesByName("pecofence")) if (p.Id != pid) { newPid = p.Id; born = t; }
      if (newPid != 0 && VisibleFences((uint)newPid) > 0) shown = t;
      Thread.Sleep(20);
    }
    return string.Format("exit={0} code={1:X8} new_pid={2} born={3} fences={4}", exited, code, newPid, born, shown);
  }
}
'@

$SxDir = 'C:\pf\sx-folder'
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
  ExtArm 'QueryContextMenu'
  $t0 = [DateTime]::UtcNow
  $p = CellPt 'SxM' 0
  [G]::RClick($p[0], $p[1])
  $watch = [SxMon]::Watch($p1.Id, 90)
  ExtDisarm
  Log "watch: $watch"
  $w = @{}; foreach ($kv in ($watch -split ' ')) { $a = $kv -split '=', 2; $w[$a[0]] = $a[1] }
  $toast = ToastOcr 'crashed' 8
  $restarted = [int]$w['new_pid'] -ne 0
  $prev = "$env:LOCALAPPDATA\PecoFence\pecofence.$PfInst.prev.log"
  $crashLog = if ($restarted -and (Test-Path $prev)) { $prev } else { PfLogPath }
  $crashLine = @(Get-Content $crashLog -Encoding UTF8 | Where-Object { $_ -cmatch 'CRASH \(unhandled exception\)' }) | Select-Object -Last 1
  $crashMs = if ($crashLine) { [int]((LogTime $crashLine) - $t0).TotalMilliseconds } else { -1 }
  Check 'F the crash line names the extension as another program''s file' ($crashLine -match ' at=pf_shellext_fault\.dll\+0x' -and $crashLine -match 'module_owner="?other') "crash line $crashMs ms after the click: $(if ($crashLine -match '(code=.*?) dump=') { $Matches[1] })"
  Check 'F the process ends with the exception''s code and the watchdog starts PecoFence again' ($w['code'] -eq 'C0000005' -and $restarted -and [int]$w['fences'] -ge 0) "after the click: exit $($w['exit']) ms (code $($w['code'])), new process $($w['born']) ms, its first fence $($w['fences']) ms"
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
  Remove-Item $Stamp -ErrorAction SilentlyContinue
  Remove-Item "$env:LOCALAPPDATA\PecoFence\crash-*.dmp" -ErrorAction SilentlyContinue
}

Case 'cleanup'
if ($dll) {
  $left = ExtUnregister
  Check 'the test extension is unregistered again' ($left -eq 0) "keys left: $left"
}
SxClean
