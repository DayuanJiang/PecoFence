# Issue #219, Windows 10 1809 / Server 2019 (build 17763): the tray menu opened with a real
# right-click on the notification icon took neither the arrow keys nor Esc until a click in it.
# The menu started while PecoFence's thread still had the right button as "down" in its key
# state, and Windows ignores the keyboard in a menu for as long as a button counts as held.
# Needs a box of that build (the Server 2025 boxes never showed it): a `manual` line in suite.txt.
#
# On build 17763 UI Automation does not show the notification icons to TrayIconRect, so there
# the icon's place comes from Shell_NotifyIconGetRect (result 1: Windows moved the icon under
# the "^" button and the rectangle is that button's; a click on it opens the list).
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class TrayPlace {
  [StructLayout(LayoutKind.Sequential)] public struct NII { public uint cbSize; public IntPtr hWnd; public uint uID; public Guid guidItem; }
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, R_, B; }
  [DllImport("shell32.dll")] static extern int Shell_NotifyIconGetRect(ref NII id, out R r);
  // "result left top right bottom" of the icon with this owner window and id.
  public static string Ask(IntPtr hwnd, uint id) {
    var n = new NII(); n.cbSize = (uint)Marshal.SizeOf(typeof(NII)); n.hWnd = hwnd; n.uID = id;
    R r; int hr = Shell_NotifyIconGetRect(ref n, out r);
    return hr + " " + r.L + " " + r.T + " " + r.R_ + " " + r.B;
  }
}
"@
function TrayAsk() { return @(([TrayPlace]::Ask([G]::FindWindowW('PecoFence.Control', [NullString]::Value), 1)) -split ' ' | ForEach-Object { [int]$_ }) }
# A real right-click on PecoFence's notification icon; the menu window, or zero.
function TrayRightClick() {
  [G]::MoveTo(700, 450); Start-Sleep -Milliseconds 200
  $r = TrayIconRect
  if ($r) { [G]::RClick([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2)); return (MenuWin 5) }
  $p = TrayAsk
  if ($p[0] -eq 1) {
    [G]::Click([int](($p[1] + $p[3]) / 2), [int](($p[2] + $p[4]) / 2)); Start-Sleep -Milliseconds 900
    $p = TrayAsk
  }
  Log "tray icon by Shell_NotifyIconGetRect: $($p -join ' ')"
  if ($p[0] -lt 0) { return [IntPtr]::Zero }
  [G]::RClick([int](($p[1] + $p[3]) / 2), [int](($p[2] + $p[4]) / 2))
  return (MenuWin 5)
}
function MenuShown() { $m = [G]::FindWindowW('#32768', [NullString]::Value); return ($m -ne [IntPtr]::Zero -and [G]::IsWindowVisible($m)) }
# The highlighted entry (MFS_HILITE).
function MenuHot($m) { return ((@(MenuRows $m) | Where-Object { $_.text -and ($_.state -band 0x80) } | ForEach-Object { $_.text }) -join ',') }

function TrayKeysRound($round) {
  trap { PfTrap $_; continue }
  $m = TrayRightClick
  Start-Sleep -Milliseconds 400
  $texts = MenuTexts $m
  Check "round $round the right-click opens PecoFence's menu" ($m -ne [IntPtr]::Zero -and $texts -match 'Exit PecoFence') "menu=$m foreground: $(Fg); $texts"
  $h0 = MenuHot $m
  [G]::Press(0x28); Start-Sleep -Milliseconds 300
  $h1 = MenuHot $m
  [G]::Press(0x28); Start-Sleep -Milliseconds 300
  $h2 = MenuHot $m
  Check "round $round the Down key moves the highlight" ($h1 -ne '' -and $h2 -ne '' -and $h2 -ne $h1) "highlight: '$h0' -> '$h1' -> '$h2'"
  Shot "tray-keys-$round"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 700
  Check "round $round Esc closes the menu" (-not (MenuShown)) "still open=$(MenuShown)"
  # A menu that ignored Esc: a click on the bare desktop closes it, so the next steps can run.
  if (MenuShown) { DeskClick 1000 700 | Out-Null; Start-Sleep -Milliseconds 700 }
  $line = @(PfLogAll | Where-Object { $_ -match 'popup menu: mouse buttons' }) | Select-Object -Last 1
  Log "round ${round}: $line"
}

Case 'setup'
CloseWindows
PfStart
Start-Sleep -Seconds 2

Case 'tray menu after a real right-click'
TrayKeysRound 1
Case 'the same again'
TrayKeysRound 2
