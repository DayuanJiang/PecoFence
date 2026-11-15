# Issues round 2026-11-15, package "crash". Parts (run.sh --var Part=A, B or C; all by default):
#  A  feedback #163: another program's code unmasks floating-point exceptions on the UI thread
#     (as a menu extension did on the reporter's PC); the next frame of an icon's fade must not
#     end the process with 0xc0000090.
#  B  feedback #165: the feedback form's "N above other windows" counts only windows of other
#     programs between a fence and the desktop.
#  C  feedback #113: cancelling Windows' 「替换或跳过文件」 dialog of a file dropped on a fence.

$IcPart = 'ABC'
if ($Part) { $IcPart = $Part }

Add-Type @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class Fp {
  [DllImport("kernel32.dll", SetLastError = true)] static extern IntPtr OpenThread(uint access, bool inherit, uint tid);
  [DllImport("kernel32.dll")] static extern uint SuspendThread(IntPtr h);
  [DllImport("kernel32.dll")] static extern uint ResumeThread(IntPtr h);
  [DllImport("kernel32.dll", SetLastError = true)] static extern bool GetThreadContext(IntPtr h, IntPtr ctx);
  [DllImport("kernel32.dll", SetLastError = true)] static extern bool SetThreadContext(IntPtr h, IntPtr ctx);
  [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
  // MXCSR of the thread that owns hwnd becomes (old & and) | or. CONTEXT (x64) is 1232 bytes,
  // 16-byte aligned: ContextFlags at 0x30, MxCsr at 0x34, FltSave (XMM_SAVE_AREA32) at 0x100
  // with its own MxCsr at +0x18.
  public static string SetMxcsr(IntPtr hwnd, uint andMask, uint orMask) {
    uint pid; uint tid = GetWindowThreadProcessId(hwnd, out pid);
    IntPtr h = OpenThread(0x0002 | 0x0008 | 0x0010 | 0x0040, false, tid);
    if (h == IntPtr.Zero) return "OpenThread failed " + Marshal.GetLastWin32Error();
    IntPtr raw = Marshal.AllocHGlobal(1232 + 16);
    IntPtr ctx = new IntPtr((raw.ToInt64() + 15) & ~15L);
    try {
      for (int i = 0; i < 1232; i += 4) Marshal.WriteInt32(ctx, i, 0);
      Marshal.WriteInt32(ctx, 0x30, 0x00100009);
      if (SuspendThread(h) == 0xFFFFFFFF) return "SuspendThread failed";
      try {
        if (!GetThreadContext(h, ctx)) return "GetThreadContext failed " + Marshal.GetLastWin32Error();
        uint old = (uint)Marshal.ReadInt32(ctx, 0x34);
        uint nv = (old & andMask) | orMask;
        Marshal.WriteInt32(ctx, 0x34, (int)nv);
        Marshal.WriteInt32(ctx, 0x118, (int)nv);
        if (!SetThreadContext(h, ctx)) return "SetThreadContext failed " + Marshal.GetLastWin32Error();
        for (int i = 0; i < 1232; i += 4) Marshal.WriteInt32(ctx, i, 0);
        Marshal.WriteInt32(ctx, 0x30, 0x00100009);
        GetThreadContext(h, ctx);
        uint back = (uint)Marshal.ReadInt32(ctx, 0x34);
        return string.Format("tid={0} mxcsr {1:x4} -> {2:x4} (read back {3:x4})", tid, old, nv, back);
      } finally { ResumeThread(h); }
    } finally { Marshal.FreeHGlobal(raw); CloseHandle(h); }
  }
}
public static class Zb {
  delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] static extern IntPtr GetShellWindow();
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RC r);
  [DllImport("dwmapi.dll")] static extern int DwmGetWindowAttribute(IntPtr h, int a, out int v, int s);
  [StructLayout(LayoutKind.Sequential)] public struct RC { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int hh, bool r);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  static string Cls(IntPtr h) { var s = new StringBuilder(256); GetClassNameW(h, s, 256); return s.ToString(); }
  static string Proc(uint pid) { try { return System.Diagnostics.Process.GetProcessById((int)pid).ProcessName; } catch { return "?"; } }
  // The windows with a visible surface between the highest fence and Explorer's desktop window
  // (the same test as the feedback form's count), top first: class, process, rect.
  public static string Between() {
    var all = new List<IntPtr>();
    EnumProc cb = (h, l) => { all.Add(h); return true; };
    EnumWindows(cb, IntPtr.Zero);
    uint shellPid; GetWindowThreadProcessId(GetShellWindow(), out shellPid);
    int first = -1, host = -1;
    for (int i = 0; i < all.Count; i++) {
      string c = Cls(all[i]);
      uint pid; GetWindowThreadProcessId(all[i], out pid);
      if (first < 0 && c == "PecoFence.Fence" && IsWindowVisible(all[i])) first = i;
      if ((c == "Progman" || c == "WorkerW") && pid == shellPid && IsWindowVisible(all[i])) { host = i; }
    }
    if (first < 0) return "no visible fence";
    var sb = new StringBuilder();
    for (int i = first + 1; i < all.Count && (host < 0 || i < host); i++) {
      IntPtr h = all[i];
      if (!IsWindowVisible(h) || IsIconic(h)) continue;
      int cloak; DwmGetWindowAttribute(h, 14, out cloak, 4);
      if (cloak != 0) continue;
      RC r; GetWindowRect(h, out r);
      if (r.R <= r.L || r.B <= r.T) continue;
      string c = Cls(h);
      if (c == "PecoFence.Fence" || c == "PecoFence.Shadow") continue;
      uint pid; GetWindowThreadProcessId(h, out pid);
      sb.Append(string.Format("{0} [{1}] {2},{3},{4},{5}; ", c, Proc(pid), r.L, r.T, r.R, r.B));
    }
    return sb.Length == 0 ? "none" : sb.ToString();
  }
}
'@

function UiThreadFence() {
  foreach ($h in @([W]::Tops('PecoFence.Fence'))) { if ([G]::IsWindowVisible($h)) { return $h } }
  return [IntPtr]::Zero
}
function Crashes() { return @(PfLogAll | Where-Object { $_ -match 'CRASH \(unhandled exception\)' }) }
function NthFeedbackInfo($n, $sec = 40) {
  for ($i = 0; $i -lt $sec * 5; $i++) {
    $p = PfLogPath
    if (Test-Path $p) {
      $m = @(Get-Content $p -Encoding UTF8 | Where-Object { $_ -match 'feedback-info \{' })
      if ($m.Count -ge $n) { $l = $m[$n - 1]; return ($l.Substring($l.IndexOf('feedback-info {') + 14) | ConvertFrom-Json) }
    }
    Start-Sleep -Milliseconds 200
  }
  return $null
}
$made = New-Object System.Collections.ArrayList
function Made($p) { [void]$made.Add($p); return $p }

# ---------------------------------------------------------------- A: unmasked exceptions
if ($IcPart -match 'A') {
  Case 'A setup'
  CloseWindows
  foreach ($n in '浮点 报告.txt', '浮点 Beta 文档.txt', 'fp 计划 一.txt') { Set-Content -LiteralPath (Made (Join-Path $PfDesk $n)) 'fp' }
  PfFresh
  Start-Sleep -Seconds 3
  ClearFences
  Pf fence create --title '浮点 测试' --rect '480,140,420,300' | Out-Null
  foreach ($n in '浮点 报告.txt', '浮点 Beta 文档.txt', 'fp 计划 一.txt') { Pf item move (Join-Path $PfDesk $n) --to '浮点 测试' | Out-Null }
  Start-Sleep -Seconds 2
  ParkMouse 1300 800

  Case 'A1 invalid operation unmasked while nothing happens, then a right-click on an icon'
  $f = UiThreadFence
  $r = [Fp]::SetMxcsr($f, [Convert]::ToUInt32("FFFFFF40", 16), 0)
  Log "inject: $r"
  Start-Sleep -Milliseconds 300
  $p = try { ItemPt '浮点 测试' '浮点 报告' } catch { $null }
  if ($p) { [G]::RClick($p[0], $p[1]) }
  Start-Sleep -Milliseconds 1200
  Shot 'A1-menu'
  if (PfAlive) {
    CloseMenus
    $q = try { ItemPt '浮点 测试' 'fp 计划 一' } catch { $null }
    if ($q) { [G]::MoveTo($q[0], $q[1]); Start-Sleep -Milliseconds 600; [G]::MoveTo($q[0] - 80, $q[1]); Start-Sleep -Milliseconds 600 }
  }
  ParkMouse 1300 800
  Start-Sleep -Seconds 2
  $alive = PfAlive
  $c = @(Crashes)
  $restored = PfWaitLog 'floating-point control word changed by other code' 1
  Log "crash lines: $($c -join ' || ')"
  Check 'A1 the process keeps running (no 0xc0000090 crash)' ($r -match '^tid=' -and $alive -and $c.Count -eq 0) "alive=$alive inject='$r' crash='$($c -join ' | ')'"
  Check 'A1 the log says the control word was found changed and put back' ([bool]$restored) "$restored"

  Case 'A2 unmasked while the item menu is open, then Esc and a hover'
  $before = @(Crashes).Count
  if (-not (PfAlive)) {
    PfStart; Start-Sleep -Seconds 4; ParkMouse 1300 800
  }
  $p = try { ItemPt '浮点 测试' '浮点 Beta 文档' } catch { $null }
  if ($p) { [G]::RClick($p[0], $p[1]) }
  Start-Sleep -Milliseconds 1000
  $f = UiThreadFence
  $r = [Fp]::SetMxcsr($f, [Convert]::ToUInt32("FFFFFF40", 16), 0)
  Log "inject during the menu: $r"
  Start-Sleep -Milliseconds 300
  [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  if (PfAlive) {
    $q = try { ItemPt '浮点 测试' '浮点 报告' } catch { $null }
    if ($q) { [G]::MoveTo($q[0], $q[1]); Start-Sleep -Milliseconds 700; [G]::MoveTo($q[0] + 90, $q[1]); Start-Sleep -Milliseconds 700 }
  }
  ParkMouse 1300 800
  Start-Sleep -Seconds 2
  $alive = PfAlive
  $c = @(Crashes)
  Check 'A2 the process keeps running' ($r -match '^tid=' -and $alive -and $c.Count -eq $before) "alive=$alive inject='$r' crash='$($c -join ' | ')'"
  $lines = @(PfLogAll | Where-Object { $_ -match 'floating-point control word' })
  Log "control word lines: $($lines -join ' || ')"
  if (PfAlive) { PfQuit }
}

# ---------------------------------------------------------------- B: the count of covered windows
if ($IcPart -match 'B') {
  Case 'B setup: three fences, a merged tab, a rolled fence, Peek a few times, the settings open'
  CloseWindows
  foreach ($n in 'cov 甲 文件.txt', 'cov Beta.txt', 'cov 丙 计划 表.txt', 'cov Delta.txt') { Set-Content -LiteralPath (Made (Join-Path $PfDesk $n)) 'cov' }
  PfFresh @('sleep 25000', 'feedback-info', 'sleep 25000', 'feedback-info', 'sleep 25000', 'feedback-info', 'sleep 3600000')
  $t0 = Get-Date
  Start-Sleep -Seconds 3
  ClearFences
  Pf fence create --title '覆盖 甲' --rect '420,120,300,260' | Out-Null
  Pf fence create --title '覆盖 Beta' --rect '760,120,300,260' | Out-Null
  Pf fence create --title '覆盖 丙' --rect '1100,120,300,260' | Out-Null
  Pf fence create --title '覆盖 合并' --rect '420,480,300,260' | Out-Null
  Pf item move (Join-Path $PfDesk 'cov 甲 文件.txt') --to '覆盖 甲' | Out-Null
  Pf item move (Join-Path $PfDesk 'cov Beta.txt') --to '覆盖 Beta' | Out-Null
  Pf item move (Join-Path $PfDesk 'cov 丙 计划 表.txt') (Join-Path $PfDesk 'cov Delta.txt') --to '覆盖 丙' | Out-Null
  Pf fence merge '覆盖 合并' --into '覆盖 甲' | Out-Null
  Pf fence roll '覆盖 Beta' | Out-Null
  Start-Sleep -Seconds 1
  Log "between, at rest: $([Zb]::Between())"
  foreach ($k in 1..3) { Pf peek start | Out-Null; Start-Sleep -Milliseconds 900; Pf peek end | Out-Null; Start-Sleep -Milliseconds 700 }
  Log "between, after three Peeks: $([Zb]::Between())"
  Pf settings open-ui | Out-Null
  Start-Sleep -Seconds 4
  Log "between, settings open: $([Zb]::Between())"
  $wait = 25 - ((Get-Date) - $t0).TotalSeconds
  if ($wait -gt 0) { Start-Sleep -Seconds ([int]$wait + 1) }
  $fi = NthFeedbackInfo 1
  $b1 = [Zb]::Between()
  Log "state 1: $($fi.system.state)"
  Log "between 1: $b1"
  Shot 'B1-settings-open'

  Case 'B2 Peek by its keys, the dimmer clicked, double Ctrl, a Notepad window'
  foreach ($k in 1..2) { PeekKey; Start-Sleep -Milliseconds 900; DeskClick 1500 820; Start-Sleep -Milliseconds 800 }
  DoubleCtrl; Start-Sleep -Milliseconds 900; [G]::Press(0x1B); Start-Sleep -Milliseconds 600
  Start-Process notepad.exe; Start-Sleep -Seconds 2
  $wait = 50 - ((Get-Date) - $t0).TotalSeconds
  if ($wait -gt 0) { Start-Sleep -Seconds ([int]$wait + 1) }
  $fi2 = NthFeedbackInfo 2
  $b2 = [Zb]::Between()
  Log "state 2: $($fi2.system.state)"
  Log "between 2: $b2"
  Shot 'B2-notepad'

  Case 'B3 Notepad minimized, the desktop clicked'
  foreach ($h in @([W]::Tops('Notepad'))) { [Zb]::ShowWindow($h, 6) | Out-Null }
  Start-Sleep -Seconds 1
  DeskClick 1500 820
  $wait = 75 - ((Get-Date) - $t0).TotalSeconds
  if ($wait -gt 0) { Start-Sleep -Seconds ([int]$wait + 1) }
  $fi3 = NthFeedbackInfo 3
  $b3 = [Zb]::Between()
  Log "state 3: $($fi3.system.state)"
  Log "between 3: $b3"
  $count = { param($fi) if ("$($fi.system.state)" -match '(\d+) above other windows') { [int]$Matches[1] } else { -1 } }
  $n1 = & $count $fi; $n2 = & $count $fi2; $n3 = & $count $fi3
  $other = { param($b) ($b -split '; ' | Where-Object { $_ -and $_ -ne 'none' -and $_ -notmatch '\[pecofence\]' }).Count }
  Check 'B the count is 0 when no window of another program is between a fence and the desktop' ((($n1 -eq 0) -or (& $other $b1) -gt 0) -and (($n2 -eq 0) -or (& $other $b2) -gt 0) -and (($n3 -eq 0) -or (& $other $b3) -gt 0)) "counts $n1 / $n2 / $n3; between: [$b1] [$b2] [$b3]"
  Check 'B the state line names what it counted when it counts' (($n1 -le 0 -or "$($fi.system.state)" -match 'above other windows \(') -and ($n2 -le 0 -or "$($fi2.system.state)" -match 'above other windows \(') -and ($n3 -le 0 -or "$($fi3.system.state)" -match 'above other windows \(')) "$($fi.system.state) || $($fi2.system.state) || $($fi3.system.state)"
  CloseWindows
  PfQuit
}

# ---------------------------------------------------------------- C: cancel in the conflict dialog
if ($IcPart -match 'C') {
  Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
  Case 'C setup: a file on the desktop in a fence, the same name in a folder open in Explorer'
  CloseWindows
  $src = Join-Path $PfRun 'src 冲突'
  Remove-Item -Recurse -Force $src -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force $src | Out-Null
  $name = '冲突 重复 报告.txt'
  Set-Content -LiteralPath (Made (Join-Path $PfDesk $name)) 'desktop copy'
  Set-Content -LiteralPath (Join-Path $src $name) 'folder copy, newer'
  PfFresh
  Start-Sleep -Seconds 3
  ClearFences
  Pf fence create --title '冲突 栅栏' --rect '980,160,420,300' | Out-Null
  Pf item move (Join-Path $PfDesk $name) --to '冲突 栅栏' | Out-Null
  Start-Sleep -Seconds 1
  Start-Process explorer.exe -ArgumentList "`"$src`""
  $ex = [IntPtr]::Zero
  for ($i = 0; $i -lt 40 -and $ex -eq [IntPtr]::Zero; $i++) { Start-Sleep -Milliseconds 250; $ex = @([W]::Tops('CabinetWClass'))[0]; if (-not $ex) { $ex = [IntPtr]::Zero } }
  [Zb]::MoveWindow($ex, 40, 120, 820, 560, $true) | Out-Null
  Start-Sleep -Seconds 2
  # Details view: the file is a row with its name.
  function SrcItemPt() {
    $el = UiaFind $ex $name
    if (-not $el) { return $null }
    $b = $el.Current.BoundingRectangle
    return @([int]($b.X + [math]::Min(60, $b.Width / 2)), [int]($b.Y + $b.Height / 2))
  }
  function FenceDropPt() { $r = (FenceGet '冲突 栅栏').windowRect; return @([int]($r.x + $r.width / 2), [int]($r.y + $r.height - 60)) }
  function ConflictDlg($sec = 6) {
    for ($i = 0; $i -lt $sec * 5; $i++) {
      foreach ($h in @([W]::Tops('OperationStatusWindow')) + @([W]::Tops('#32770'))) {
        if ($h -and [G]::IsWindowVisible($h) -and ([W]::ProcOf($h) -eq 'pecofence')) { return $h }
      }
      Start-Sleep -Milliseconds 200
    }
    return [IntPtr]::Zero
  }
  function ButtonNames($h) {
    $root = [Windows.Automation.AutomationElement]::FromHandle($h)
    $all = $root.FindAll([Windows.Automation.TreeScope]::Descendants, [Windows.Automation.Condition]::TrueCondition)
    return (@($all | ForEach-Object { "$($_.Current.ControlType.ProgrammaticName -replace 'ControlType\.','')=$($_.Current.Name)" } | Where-Object { $_ -notmatch '=$' }) -join ' | ')
  }
  function Answer($h, $how) {
    switch ($how) {
      'esc' { [G]::SetForegroundWindow($h) | Out-Null; Start-Sleep -Milliseconds 200; [G]::Press(0x1B) }
      'close' { $el = UiaFind $h 'Close'; if ($el) { $b = $el.Current.BoundingRectangle; [G]::Click([int]($b.X + $b.Width / 2), [int]($b.Y + $b.Height / 2)) } else { [Zb]::PostMessageW($h, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null } }
      'skip' { $el = UiaFind $h 'Skip this file'; if (-not $el) { $el = UiaFind $h 'Skip' }; if ($el) { $b = $el.Current.BoundingRectangle; [G]::Click([int]($b.X + $b.Width / 2), [int]($b.Y + $b.Height / 2)) } }
    }
  }
  $rows = @(); $seen = 0; $crashAt = ''
  foreach ($how in 'esc', 'close', 'skip') {
    Case "C $how in the conflict dialog, 8 times"
    for ($k = 1; $k -le 8; $k++) {
      if (-not (PfAlive)) { break }
      if (-not (Test-Path -LiteralPath (Join-Path $src $name))) { Set-Content -LiteralPath (Join-Path $src $name) "folder copy $k" }
      $p = SrcItemPt; $q = FenceDropPt
      if (-not $p) { $rows += "${how} ${k}: source row not found"; continue }
      DragTo $p[0] $p[1] $q[0] $q[1] 20
      $dlg = ConflictDlg
      if ($dlg -eq [IntPtr]::Zero) { $rows += "${how} ${k}: no dialog"; Shot "C-$how-$k-nodlg"; Start-Sleep -Seconds 1; continue }
      $seen++
      if ($seen -eq 1) { Log "dialog: class $([W]::Info($dlg)); $(ButtonNames $dlg)"; Shot 'C-dialog' }
      Start-Sleep -Milliseconds (300 + 150 * ($k % 4))
      Answer $dlg $how
      Start-Sleep -Milliseconds 2500
      $c = @(Crashes)
      $rows += "${how} ${k}: alive=$(PfAlive) dialog left=$([G]::IsWindowVisible($dlg))"
      if ($c.Count -gt 0) { $crashAt = "$how $k"; break }
    }
    if ($crashAt) { break }
  }
  Log ($rows -join ' | ')
  $c = @(Crashes)
  Log "crash lines: $($c -join ' || ')"
  Check 'C the dialog came up for the drops' ($seen -ge 12) "$seen dialogs; $($rows -join ' | ')"
  Check 'C no crash after an answer in the conflict dialog' ((PfAlive) -and $c.Count -eq 0) "crash at '$crashAt': $($c -join ' | ')"
  CloseWindows
  if (PfAlive) { PfQuit }
  Remove-Item -Recurse -Force $src -ErrorAction SilentlyContinue
}

foreach ($p in $made) { Remove-Item -LiteralPath $p -Force -ErrorAction SilentlyContinue }
