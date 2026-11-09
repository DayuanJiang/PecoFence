# scripts/wintest/vdesk.ps1: Windows virtual desktops for the steps (loaded by lib.ps1).
# Explorer keeps them in HKCU\...\Explorer\VirtualDesktops: CurrentVirtualDesktop (Windows 10:
# under SessionInfo\<session>\VirtualDesktops) and VirtualDesktopIDs (16 bytes each, Task View
# order) appear with the second desktop; Desktops\{id}\Name only after a rename in Task View.
# Writing Name ourselves does not reach Task View (Explorer caches it), so VdRename renames
# there. A desktop left by an earlier job keeps its name: name the ones a test relies on.

Add-Type -TypeDefinition @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class VD {
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowTextW(IntPtr h, StringBuilder s, int n);
  [DllImport("kernel32.dll")] static extern bool ProcessIdToSessionId(uint pid, out uint session);
  [DllImport("kernel32.dll")] static extern uint GetCurrentProcessId();
  public static bool Visible(IntPtr h) { return IsWindowVisible(h); }
  public static uint Session() { uint s; ProcessIdToSessionId(GetCurrentProcessId(), out s); return s; }
  // A top-level window of a class by its title, shown or hidden (PecoFence.Fence: a fence).
  public static IntPtr Find(string cls, string title) {
    IntPtr found = IntPtr.Zero;
    EnumWindows((h, l) => {
      var c = new StringBuilder(256); GetClassNameW(h, c, 256);
      if (c.ToString() != cls) return true;
      var t = new StringBuilder(512); GetWindowTextW(h, t, 512);
      if (title == null || t.ToString() == title) { found = h; return false; }
      return true;
    }, IntPtr.Zero);
    return found;
  }
  public static IntPtr Fence(string title) { return Find("PecoFence.Fence", title); }
}
'@

$VdKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\VirtualDesktops'
function GuidOf([byte[]]$b, $at = 0) { if (-not $b -or $b.Length -lt $at + 16) { return $null }; return ([Guid]::new([byte[]]$b[$at..($at + 15)])).ToString() }
function VdCurrent() {
  $p = Get-ItemProperty $VdKey -ErrorAction SilentlyContinue
  if (-not $p.CurrentVirtualDesktop) { $p = Get-ItemProperty "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\SessionInfo\$([VD]::Session())\VirtualDesktops" -ErrorAction SilentlyContinue }
  return (GuidOf $p.CurrentVirtualDesktop)
}
function VdIds() {
  $b = (Get-ItemProperty $VdKey -ErrorAction SilentlyContinue).VirtualDesktopIDs; $out = @()
  if ($b) { for ($i = 0; $i + 16 -le $b.Length; $i += 16) { $out += (GuidOf $b $i) } }
  return $out
}
function VdName($id) { return (Get-ItemProperty "$VdKey\Desktops\{$id}" -ErrorAction SilentlyContinue).Name }
# "current=<id> count=N <id>='<name>' ..." for the log.
function VdState() {
  $ids = @(VdIds)
  return "current=$(VdCurrent) count=$($ids.Count) " + (($ids | ForEach-Object { "$_='$(VdName $_)'" }) -join ' ')
}
function VdChord($vk) { [G]::KeyDown(0x5B); [G]::KeyDown(0x11); Start-Sleep -Milliseconds 40; [G]::Press($vk); Start-Sleep -Milliseconds 40; [G]::KeyUp(0x11); [G]::KeyUp(0x5B); Start-Sleep -Seconds 2 }
# Win+Ctrl+D (a new desktop, switched to), Win+Ctrl+F4 (close the current one), Win+Ctrl+Left/Right.
function VdNew() { VdChord 0x44 }
function VdClose() { VdChord 0x73 }
function VdLeft() { VdChord 0x25 }
function VdRight() { VdChord 0x27 }
function TaskView() { [G]::KeyDown(0x5B); Start-Sleep -Milliseconds 40; [G]::Press(0x09); Start-Sleep -Milliseconds 40; [G]::KeyUp(0x5B); Start-Sleep -Seconds 2 }
# Back to one desktop (a job may have left more).
function VdOnlyOne() { for ($i = 0; $i -lt 8 -and @(VdIds).Count -gt 1; $i++) { VdClose } }
# Task View at 1600x900 centres the desktop tiles and the "New desktop" tile, 248 px apart.
function TaskViewTileX($n) { $tiles = @(VdIds).Count + 1; return 800 - ($tiles - 1) * 124 + ($n - 1) * 248 }
# Desktop $n (from 1) renamed in Task View: a click on its label, the new name, Enter.
function VdRename($n, $name) {
  for ($k = 0; $k -lt 2 -and (VdName (@(VdIds)[$n - 1])) -ne $name; $k++) {
    TaskView
    [G]::Click((TaskViewTileX $n) - 124 + 32, 690); Start-Sleep -Milliseconds 800
    [G]::KeyDown(0x11); [G]::Press(0x41); [G]::KeyUp(0x11)
    [G]::Type($name); Start-Sleep -Milliseconds 300
    [G]::Press(0x0D); Start-Sleep -Milliseconds 800
    [G]::Press(0x1B); Start-Sleep -Seconds 1
  }
}
# Switch to desktop $n by clicking its thumbnail in Task View.
function TaskViewPick($n) { TaskView; [G]::Click((TaskViewTileX $n), 770); Start-Sleep -Seconds 2 }
# Two desktops named $a and $b, the first one current.
function VdTwo($a, $b) {
  VdOnlyOne; VdNew; VdLeft
  VdRename 1 $a; VdRename 2 $b
  Log ("desktops: " + (VdState))
}

# Whether the fence window titled $t is shown (after its fade); $null when it has no window.
function FVis($t) { $h = [VD]::Fence($t); if ($h -eq [IntPtr]::Zero) { return $null }; return [VD]::Visible($h) }
function VisText($titles) { return (($titles | ForEach-Object { "$_=$(FVis $_)" }) -join ' ') }
# The fence menu of the fence titled $t (a right-click on its title row) and its 在哪个虚拟桌面显示
# submenu (English UI: "Show on virtual desktop").
function FenceMenu($t) { $r = Rect ([VD]::Fence($t)); [G]::RClick($r.L + 150, $r.T + 14); Start-Sleep -Milliseconds 900; return (MenuWin) }
function VdSubmenu($t, $entry = 'Show on virtual desktop|在哪个虚拟桌面显示') {
  $m = FenceMenu $t
  MenuPick $entry $m | Out-Null; Start-Sleep -Milliseconds 700
  return (MenuWin)
}
# Esc until no popup menu is visible (the kit's CloseMenus looks at the first #32768 window only,
# which can be a hidden one while a menu and its submenu are open). An open fence menu holds
# PecoFence: desktop switches are followed only once it closes.
function CloseAllMenus() { for ($i = 0; $i -lt 4 -and @([W]::Tops('#32768')).Count -gt 0; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 300 } }
# Windows shows a tray balloon as a toast at the bottom right: its close button.
function CloseToast() { [G]::Click(1556, 734); Start-Sleep -Milliseconds 500; [G]::MoveTo(1250, 760) }
# Removes PecoFence's notices, on screen or still waiting in Windows' queue (a vd test makes many
# in a row, and one left over covered a fence in the next test file). Windows files a tray icon's
# notices under an app id it makes (NotifyIconGeneratedAumid_<n>); its AppUserModelId key names
# the exe (DisplayName pecofence.exe).
function ClearPfNotices() {
  $null = [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime]
  $ids = @(Get-ChildItem 'HKCU:\Software\Classes\AppUserModelId' -ErrorAction SilentlyContinue | Where-Object { (Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue).DisplayName -match 'pecofence' } | ForEach-Object { $_.PSChildName })
  if (-not $ids) { Log 'notices: no app id for pecofence.exe'; return }
  foreach ($id in $ids) {
    try { [Windows.UI.Notifications.ToastNotificationManager]::History.Clear($id); Log "notices of $id cleared" } catch { Log "notices of $id not cleared: $($_.Exception.Message)" }
  }
}
