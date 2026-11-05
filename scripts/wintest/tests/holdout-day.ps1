# Hold-out test: installing the build under test and a normal user's day with it, with the real
# mouse and keyboard. Needs --extra with old-setup.exe (released 0.2.5), new-setup.exe and
# portable.zip (the build under test). Runs the installed edition (no test instance), so the
# kit's Pf/PfStart/PfLogErrors are not used: the helpers below talk to the installed copy.

# ---- lib candidates

# A user's PecoFence (installed or portable, no PECOFENCE_INSTANCE): its CLI, its log, starting it
# the way Explorer does, waiting for it to quit.
function Ipf {
  $a = @($args | ForEach-Object { $_ })
  $r = & $script:CliExe @a 2>&1 | Out-String
  Log ("user cli " + ($a -join ' ') + " -> " + $r.Trim())
  return $r
}
# JSON only when the CLI printed some (an error goes to stderr as JSON too, with a prefix).
function IpfJson { $r = (Ipf @args).Trim(); if ($r -match '^[\[\{]') { return ($r | ConvertFrom-Json) }; return $null }
function UserLog() { return "$env:LOCALAPPDATA\PecoFence\pecofence.log" }
function StartUser($exePath) {
  Start-Process explorer.exe "`"$exePath`""
  $h = WaitWin 'PecoFence.Control' 30 -Hidden
  Start-Sleep -Seconds 4
  return $h
}
function WaitNoPf($sec = 15) {
  for ($i = 0; $i -lt $sec * 5; $i++) { if (-not (Get-Process pecofence -ErrorAction SilentlyContinue)) { return $true }; Start-Sleep -Milliseconds 200 }
  return $false
}
function WaitGoneWin($cls, $sec) {
  for ($i = 0; $i -lt $sec * 5; $i++) { $h = [G]::FindWindowW($cls, [NullString]::Value); if ($h -eq [IntPtr]::Zero -or -not [G]::IsWindowVisible($h)) { return $true }; Start-Sleep -Milliseconds 200 }
  return $false
}

# Windows of a class, popup menus (owner-drawn: text and rect through MN_GETHMENU), the window
# under a point, the desktop's own icon view.
Add-Type @'
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class DayW {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowTextW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern IntPtr SendMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] static extern int GetMenuItemCount(IntPtr m);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetMenuStringW(IntPtr m, uint id, StringBuilder s, int n, uint flags);
  [DllImport("user32.dll")] static extern uint GetMenuState(IntPtr m, uint id, uint flags);
  [DllImport("user32.dll")] static extern bool GetMenuItemRect(IntPtr h, IntPtr m, uint i, out RECT r);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(POINT p);
  [DllImport("user32.dll")] static extern IntPtr GetAncestor(IntPtr h, uint f);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr FindWindowExW(IntPtr p, IntPtr a, string c, string t);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr FindWindowW(string c, string t);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
  public static string Cls(IntPtr h) { var s = new StringBuilder(256); GetClassNameW(h, s, 256); return s.ToString(); }
  public static string Title(IntPtr h) { var s = new StringBuilder(512); GetWindowTextW(h, s, 512); return s.ToString(); }
  public static string ProcOf(IntPtr h) { uint pid; GetWindowThreadProcessId(h, out pid); try { return System.Diagnostics.Process.GetProcessById((int)pid).ProcessName; } catch { return "?"; } }
  // Visible top-level windows of a class, topmost first.
  public static IntPtr[] Tops(string cls) {
    var list = new List<IntPtr>();
    EnumWindows((h, l) => { if (IsWindowVisible(h) && Cls(h) == cls) list.Add(h); return true; }, IntPtr.Zero);
    return list.ToArray();
  }
  [DllImport("user32.dll")] static extern bool EnumChildWindows(IntPtr p, EnumProc f, IntPtr l);
  [DllImport("user32.dll")] static extern bool IsWindowEnabled(IntPtr h);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr SendMessageW(IntPtr h, uint m, IntPtr w, StringBuilder l);
  public static string Text(IntPtr h) { var s = new StringBuilder(2048); SendMessageW(h, 0x0D, (IntPtr)2048, s); return s.ToString(); }
  // Visible child buttons: "hwnd TAB caption without & TAB enabled TAB L,T,R,B" lines.
  public static string Buttons(IntPtr top) {
    var sb = new StringBuilder();
    EnumChildWindows(top, (h, l) => {
      if (Cls(h).IndexOf("Button", StringComparison.OrdinalIgnoreCase) >= 0 && IsWindowVisible(h)) {
        RECT r; GetWindowRect(h, out r);
        sb.Append((long)h + "\t" + Text(h).Replace("&", "") + "\t" + IsWindowEnabled(h) + "\t" + r.L + "," + r.T + "," + r.R + "," + r.B + "\n");
      }
      return true; }, IntPtr.Zero);
    return sb.ToString();
  }
  // The texts of a dialog's static controls (a message box's message).
  public static string Statics(IntPtr top) {
    var sb = new StringBuilder();
    EnumChildWindows(top, (h, l) => { var c = Cls(h); if (c == "Static" || c.StartsWith("TNewStaticText")) sb.Append(Text(h) + " "); return true; }, IntPtr.Zero);
    return sb.ToString();
  }
  public static string Info(IntPtr h) { RECT r; GetWindowRect(h, out r); return ProcOf(h) + " '" + Title(h) + "' " + r.L + "," + r.T + "," + r.R + "," + r.B; }
  // Items of an open popup menu window: "index TAB text TAB state TAB L,T,R,B" lines.
  public static string MenuItems(IntPtr menuWnd) {
    IntPtr m = SendMessageW(menuWnd, 0x01E1, IntPtr.Zero, IntPtr.Zero);
    if (m == IntPtr.Zero) return "";
    var sb = new StringBuilder();
    int n = GetMenuItemCount(m);
    for (int i = 0; i < n; i++) {
      var t = new StringBuilder(512);
      GetMenuStringW(m, (uint)i, t, 512, 0x400);
      uint st = GetMenuState(m, (uint)i, 0x400);
      RECT r; GetMenuItemRect(IntPtr.Zero, m, (uint)i, out r);
      sb.Append(i + "\t" + t.ToString().Replace("\t", " ~ ") + "\t" + st + "\t" + r.L + "," + r.T + "," + r.R + "," + r.B + "\n");
    }
    return sb.ToString();
  }
  public static string RootClassAt(int x, int y) {
    var p = new POINT(); p.X = x; p.Y = y;
    var h = WindowFromPoint(p);
    return h == IntPtr.Zero ? "" : Cls(GetAncestor(h, 2));
  }
  public static IntPtr DefView() {
    IntPtr found = FindWindowExW(FindWindowW("Progman", null), IntPtr.Zero, "SHELLDLL_DefView", null);
    if (found != IntPtr.Zero) return found;
    EnumWindows((h, l) => { if (Cls(h) == "WorkerW") { var d = FindWindowExW(h, IntPtr.Zero, "SHELLDLL_DefView", null); if (d != IntPtr.Zero) { found = d; return false; } } return true; }, IntPtr.Zero);
    return found;
  }
  // Windows' own desktop icons on screen (the FolderView list under SHELLDLL_DefView).
  public static bool IconsVisible() { var d = DefView(); if (d == IntPtr.Zero) return false; var lv = FindWindowExW(d, IntPtr.Zero, "SysListView32", null); return lv != IntPtr.Zero && IsWindowVisible(lv); }
  // Explorer's View > Show desktop icons.
  public static void ToggleIcons() { var d = DefView(); if (d != IntPtr.Zero) PostMessageW(d, 0x111, (IntPtr)0x7402, IntPtr.Zero); }
}
'@

# Popup menus: the topmost open one, its rows, a click on the row whose text matches.
function MenuWin($sec = 6) {
  for ($i = 0; $i -lt $sec * 10; $i++) { $m = @([DayW]::Tops('#32768')); if ($m.Count) { return $m[0] }; Start-Sleep -Milliseconds 100 }
  return [IntPtr]::Zero
}
function MenuRows($h) {
  $rows = @()
  foreach ($line in ([DayW]::MenuItems($h) -split "`n")) {
    if (-not $line) { continue }
    $p = $line -split "`t"; $r = $p[3] -split ','
    $rows += [pscustomobject]@{ i = [int]$p[0]; text = ($p[1] -replace '&', ''); state = [int64]$p[2]; L = [int]$r[0]; T = [int]$r[1]; R = [int]$r[2]; B = [int]$r[3] }
  }
  return $rows
}
# Entry texts, a greyed one marked (off).
function MenuTexts($h) { return ((@(MenuRows $h) | Where-Object { $_.text } | ForEach-Object { if ($_.state -band 3) { "$($_.text) (off)" } else { $_.text } }) -join ' | ') }
function MenuDupes($h) { return ((@(MenuRows $h) | Where-Object { $_.text } | Group-Object text | Where-Object { $_.Count -gt 1 } | ForEach-Object { $_.Name }) -join ', ') }
function MenuPoint($h, $regex) {
  $row = @(MenuRows $h) | Where-Object { $_.text -match $regex } | Select-Object -First 1
  if (-not $row) { return $null }
  return @([int](($row.L + $row.R) / 2), [int](($row.T + $row.B) / 2))
}
function MenuPick($regex, $h = [IntPtr]::Zero) {
  if ($h -eq [IntPtr]::Zero) { $h = MenuWin 5 }
  if ($h -eq [IntPtr]::Zero) { Log "MenuPick '$regex' found no menu"; return $null }
  $p = MenuPoint $h $regex
  if (-not $p) { Log "MenuPick '$regex' not among: $(MenuTexts $h)"; return $null }
  $t = (@(MenuRows $h) | Where-Object { $_.text -match $regex } | Select-Object -First 1).text
  [G]::Click($p[0], $p[1])
  Log "menu pick '$t'"
  return $t
}
function CloseMenus() { for ($i = 0; $i -lt 4 -and @([DayW]::Tops('#32768')).Count; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 300 } }

# A real click on a child button by its caption (Inno wizard, message boxes: UI Automation finds
# no Inno buttons on this box), the text of a message box.
function ClickButton($top, $regex) {
  $list = [DayW]::Buttons($top)
  foreach ($line in ($list -split "`n")) {
    if (-not $line) { continue }
    $p = $line -split "`t"
    if ($p[1] -match $regex -and $p[2] -eq 'True') {
      $r = $p[3] -split ','
      [G]::Click([int](([int]$r[0] + [int]$r[2]) / 2), [int](([int]$r[1] + [int]$r[3]) / 2))
      Log "button '$($p[1])' in $([DayW]::Cls($top))"
      return $p[1]
    }
  }
  Log ("no button '$regex' in $([DayW]::Cls($top)), only: " + (($list -split "`n" | ForEach-Object { ($_ -split "`t")[1] }) -join ' | '))
  return $null
}
function DlgText($hwnd) { return [DayW]::Statics($hwnd).Trim() }
# Message boxes of a setup or uninstaller (any #32770 that is not ours, Explorer's or the job's).
function SetupBoxes() { return @([DayW]::Tops('#32770') | Where-Object { [DayW]::ProcOf($_) -notmatch '^(pecofence|explorer|powershell|notepad|conhost)' }) }

# Inno setup run interactively: language OK, the "currently running" prompt (OnRunning closes the
# app, then OK), Next / Install / Finish. Returns the exit code and the pages seen.
function Wizard($exe, $logName, [scriptblock]$OnRunning) {
  $seen = New-Object System.Collections.ArrayList
  $p = Start-Process $exe -ArgumentList "/LOG=`"$PfRun\$logName`"" -PassThru
  $t0 = Get-Date
  while (-not $p.HasExited -and ((Get-Date) - $t0).TotalSeconds -lt 120) {
    Start-Sleep -Milliseconds 800
    $lang = [G]::FindWindowW('TSelectLanguageForm', [NullString]::Value)
    if ($lang -ne [IntPtr]::Zero -and [G]::IsWindowVisible($lang)) { [void]$seen.Add("language $(ClickButton $lang '^OK$')"); continue }
    $boxes = @(SetupBoxes)
    if ($boxes.Count) {
      $txt = DlgText $boxes[0]
      if ($txt -match 'currently running') {
        [void]$seen.Add('app running prompt'); Shot "$logName-running"
        try { & $OnRunning } catch { Log "closing the app failed: $($_.Exception.Message)" }
        Start-Sleep -Seconds 1; [void](ClickButton $boxes[0] '^OK$'); continue
      }
      [void]$seen.Add("box '$txt' $(ClickButton $boxes[0] '^(OK|Yes)$')"); continue
    }
    $wiz = [G]::FindWindowW('TWizardForm', [NullString]::Value)
    if ($wiz -ne [IntPtr]::Zero -and [G]::IsWindowVisible($wiz)) {
      $n = ClickButton $wiz '^(Next|Install|Finish)'
      if ($n) { [void]$seen.Add($n); Start-Sleep -Milliseconds 700 }
    }
  }
  $p.WaitForExit(5000) | Out-Null
  return [pscustomobject]@{ code = $p.ExitCode; exited = $p.HasExited; seen = ($seen -join ' > ') }
}
# The Inno uninstaller run interactively (it hands over to a copy in %TEMP% and exits at once).
function Uninstall($unins, [scriptblock]$OnRunning) {
  $seen = New-Object System.Collections.ArrayList
  Start-Process $unins
  $t0 = Get-Date; $doneAt = $null
  while (((Get-Date) - $t0).TotalSeconds -lt 150) {
    Start-Sleep -Milliseconds 800
    $boxes = @(SetupBoxes)
    if ($boxes.Count) {
      $txt = DlgText $boxes[0]
      if ($txt -match 'currently running') { [void]$seen.Add('app running prompt'); Shot 'uninstall-running'; try { & $OnRunning } catch { Log "closing the app failed: $($_.Exception.Message)" }; Start-Sleep -Seconds 1; [void](ClickButton $boxes[0] '^OK$'); continue }
      if ($txt -match 'sure') { [void]$seen.Add("confirm $(ClickButton $boxes[0] '^Yes$')"); continue }
      [void]$seen.Add("box '$txt' $(ClickButton $boxes[0] '^OK$')")
      if ($txt -match 'removed') { $doneAt = Get-Date }
      continue
    }
    if ($doneAt -and ((Get-Date) - $doneAt).TotalSeconds -gt 3) { break }
  }
  return ($seen -join ' > ')
}
function SilentUninstall($unins) {
  Start-Process $unins -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART' -Wait
  for ($i = 0; $i -lt 60 -and (Test-Path (Join-Path (Split-Path $unins) 'pecofence.exe')); $i++) { Start-Sleep -Milliseconds 500 }
}
function SilentSetup($exe, $logName) {
  $p = Start-Process $exe -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', "/LOG=$PfRun\$logName" -PassThru
  $p.WaitForExit(180000) | Out-Null
  return $p.ExitCode
}

# The tray icon (on the taskbar or under Show Hidden Icons) and its menu, with the real mouse.
# A UI Automation element with a real rectangle on screen (hidden tray icons have NaN rects).
function OnScreen($el) { $r = $el.Current.BoundingRectangle; return ((-not $el.Current.IsOffscreen) -and $r.Width -gt 0 -and $r.Height -gt 0 -and -not [double]::IsNaN($r.X)) }
function TrayIconRect() {
  $any = [Windows.Automation.Condition]::TrueCondition
  for ($try = 0; $try -lt 3; $try++) {
    $tray = [Windows.Automation.AutomationElement]::FromHandle([G]::FindWindowW('Shell_TrayWnd', [NullString]::Value))
    $all = @($tray.FindAll([Windows.Automation.TreeScope]::Descendants, $any))
    $pf = $all | Where-Object { $_.Current.Name -match '^PecoFence' -and $_.Current.Name -notmatch 'running window' -and (OnScreen $_) } | Select-Object -First 1
    if ($pf) { return $pf.Current.BoundingRectangle }
    $ov = [G]::FindWindowW('TopLevelWindowForOverflowXamlIsland', [NullString]::Value)
    if ($ov -eq [IntPtr]::Zero -or -not [G]::IsWindowVisible($ov)) {
      $chev = $all | Where-Object { $_.Current.Name -match 'hidden icons|chevron' -and (OnScreen $_) } | Select-Object -First 1
      if ($chev) {
        $r = $chev.Current.BoundingRectangle
        [G]::Click([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2))
        $ov = WaitWin 'TopLevelWindowForOverflowXamlIsland' 3
      }
    }
    if ($ov -ne [IntPtr]::Zero -and [G]::IsWindowVisible($ov)) {
      $o = @([Windows.Automation.AutomationElement]::FromHandle($ov).FindAll([Windows.Automation.TreeScope]::Descendants, $any))
      $pf = $o | Where-Object { $_.Current.Name -match '^PecoFence' -and (OnScreen $_) } | Select-Object -First 1
      if ($pf) { return $pf.Current.BoundingRectangle }
    }
    Start-Sleep -Seconds 1
  }
  Log ("tray: no PecoFence icon; taskbar names: " + (($all | Where-Object { $_.Current.Name } | ForEach-Object { "$($_.Current.Name) [$($_.Current.ControlType.ProgrammaticName)]" }) -join ' | '))
  return $null
}
function TrayMenu() {
  [G]::MoveTo(700, 450); Start-Sleep -Milliseconds 200
  $r = TrayIconRect
  if (-not $r) { return [IntPtr]::Zero }
  [G]::RClick([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2))
  return (MenuWin 5)
}
function TrayPick($regex) {
  $m = TrayMenu
  if ($m -eq [IntPtr]::Zero) { Log "tray menu did not open for '$regex'"; return $null }
  Start-Sleep -Milliseconds 300
  $script:LastTrayMenu = MenuTexts $m
  $t = MenuPick $regex $m
  Start-Sleep -Milliseconds 600
  return $t
}

# The welcome notice's close box (x, 24 DIP in from the top right corner).
function CloseNotice() {
  $n = [G]::FindWindowW('PecoFence.Notice', [NullString]::Value)
  if ($n -eq [IntPtr]::Zero -or -not [G]::IsWindowVisible($n)) { return $true }
  $r = Rect $n
  [G]::Click($r.R - 24, $r.T + 24); Start-Sleep -Milliseconds 300
  [G]::MoveTo(700, 450)
  return (WaitGoneWin 'PecoFence.Notice' 3)
}

# Real mouse: double-click, drag with the button held.
function DblClick($x, $y) {
  [G]::MoveTo($x, $y); Start-Sleep -Milliseconds 80
  [G]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 30; [G]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero)
  Start-Sleep -Milliseconds 90
  [G]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 30; [G]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero)
}
function DragTo($x1, $y1, $x2, $y2, $steps = 15) {
  [G]::MoveTo($x1, $y1); Start-Sleep -Milliseconds 150
  [G]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 200
  for ($s = 1; $s -le $steps; $s++) {
    [G]::MoveTo([int]($x1 + ($x2 - $x1) * $s / $steps), [int]($y1 + ($y2 - $y1) * $s / $steps)); Start-Sleep -Milliseconds 40
  }
  Start-Sleep -Milliseconds 400
  [G]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 800
}

# Where an item's icon is on screen: icons layout at 100 %, sorted by name (namespace items
# first), 36 px title row, cells 80 x 96 at icon size 48, from `fence get` (windowRect, fit).
function ItemPt($fence, $name) {
  $f = IpfJson fence get $fence
  $items = @(IpfJson item list --fence $fence)
  $order = @(@($items | Where-Object { $_.kind -eq 'namespace' } | Sort-Object name) + @($items | Where-Object { $_.kind -ne 'namespace' } | Sort-Object name) | ForEach-Object { $_.name })
  $i = [array]::IndexOf($order, $name)
  if ($i -lt 0 -or -not $f.windowRect) { Log "ItemPt: '$name' not in '$fence' ($($order -join ', '))"; return $null }
  $wr = $f.windowRect
  $cols = if ($f.fit -and $f.fit.columns) { [int]$f.fit.columns } else { [math]::Max(1, [math]::Floor($wr.w / 80)) }
  $pitch = $wr.w / $cols
  $x = [int]($wr.x + ($i % $cols) * $pitch + $pitch / 2)
  $y = [int]($wr.y + 36 + 8 + [math]::Floor($i / $cols) * 96 + 32)
  Log "ItemPt '$name' in '$fence': index $i of $($order.Count), cols $cols -> $x,$y"
  return @($x, $y)
}
function RClickItem($fence, $name) {
  $p = ItemPt $fence $name
  if (-not $p) { return [IntPtr]::Zero }
  [G]::RClick($p[0], $p[1])
  return (MenuWin 8)
}
function ItemFence($name) { $it = @(IpfJson item list) | Where-Object { $_.name -eq $name } | Select-Object -First 1; if ($it) { return $it.fenceTitle }; return $null }
function VisFences() { return @([DayW]::Tops('PecoFence.Fence')).Count }
function BinNames() { return ((@((New-Object -ComObject Shell.Application).NameSpace(10).Items()) | ForEach-Object { $_.Name }) -join ', ') }
# Fences, rules, item memberships and the theme as sorted text lines (compared across an upgrade).
function StateText() {
  $lines = @()
  foreach ($f in @(IpfJson fence list) | Sort-Object title) { $r = $f.rect; $lines += "fence $($f.title) | $($r.x),$($r.y),$($r.w),$($r.h) | tint $($f.tint)" }
  foreach ($r in @((IpfJson rule list).list)) { $lines += "rule $($r.name) -> $($r.targetTitle) enabled=$($r.enabled)" }
  foreach ($it in @(IpfJson item list) | Sort-Object name) { $lines += "item $($it.name) in $($it.fenceTitle)" }
  $lines += "theme $((Ipf settings get theme).Trim())"
  return $lines
}
function LogTime($line) { if ($line -match '^(\S+Z)\s') { return [DateTimeOffset]::Parse($Matches[1]).UtcDateTime }; return $null }

# ---- steps

Case 'setup'
CloseWindows
# Installers or uninstallers an aborted earlier job left open (ctl reset does not end them).
Get-Process | Where-Object { $_.ProcessName -match 'setup|^_iu|^unins' } | Stop-Process -Force -ErrorAction SilentlyContinue
# A browser an earlier job opened (the notice's Manual, an EC2 .url) covers the desktop.
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1
$prog = "$env:LOCALAPPDATA\Programs\PecoFence"
$exe = "$prog\pecofence.exe"
$lnk = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\PecoFence\PecoFence.lnk"
$runKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$script:CliExe = "$prog\pecofence-cli.exe"
if (Test-Path "$prog\unins000.exe") { SilentUninstall "$prog\unins000.exe" }
Remove-Item -Recurse -Force "$env:APPDATA\PecoFence", "$env:APPDATA\OpenFence", "$env:LOCALAPPDATA\PecoFence" -ErrorAction SilentlyContinue
Remove-ItemProperty $runKey -Name PecoFence -ErrorAction SilentlyContinue
$testFiles = @('Alpha Folder', 'Beta notes.txt', 'Beta notes - Copy.txt', 'Gamma link.lnk', 'Gamma renamed.lnk', 'Report 2026.pdf', 'Delta temp.txt', 'Epsilon temp.txt', 'Zeta drag.txt', 'Eta drag.txt')
foreach ($n in $testFiles) { Remove-Item -LiteralPath "$PfDesk\$n" -Recurse -Force -ErrorAction SilentlyContinue }
Clear-RecycleBin -Force -ErrorAction SilentlyContinue
if (-not [DayW]::IconsVisible()) { [DayW]::ToggleIcons(); Start-Sleep -Seconds 1 }
# This PC on the desktop (Desktop icon settings), so it shows in the Desktop fence.
$hdi = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\HideDesktopIcons\NewStartPanel'
if (-not (Test-Path $hdi)) { New-Item $hdi -Force | Out-Null }
Set-ItemProperty $hdi -Name '{20D04FE0-3AEA-1069-A2D8-08002B30309D}' -Value 0 -Type DWord
New-Item -ItemType Directory -Force "$PfDesk\Alpha Folder" | Out-Null
Set-Content "$PfDesk\Alpha Folder\inside.txt" 'inside'
foreach ($n in 'Beta notes.txt', 'Delta temp.txt', 'Epsilon temp.txt', 'Zeta drag.txt', 'Eta drag.txt') { Set-Content "$PfDesk\$n" $n }
Set-Content "$PfDesk\Report 2026.pdf" '%PDF-1.4 test'
$sc = (New-Object -ComObject WScript.Shell).CreateShortcut("$PfDesk\Gamma link.lnk"); $sc.TargetPath = "$env:WINDIR\notepad.exe"; $sc.Save()
$bare = [DayW]::RootClassAt(1300, 650)
Check 'the setups, the zip and the desktop items are there, the desktop is bare' ((Test-Path "$PfExtra\old-setup.exe") -and (Test-Path "$PfExtra\new-setup.exe") -and (Test-Path "$PfExtra\portable.zip") -and (Test-Path "$PfDesk\Gamma link.lnk") -and -not (Test-Path $exe) -and $bare -match '^(Progman|WorkerW)$') "icons shown=$([DayW]::IconsVisible()) at 1300,650: $bare"

Case 'D1 install 0.2.5 and use it'
$code = SilentSetup "$PfExtra\old-setup.exe" 'setup-old.log'
Check 'the 0.2.5 setup succeeds' ($code -eq 0 -and (Test-Path $exe)) "exit=$code"
$oldHash = (Get-FileHash $exe).Hash
$h = StartUser $exe
$st = IpfJson status
Check '0.2.5 runs' ($h -ne [IntPtr]::Zero -and $st.appVersion) "version=$($st.appVersion) config=$($st.configPath)"
$fs = @(IpfJson fence list)
Check 'its first run makes fences' ($fs.Count -ge 3) (($fs | ForEach-Object { $_.title }) -join ', ')
$other = $fs | Where-Object { $_.kind -ne 'inbox' } | Select-Object -First 1
Ipf fence rename $other.id 'Renamed in 0.2.5' | Out-Null
Ipf fence create --title Work --rect '700,300,340,250' | Out-Null
Ipf fence set Work tint '#3366ff' | Out-Null
Ipf rule add --name 'PDF to Work' --ext pdf --to Work --index 0 | Out-Null
Ipf rule apply | Out-Null
Ipf item move "$PfDesk\Beta notes.txt" "$PfDesk\Alpha Folder" --to Work | Out-Null
Ipf settings set theme dark | Out-Null
Start-Sleep -Seconds 2
$before = StateText
Log ("state before the upgrade:`n" + ($before -join "`n"))
Check 'the fence, the rule and the moved icons took' ((@($before -match '^fence Work ').Count -eq 1) -and (@($before -match '^item Report 2026 in Work$').Count -eq 1) -and (@($before -match '^item Alpha Folder in Work$').Count -eq 1) -and (@($before -match '^rule PDF to Work -> Work').Count -eq 1)) (($before -match 'Work') -join ' ; ')
Shot 'd1-0.2.5'

Case 'D1 upgrade with the new setup while 0.2.5 runs'
CloseNotice | Out-Null
$script:quitBy = $null; $script:quitGone = $false
$w = Wizard "$PfExtra\new-setup.exe" 'setup-upgrade.log' { $script:quitBy = TrayPick '^Exit PecoFence'; $script:quitGone = WaitNoPf 15 }
Check 'the wizard runs to the end' ($w.exited -and $w.code -eq 0) "exit=$($w.code) pages: $($w.seen)"
Check 'it asks to close PecoFence, which quits from its tray menu' (($w.seen -match 'app running prompt') -and $quitBy -and $quitGone) "tray pick='$quitBy' gone=$quitGone menu: $LastTrayMenu"
$h = WaitWin 'PecoFence.Control' 30 -Hidden
Start-Sleep -Seconds 5
$newHash = (Get-FileHash $exe).Hash
$tools = Ipf settings get tools
Check 'Finish starts the new build' ($h -ne [IntPtr]::Zero -and $newHash -ne $oldHash -and $tools -match 'dialogJump') "hash $($oldHash.Substring(0, 8)) -> $($newHash.Substring(0, 8)); tools=$($tools.Trim())"
$after = StateText
$diff = @(Compare-Object $before $after | ForEach-Object { "$($_.SideIndicator) $($_.InputObject)" })
Check 'fences, rules, icons and settings are the same after the upgrade' ($diff.Count -eq 0) ($diff -join ' ; ')
Shot 'd1-upgraded'

Case 'D2 fresh install on a clean profile'
PfQuit
SilentUninstall "$prog\unins000.exe"
Remove-Item -Recurse -Force "$env:APPDATA\PecoFence", "$env:APPDATA\OpenFence", "$env:LOCALAPPDATA\PecoFence" -ErrorAction SilentlyContinue
if (-not [DayW]::IconsVisible()) { [DayW]::ToggleIcons(); Start-Sleep -Seconds 1 }
Check 'a clean profile' (-not (Test-Path $exe) -and -not (Test-Path "$env:APPDATA\PecoFence"))
$w = Wizard "$PfExtra\new-setup.exe" 'setup-fresh.log' { Log 'unexpected running prompt' }
Check 'the wizard runs to the end' ($w.exited -and $w.code -eq 0) "exit=$($w.code) pages: $($w.seen)"
$h = WaitWin 'PecoFence.Control' 30 -Hidden
$notice = WaitWin 'PecoFence.Notice' 20
Check 'the first run shows the welcome notice' ($notice -ne [IntPtr]::Zero) $(if ($notice -ne [IntPtr]::Zero) { [DayW]::Info($notice) })
Shot 'd2-welcome'
Check 'its close box closes it' (CloseNotice)
Start-Sleep -Seconds 2
$fs = @(IpfJson fence list)
$items = @(IpfJson item list)
$alpha = $items | Where-Object { $_.name -eq 'Alpha Folder' }
Check 'the first run sorts the desktop into fences' ($fs.Count -ge 3 -and $alpha.assignedBy -eq 'rule') ("fences: " + (($fs | ForEach-Object { $_.title }) -join ', ') + "; Alpha Folder in $($alpha.fenceTitle) by $($alpha.assignedBy)")
Check 'the Start menu shortcut and the startup entry are there' ((Test-Path $lnk) -and (Get-ItemProperty $runKey -ErrorAction SilentlyContinue).PecoFence) "run=$((Get-ItemProperty $runKey -ErrorAction SilentlyContinue).PecoFence)"

Case 'prepare the day'
Ipf fence create --title Work --rect '370,290,330,250' | Out-Null
Ipf fence create --title Drag --rect '720,290,330,250' | Out-Null
Ipf item move "$PfDesk\Alpha Folder" "$PfDesk\Beta notes.txt" "$PfDesk\Delta temp.txt" "$PfDesk\Epsilon temp.txt" "$PfDesk\Gamma link.lnk" --to Work | Out-Null
Ipf item move "$PfDesk\Eta drag.txt" "$PfDesk\Zeta drag.txt" --to Drag | Out-Null
Ipf fence set --all sort name | Out-Null
Ipf fence set inbox desktopLayout false | Out-Null
$slots = @('20,20,330,250', '370,20,330,250', '720,20,330,250', '20,290,330,250', '20,560,330,250', '370,560,330,250', '720,560,330,250')
$k = 0
foreach ($f in @(IpfJson fence list)) { if ($f.title -in 'Work', 'Drag') { continue }; Ipf fence move $f.id --rect $slots[$k] | Out-Null; $k++ }
Start-Sleep -Seconds 2
$fs = @(IpfJson fence list)
Log ("fences: " + (($fs | ForEach-Object { "$($_.title) $($_.windowRect.x),$($_.windowRect.y),$($_.windowRect.w),$($_.windowRect.h) items=$($_.itemCount)" }) -join ' ; '))
$work = @(IpfJson item list --fence Work | ForEach-Object { $_.name }) -join ', '
$inbox = @(IpfJson item list --fence inbox | ForEach-Object { $_.name }) -join ', '
Check 'Work holds the five test items, the Desktop fence This PC and the Recycle Bin' (($work -split ', ').Count -eq 5 -and $inbox -match 'Recycle Bin' -and $inbox -match 'This PC') "Work: $work; Desktop: $inbox"
Shot 'day-0'

Case 'D4 right-click a folder right after start'
$pt = ItemPt 'Work' 'Alpha Folder'
$t = TrayPick '^Exit PecoFence'
Check 'Exit in the tray menu quits it' ($t -and (WaitNoPf 15)) "pick='$t' menu: $LastTrayMenu"
[G]::MoveTo($pt[0], $pt[1])
$clock = [Diagnostics.Stopwatch]::StartNew()
Start-Process explorer.exe "`"$exe`""
$shownAt = -1
for ($i = 0; $i -lt 400; $i++) { if ([DayW]::RootClassAt($pt[0], $pt[1]) -eq 'PecoFence.Fence') { $shownAt = $clock.ElapsedMilliseconds; break }; Start-Sleep -Milliseconds 25 }
$clickUtc = [DateTime]::UtcNow
[G]::RClick($pt[0], $pt[1]); $clickAt = $clock.ElapsedMilliseconds
$m = MenuWin 15; $menuAt = $clock.ElapsedMilliseconds
$texts = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
Shot 'd4-first-menu'
Check 'the first folder right-click opens its menu' ($m -ne [IntPtr]::Zero -and $texts -match '^Open') "fence on screen at $shownAt ms, right-click at $clickAt ms, menu at $menuAt ms after start; items: $texts"
CloseMenus
$sw = [Diagnostics.Stopwatch]::StartNew(); $st = Ipf status; $ms = $sw.ElapsedMilliseconds
Check 'the app answers at once after that menu' ($st -match 'appVersion' -and $ms -lt 5000) "status in $ms ms"
$lines = @(Get-Content (UserLog) -Encoding UTF8)
$start = $lines | Where-Object { $_ -match 'process started' } | Select-Object -First 1
$cache = $lines | Where-Object { $_ -match 'shell handler cache' } | Select-Object -First 1
$t1 = LogTime $start; $t2 = LogTime $cache
$after = if ($t1 -and $t2) { [int]($t2 - $t1).TotalMilliseconds } else { -1 }
$beforeClick = if ($t2) { [int]($clickUtc - $t2).TotalMilliseconds } else { 0 }
Check 'the log has shell handler cache prepared early after start' ($cache -match 'prepared' -and $after -ge 0 -and $after -lt 5000) "$after ms after process started, $beforeClick ms before the right-click: $cache"

Case 'D4 every item menu'
Start-Sleep -Seconds 15
$m = RClickItem 'Work' 'Alpha Folder'; $texts = MenuTexts $m
Check 'a folder, again later' ($texts -match '^Open') $texts
CloseMenus
$m = RClickItem 'Work' 'Beta notes'; $texts = MenuTexts $m
Check 'a file: Open, Send to, Copy as path, Properties' ($texts -match '^Open' -and $texts -match 'Send to' -and $texts -match 'Copy as path' -and $texts -match 'Properties') $texts
$p = MenuPoint $m '^Send to'
if ($p) { [G]::MoveTo($p[0] - 20, $p[1]); Start-Sleep -Milliseconds 300; [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 1500 }
$menus = @([DayW]::Tops('#32768'))
$sub = if ($menus.Count -ge 2) { MenuTexts $menus[0] } else { '' }
Shot 'd4-send-to'
Check 'hovering Send to opens its submenu' ($menus.Count -ge 2 -and $sub -match 'Desktop|Compressed') "menus=$($menus.Count) submenu: $sub"
CloseMenus
$m = RClickItem 'Work' 'Gamma link'; $texts = MenuTexts $m; $dupes = MenuDupes $m
Check 'a shortcut: Open file location' ($texts -match '^Open' -and $texts -match 'Open file location') $texts
Check 'a shortcut: no entry twice in its menu' (-not $dupes) "twice: $dupes"
CloseMenus
$m = RClickItem 'inbox' 'This PC'; $texts = MenuTexts $m
Check 'This PC: Rename, Delete and Open file location greyed' ($texts -match '^Open ~ Enter \|' -and $texts -match 'Rename ~ F2 \(off\)' -and $texts -match 'Delete ~ Delete \(off\)' -and $texts -match 'Open file location \(off\)') $texts
CloseMenus
$m = RClickItem 'inbox' 'Recycle Bin'; $texts = MenuTexts $m
Check 'the empty Recycle Bin: Empty Recycle Bin greyed' ($texts -match '^Open ~ Enter \|' -and $texts -match 'Empty Recycle Bin \(off\)') $texts
CloseMenus
$sw = [Diagnostics.Stopwatch]::StartNew(); $st = Ipf status; $ms = $sw.ElapsedMilliseconds
Check 'the app still answers after the menus' ($st -match 'appVersion' -and $ms -lt 5000) "status in $ms ms"

Case 'D4 menu commands'
$m = RClickItem 'Work' 'Alpha Folder'; $t = MenuPick '^Open' $m
Start-Sleep -Seconds 4
$ex = Explorers
Check 'Open on a folder opens it in Explorer' ($t -and $ex -match 'Alpha Folder') "pick='$t' explorer: $ex"
CloseWindows
$m = RClickItem 'Work' 'Beta notes'; $t = MenuPick '^Properties' $m
$props = [IntPtr]::Zero
for ($i = 0; $i -lt 40 -and $props -eq [IntPtr]::Zero; $i++) { Start-Sleep -Milliseconds 200; $props = @([DayW]::Tops('#32770') | Where-Object { [DayW]::Title($_) -match 'Properties' }) | Select-Object -First 1; if (-not $props) { $props = [IntPtr]::Zero } }
Check 'Properties opens the file properties' ($props -ne [IntPtr]::Zero) "pick='$t' window: $(if ($props -ne [IntPtr]::Zero) { [DayW]::Info($props) })"
Shot 'd4-properties'
if ($props -ne [IntPtr]::Zero) { [DayW]::PostMessageW($props, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; Start-Sleep -Milliseconds 600 }
Set-Clipboard -Value 'empty'
$m = RClickItem 'Work' 'Beta notes'; $t = MenuPick '^Copy as path' $m
Start-Sleep -Milliseconds 800
$clip = (Get-Clipboard -Raw)
Check 'Copy as path puts the quoted path on the clipboard' ($clip -and $clip.Trim() -eq "`"$PfDesk\Beta notes.txt`"") "pick='$t' clipboard=$clip"
$m = RClickItem 'Work' 'Gamma link'; $t = MenuPick '^Rename' $m
$ren = WaitWin 'PecoFence.Rename' 5
Start-Sleep -Milliseconds 300
$fg = Fg
[G]::Type('Gamma renamed'); Start-Sleep -Milliseconds 200; [G]::Press(0x0D); Start-Sleep -Seconds 2
Check 'Rename renames the file on disk' ((Test-Path "$PfDesk\Gamma renamed.lnk") -and -not (Test-Path "$PfDesk\Gamma link.lnk")) "pick='$t' editor=$ren fg=$fg"
$m = RClickItem 'Work' 'Delta temp'; $t = MenuPick '^Delete' $m
Start-Sleep -Seconds 1
$boxes = @(SetupBoxes); if ($boxes.Count) { Log "delete asked: $(DlgText $boxes[0])"; ClickButton $boxes[0] '^Yes$' | Out-Null }
Start-Sleep -Seconds 2
Check 'Delete moves the file to the Recycle Bin' (-not (Test-Path "$PfDesk\Delta temp.txt") -and (BinNames) -match 'Delta temp') "pick='$t' bin: $(BinNames)"
$p = ItemPt 'Work' 'Epsilon temp'
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
$fg = Fg
[G]::Press(0x2E); Start-Sleep -Seconds 1
$boxes = @(SetupBoxes); if ($boxes.Count) { Log "delete asked: $(DlgText $boxes[0])"; ClickButton $boxes[0] '^Yes$' | Out-Null }
Start-Sleep -Seconds 2
Check 'the Delete key deletes the selected icon' (-not (Test-Path "$PfDesk\Epsilon temp.txt")) "fg=$fg bin: $(BinNames)"
$m = RClickItem 'inbox' 'Recycle Bin'; $texts = MenuTexts $m
Check 'with deleted files in it, the Recycle Bin offers Empty Recycle Bin' ($texts -match 'Empty Recycle Bin \|') $texts
CloseMenus
$p = ItemPt 'Work' 'Beta notes'
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
Chord 0x11 0x43; Start-Sleep -Milliseconds 600
$drop = @(Get-Clipboard -Format FileDropList -ErrorAction SilentlyContinue | ForEach-Object { $_.FullName })
Chord 0x11 0x56; Start-Sleep -Seconds 3
$copyIn = ItemFence 'Beta notes - Copy'
Check 'Ctrl+C copies the file, Ctrl+V pastes a copy (filed by the rules)' (($drop -join ',') -eq "$PfDesk\Beta notes.txt" -and (Test-Path "$PfDesk\Beta notes - Copy.txt") -and $copyIn) "clipboard=$($drop -join ',') copy in '$copyIn' fg=$(Fg)"
$hang = @(Get-Content (UserLog) -Encoding UTF8 | Where-Object { $_ -match 'UI thread not responding' })
Check 'no UI thread not responding in the log, app alive' ($hang.Count -eq 0 -and (Get-Process pecofence -ErrorAction SilentlyContinue)) (($hang | Select-Object -First 2) -join ' | ')

Case 'D6 double-click the desktop'
CloseWindows
$free = @(1300, 650)
[G]::Click($free[0], $free[1]); Start-Sleep -Milliseconds 1200
$n0 = VisFences
DblClick $free[0] $free[1]; Start-Sleep -Seconds 2
$n1 = VisFences
DblClick $free[0] $free[1]; Start-Sleep -Seconds 2
$n2 = VisFences
Check 'a double-click on the empty desktop hides the fences, another shows them' ($n0 -gt 0 -and $n1 -eq 0 -and $n2 -eq $n0) "visible fences: $n0 -> $n1 -> $n2; at the point: $([DayW]::RootClassAt($free[0], $free[1]))"

Case 'D6 marquee: new fence here'
$c0 = @(IpfJson fence list).Count
[G]::Click(1300, 650); Start-Sleep -Milliseconds 1000
DragTo 1150 120 1450 360
$m = MenuWin 5
$texts = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
Shot 'd6-marquee'
Check 'a rectangle on the empty desktop offers Create a fence here' ($texts -match 'Create a fence here') "menu: $texts"
$t = MenuPick 'Create a fence here' $m
$ren = WaitWin 'PecoFence.Rename' 5
Start-Sleep -Milliseconds 300
[G]::Type('Marquee fence'); [G]::Press(0x0D); Start-Sleep -Seconds 2
$mf = IpfJson fence get 'Marquee fence'
$r = $mf.rect
Check 'the new fence is there, titled as typed, where the rectangle was' ($mf.title -eq 'Marquee fence' -and [math]::Abs($r.x - 1150) -le 40 -and [math]::Abs($r.y - 120) -le 40 -and @(IpfJson fence list).Count -eq $c0 + 1) "rename editor=$ren rect=$($r.x),$($r.y),$($r.w),$($r.h)"

Case 'D6 Peek hotkey'
Start-Process notepad.exe; Start-Sleep -Seconds 2
$np = [G]::FindWindowW('Notepad', [NullString]::Value)
[L]::MoveWindow($np, 40, 30, 1100, 780, $true) | Out-Null; Start-Sleep -Milliseconds 500
TitleClick $np
PeekKey; Start-Sleep -Milliseconds 1500
$dim = [G]::FindWindowW('PecoFence.PeekDimmer', [NullString]::Value)
$on = ($dim -ne [IntPtr]::Zero -and [G]::IsWindowVisible($dim)); $box = [M]::BoxText()
Shot 'd6-peek'
Check 'Ctrl+Alt+Space brings the fences above Notepad' ($on -or $null -ne $box) "dimmer visible=$on search box='$box' fg=$(Fg)"
for ($i = 0; $i -lt 3 -and (([G]::IsWindowVisible($dim)) -or $null -ne [M]::BoxText()); $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 600 }
Check 'Esc ends the Peek' (-not [G]::IsWindowVisible($dim) -and $null -eq [M]::BoxText()) "fg=$(Fg)"
CloseWindows

Case 'D6 drag icons'
$p = ItemPt 'Drag' 'Eta drag'
$wk = IpfJson fence get Work
$to = @(($wk.windowRect.x + $wk.windowRect.w - 50), ($wk.windowRect.y + $wk.windowRect.h - 40))
DragTo $p[0] $p[1] $to[0] $to[1]
Start-Sleep -Seconds 1
$in = ItemFence 'Eta drag'
Check 'an icon dragged into another fence lands there' ($in -eq 'Work') "Eta drag is in '$in'"
$p = ItemPt 'Drag' 'Zeta drag'
DragTo $p[0] $p[1] 1300 780
Start-Sleep -Seconds 1
$in = ItemFence 'Zeta drag'
Check 'an icon dragged onto the bare desktop goes back to the Desktop fence, the file stays' ($in -eq (IpfJson fence get inbox).title -and (Test-Path "$PfDesk\Zeta drag.txt")) "Zeta drag is in '$in'"
Shot 'd6-dragged'

Case 'D6 Desktop fence arranged like the desktop'
$ib = IpfJson fence get inbox
[G]::RClick($ib.windowRect.x + 90, $ib.windowRect.y + 18)
$m = MenuWin 5; $texts = MenuTexts $m
$t = MenuPick 'Arrange like the desktop' $m
Start-Sleep -Seconds 2
$ib2 = IpfJson fence get inbox
$wr = $ib2.windowRect
Shot 'd6-arranged'
Check 'Arrange like the desktop in the Desktop fence menu' ($t -and $ib2.desktopLayout -eq $true) "pick='$t' windowRect=$($wr.x),$($wr.y),$($wr.w),$($wr.h) menu: $texts"
[G]::Click($free[0], $free[1]); Start-Sleep -Milliseconds 1200
$n0 = VisFences
DblClick $free[0] $free[1]; Start-Sleep -Seconds 2
$n1 = VisFences
DblClick $free[0] $free[1]; Start-Sleep -Seconds 2
$n2 = VisFences
Check 'the desktop double-click still hides and shows with the arranged Desktop fence' ($n0 -gt 0 -and $n1 -eq 0 -and $n2 -eq $n0) "visible fences: $n0 -> $n1 -> $n2"

Case 'D6 tray menu'
$t = TrayPick '^Hide all fences'; Start-Sleep -Seconds 1
$n1 = VisFences
$t2 = TrayPick '^Show all fences'; Start-Sleep -Seconds 1
$n2 = VisFences
Check 'Hide all fences and Show all fences in the tray menu' ($t -and $t2 -and $n1 -eq 0 -and $n2 -gt 0) "picks '$t' / '$t2'; visible $n1 -> $n2; menu: $LastTrayMenu"

Case 'D6 settings'
$t = TrayPick '^Settings'
$sh = WaitWin 'PecoFence.Settings' 20
Check 'Settings opens from the tray menu' ($sh -ne [IntPtr]::Zero) "pick='$t'"
[L]::MoveWindow($sh, 150, 20, 1200, 840, $true) | Out-Null; Start-Sleep -Seconds 3
Shot 'd6-settings'
$s0 = IpfJson settings get
# The other theme style tile, the colour mode list (Dark), the Chameleon switch.
$tile = if ($s0.themeStyle -eq 'fluent') { @(1090, 270) } else { @(680, 270) }
[G]::Click($tile[0], $tile[1]); Start-Sleep -Seconds 2
$s1 = IpfJson settings get
Check 'a click on the other theme style tile switches the style' ($s1.themeStyle -ne $s0.themeStyle) "themeStyle $($s0.themeStyle) -> $($s1.themeStyle)"
Shot 'd6-settings-style'
[G]::Click(1188, 513); Start-Sleep -Milliseconds 800
[G]::Press(0x23); Start-Sleep -Milliseconds 300; [G]::Press(0x0D); Start-Sleep -Seconds 2
$s2 = IpfJson settings get
Check 'the Color mode list picks Dark' ($s2.theme -eq 'dark') "theme $($s1.theme) -> $($s2.theme)"
[G]::Click(1192, 667); Start-Sleep -Milliseconds 800
[G]::Press(0x28); Start-Sleep -Milliseconds 300; [G]::Press(0x0D); Start-Sleep -Seconds 2
$s3 = IpfJson settings get
Check 'the Title alignment list picks Center' ($s3.titleAlign -eq 'center') "titleAlign $($s2.titleAlign) -> $($s3.titleAlign)"
Shot 'd6-settings-changed'
$sr = Rect $sh
[G]::Click($sr.R - 23, $sr.T + 16); Start-Sleep -Seconds 1
Check 'its close box closes Settings' (WaitGoneWin 'PecoFence.Settings' 5) ''

Case 'D6 log and exit'
$lines = @(Get-Content (UserLog) -Encoding UTF8)
Copy-Item (UserLog) "$PfRun\day-pecofence.log" -ErrorAction SilentlyContinue
$errs = @($lines | Where-Object { $_ -match ' ERROR |panicked' })
$warns = @($lines | Where-Object { $_ -match ' WARN ' } | ForEach-Object { ($_ -replace '^\S+\s+WARN\s+', '') } | Select-Object -Unique)
Check 'the log of the day has no ERROR' ($errs.Count -eq 0) (($errs | Select-Object -First 3) -join ' | ')
Check 'the log of the day has no WARN' ($warns.Count -eq 0) (($warns | Select-Object -First 6) -join ' | ')
$t = TrayPick '^Exit PecoFence'
$gone = WaitNoPf 15
Start-Sleep -Seconds 1
Check 'Exit quits it and the Windows desktop icons come back' ($t -and $gone -and [DayW]::IconsVisible()) "pick='$t' gone=$gone icons=$([DayW]::IconsVisible())"

Case 'D2 uninstall while it runs'
$h = StartUser $exe
$style = (Ipf settings get themeStyle).Trim()
Check 'the settings changed in the day stay after a restart' ($style -eq "`"$($s1.themeStyle)`"") "themeStyle=$style"
$seen = Uninstall "$prog\unins000.exe" { $script:quitBy = TrayPick '^Exit PecoFence'; $script:quitGone = WaitNoPf 15 }
Start-Sleep -Seconds 2
Check 'the uninstaller asks to close PecoFence, then removes it' ($seen -match 'app running prompt' -and $seen -match 'removed') "seen: $seen; tray pick='$quitBy' gone=$quitGone"
Check 'no PecoFence process is left' (-not (Get-Process pecofence, pecofence-watchdog -ErrorAction SilentlyContinue))
Check 'no program folder is left' (-not (Test-Path $prog)) $(if (Test-Path $prog) { (Get-ChildItem $prog -Recurse | ForEach-Object { $_.Name }) -join ', ' })
Check 'the Start menu shortcut and the startup entry are gone' (-not (Test-Path $lnk) -and -not (Get-ItemProperty $runKey -ErrorAction SilentlyContinue).PecoFence)
Check 'the Windows desktop icons are shown' ([DayW]::IconsVisible())

Case 'D3 portable zip'
$pdir = "$PfRun\portable"
Expand-Archive "$PfExtra\portable.zip" $pdir -Force
$appCfg = "$env:APPDATA\PecoFence\config.json"
$cfgBefore = if (Test-Path $appCfg) { (Get-FileHash $appCfg).Hash } else { 'none' }
Start-Process "$pdir\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $pdir
$h = WaitWin 'PecoFence.Control' 30 -Hidden
Start-Sleep -Seconds 5
$script:CliExe = "$pdir\pecofence-cli.exe"
$st = IpfJson status
Check 'the zip runs from its folder with --portable, config beside the exe' ($h -ne [IntPtr]::Zero -and $st.configPath -like "$pdir\config\*" -and (Test-Path "$pdir\config\config.json")) "config=$($st.configPath) version=$($st.appVersion)"
$fs = @(IpfJson fence list)
Check 'it sorts the desktop into fences' ($fs.Count -ge 3) (($fs | ForEach-Object { $_.title }) -join ', ')
$cfgAfter = if (Test-Path $appCfg) { (Get-FileHash $appCfg).Hash } else { 'none' }
$runVals = (Get-ItemProperty $runKey -ErrorAction SilentlyContinue).PSObject.Properties | Where-Object { "$($_.Value)" -like '*pecofence*' } | ForEach-Object { "$($_.Name)=$($_.Value)" }
Check 'it writes no startup entry and leaves the installed config alone' (-not $runVals -and $cfgAfter -eq $cfgBefore) "run: $runVals; appdata config $($cfgBefore.Substring(0, 4)) -> $($cfgAfter.Substring(0, 4))"
Shot 'd3-portable'
CloseNotice | Out-Null
$t = TrayPick '^Exit PecoFence'
Check 'it quits from its tray menu' ($t -and (WaitNoPf 15) -and [DayW]::IconsVisible()) "pick='$t'"
$plines = @(Get-Content (UserLog) -Encoding UTF8)
Copy-Item (UserLog) "$PfRun\portable-pecofence.log" -ErrorAction SilentlyContinue
$errs = @($plines | Where-Object { $_ -match ' ERROR |panicked' })
Check 'the portable log has no ERROR' ($errs.Count -eq 0) (($errs | Select-Object -First 3) -join ' | ')

Case 'D5 restart at sign-in'
Log 'skipped: the box cannot reboot or sign in again during a job'

Case 'cleanup'
CloseWindows
foreach ($n in $testFiles) { Remove-Item -LiteralPath "$PfDesk\$n" -Recurse -Force -ErrorAction SilentlyContinue }
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force
Remove-ItemProperty $hdi -Name '{20D04FE0-3AEA-1069-A2D8-08002B30309D}' -ErrorAction SilentlyContinue
Clear-RecycleBin -Force -ErrorAction SilentlyContinue
