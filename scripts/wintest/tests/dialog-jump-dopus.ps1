# #240 对话框跳转 with Directory Opus: coming back to an Open dialog from a Directory Opus lister
# jumps it to the lister's folder (the active tab of that lister, not of another one), Ctrl+G
# jumps there too, and Explorer windows still work in between. Each check reads the dialog's
# address bar.
# Manual: Directory Opus must be installed on the box first (as Administrator:
#   DOpusInstall-13.25.exe /VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP-
# from https://www.gpsoft.com.au/download/release); its first start asks a welcome dialog, which
# these steps answer with every choice off (no Explorer replacement, no autostart).
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class DO {
  [DllImport("user32.dll")] public static extern IntPtr SendMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
}
'@
Case 'setup'
CloseWindows
$opus = 'C:\Program Files\GPSoftware\Directory Opus'
Check 'Directory Opus is installed' (Test-Path "$opus\dopusrt.exe") $opus
$a = 'C:\pf\run\demo\Opus A'; $b = 'C:\pf\run\demo\Opus B'; $c = 'C:\pf\run\demo\Explorer C'
New-Item -ItemType Directory -Force $a, $b, $c | Out-Null
Set-Content -Path (Join-Path $a 'a.txt') -Value 'x'
Set-Culture en-US
if (-not (Get-Process dopus -ErrorAction SilentlyContinue)) { Start-Process "$opus\dopus.exe"; Start-Sleep -Seconds 8 }
# The first start's welcome dialog: every choice off, then OK.
$welcome = [G]::FindWindowW('#32770', 'Directory Opus')
if ($welcome -ne [IntPtr]::Zero) {
  $kids = New-Object System.Collections.ArrayList
  [L]::EnumChildWindows($welcome, { param($h, $l) [void]$kids.Add($h); return $true }, [IntPtr]::Zero) | Out-Null
  foreach ($k in $kids) {
    if ([L]::Cls($k) -eq 'Button' -and [L]::Text($k) -ne 'OK' -and [int][DO]::SendMessageW($k, 0xF0, [IntPtr]::Zero, [IntPtr]::Zero) -eq 1) {
      $r = Rect $k; [G]::Click($r.L + 8, [int](($r.T + $r.B) / 2)); Start-Sleep -Milliseconds 300
    }
  }
  foreach ($k in $kids) { if ([L]::Cls($k) -eq 'Button' -and [L]::Text($k) -eq 'OK') { $r = Rect $k; [G]::Click([int](($r.L + $r.R) / 2), [int](($r.T + $r.B) / 2)) } }
  Start-Sleep -Seconds 2
  Log 'Directory Opus welcome answered'
}
foreach ($w in @([W]::Tops('dopus.lister'))) { [L]::PostMessageW($w, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null }
Start-Sleep -Seconds 1
# A lister for a folder, placed right of the dialog at a height of its own (title clicks must
# reach the window meant).
function OpenLister($path, $y) {
  $before = @([W]::Tops('dopus.lister'))
  Start-Process "$opus\dopusrt.exe" -ArgumentList "/acmd Go `"$path`" NEW" -Wait
  for ($i = 0; $i -lt 30; $i++) {
    $new = @([W]::Tops('dopus.lister') | Where-Object { $before -notcontains $_ })
    if ($new.Count -gt 0) { break }
    Start-Sleep -Milliseconds 200
  }
  $h = $new | Select-Object -First 1
  if ($h) { [L]::MoveWindow($h, 820, $y, 760, 250, $true) | Out-Null; Start-Sleep -Milliseconds 500 }
  return $h
}
function OpenExplorer($path) {
  Start-Process explorer.exe "`"$path`""; Start-Sleep -Seconds 4
  $h = [G]::FindWindowW('CabinetWClass', [NullString]::Value)
  [L]::MoveWindow($h, 820, 580, 760, 260, $true) | Out-Null; Start-Sleep -Milliseconds 500
  return $h
}
function Away($dlg) {
  [QS]::SetFileName($dlg, 'C:\Windows'); [L]::PostMessageW($dlg, 0x111, [IntPtr]1, [IntPtr]0) | Out-Null
  $w = WaitAddr $dlg 'Windows'; [QS]::SetFileName($dlg, ''); return $w
}
PfStart
Pf settings set tools.dialogJump true | Out-Null
$la = OpenLister $a 60
Check 'a lister for folder A is open' ($null -ne $la) "lister=$la title='$([L]::Text($la))'"

Case 'Open dialog'
TitleClick $la
Start-Process notepad.exe -WorkingDirectory C:\pf; Start-Sleep -Seconds 2
$np = [G]::FindWindowW('Notepad', [NullString]::Value)
[L]::MoveWindow($np, 10, 40, 800, 760, $true) | Out-Null; Start-Sleep -Milliseconds 500
TitleClick $np
Chord 0x11 0x4F; Start-Sleep -Seconds 3
$dlg = [G]::FindWindowW('#32770', 'Open'); $dr = Rect $dlg
$addr = [QS]::Address($dlg)
Check 'the dialog opens in its own folder, not the lister''s' ($dlg -ne [IntPtr]::Zero -and $addr -notlike '*Opus A') (DlgState 'open' $dlg)

Case 'back from the lister'
TitleClick $la
[G]::Click($dr.L + 120, $dr.T + 14)
$t0 = Get-Date
$got = WaitAddr $dlg 'Opus A'
Check 'coming back from the lister jumps to its folder' ($got -like '*Opus A') "address=$got after $([int]((Get-Date) - $t0).TotalMilliseconds) ms"
Shot 'j01-back-from-lister'

Case 'Ctrl+G'
$away = Away $dlg
Chord 0x11 0x47
$got = WaitAddr $dlg 'Opus A'
Check 'Ctrl+G jumps to the lister''s folder' ($away -like '*Windows' -and $got -like '*Opus A') "away=$away after Ctrl+G=$got"

Case 'the lister the user was in'
$lb = OpenLister $b 320
TitleClick $lb
[G]::Click($dr.L + 120, $dr.T + 14)
$got = WaitAddr $dlg 'Opus B'
Check 'a second lister: its own folder' ($got -like '*Opus B') "address=$got"
TitleClick $la
[G]::Click($dr.L + 120, $dr.T + 14)
$got = WaitAddr $dlg 'Opus A'
Check 'back in the first lister: the first folder again' ($got -like '*Opus A') "address=$got"

Case 'Explorer in between'
$ex = OpenExplorer $c
TitleClick $ex
[G]::Click($dr.L + 120, $dr.T + 14)
$got = WaitAddr $dlg 'Explorer C'
Check 'Explorer still jumps' ($got -like '*Explorer C') "address=$got"
TitleClick $lb
[G]::Click($dr.L + 120, $dr.T + 14)
$got = WaitAddr $dlg 'Opus B'
Check 'and the lister after it' ($got -like '*Opus B') "address=$got"
$read = @(PfLogAll | Where-Object { $_ -match 'Directory Opus folder read' }).Count
Check 'the log names the folder read from Directory Opus' ($read -ge 3) "lines=$read"

Case 'cleanup'
[L]::PostMessageW($dlg, 0x111, [IntPtr]2, [IntPtr]0) | Out-Null; Start-Sleep -Seconds 1
PfQuit
CloseWindows
foreach ($w in @([W]::Tops('dopus.lister'))) { [L]::PostMessageW($w, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null }
