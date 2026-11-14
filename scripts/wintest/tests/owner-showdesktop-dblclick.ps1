# A double-click on one of Windows' own desktop icons (they show beside the fences with
# 「Windows 桌面图标」 not hidden) must open the file and leave the fences alone. It hid every
# fence when Explorer's list view did not mark the icon as the one under the pointer, and that
# mark was the only test for "this press is on an icon". The list view sets the mark when the
# pointer moves over the icon and drops it when a window comes over the pointer; when that
# window goes away again (closed, or show desktop with Win+D) and the pointer has not moved,
# the mark does not come back. A double-click then read as one on the bare desktop: quick hide.
. C:\pf\kit\deskicons.ps1
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class OwnerDbl {
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr FindWindowExW(IntPtr p, IntPtr a, string c, string t);
  [DllImport("user32.dll")] static extern IntPtr SendMessageTimeoutW(IntPtr h, uint m, IntPtr w, IntPtr l, uint f, uint t, out IntPtr r);
  // The icon Explorer's desktop list view marks as under the pointer (LVM_GETHOTITEM): -1 none, -2 no answer.
  public static long HotItem(IntPtr defView) {
    IntPtr lv = FindWindowExW(defView, IntPtr.Zero, "SysListView32", null), r;
    if (lv == IntPtr.Zero || SendMessageTimeoutW(lv, 0x103D, IntPtr.Zero, IntPtr.Zero, 2, 1000, out r) == IntPtr.Zero) return -2;
    return (long)r;
  }
}
'@
$File = 'owner-dbl.txt'

# The middle of the picture of Windows' own icon of a desktop file.
function IconPt($name) {
  $org = [DeskIcons]::Origin() -split ','
  foreach ($l in [DeskIcons]::List()) {
    $p = $l -split '\s*\|\s*'
    if ($p[0] -like "*$name") { return @(([int]$org[0] + [int]$p[1] + 24), ([int]$org[1] + [int]$p[2] + 24)) }
  }
  return $null
}
# The Notepad that shows the test file (its title follows the "hide extensions" setting), or zero.
function FileWin($sec = 8) {
  $base = [IO.Path]::GetFileNameWithoutExtension($File)
  $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.Elapsed.TotalSeconds -lt $sec) {
    foreach ($h in [W]::Tops('Notepad')) { if ([W]::Title($h) -like "$base*") { return $h } }
    Start-Sleep -Milliseconds 100
  }
  return [IntPtr]::Zero
}
function CloseNotepads() {
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Milliseconds 800
}
# Waits up to $sec for the count of visible fences to be $want; returns the count.
function FencesNow($want, $sec = 2) {
  $sw = [Diagnostics.Stopwatch]::StartNew()
  do { $n = VisFences; if ($n -eq $want) { return $n }; Start-Sleep -Milliseconds 100 } while ($sw.Elapsed.TotalSeconds -lt $sec)
  return $n
}
# A double-click where the pointer is, without moving it; what opened and the fences after it.
function DblHere($what, $hot) {
  $before = VisFences
  MDown; Start-Sleep -Milliseconds 30; MUp; Start-Sleep -Milliseconds 90; MDown; Start-Sleep -Milliseconds 30; MUp
  $h = FileWin
  Start-Sleep -Milliseconds 1200
  $after = VisFences
  Check $what ($h -ne [IntPtr]::Zero -and $before -gt 0 -and $after -eq $before) "icon marked by Explorer before the double-click: $hot (-1 none); file window=$h; fences $before -> $after"
  CloseNotepads
  Pf fence show-all | Out-Null
  Start-Sleep -Milliseconds 700
}
# The pointer rests on the icon (Explorer saw it arrive), a maximized Notepad comes over it (a
# small move inside that window and back, which Explorer does not see, makes Windows tell the
# list view that the pointer left), and $Uncover brings the desktop back into view under the
# resting pointer. Tried up to three times, until Explorer's list view marks no icon; returns
# that mark.
function RestOnIcon([scriptblock]$Uncover) {
  for ($i = 0; $i -lt 3; $i++) {
    [G]::MoveTo($script:Bare[0], $script:Bare[1]); Start-Sleep -Milliseconds 300
    [G]::MoveTo($script:Icon[0], $script:Icon[1]); Start-Sleep -Milliseconds 600
    Start-Process notepad.exe -WindowStyle Maximized
    Start-Sleep -Seconds 2
    [G]::MoveTo($script:Icon[0] + 60, $script:Icon[1] + 40); Start-Sleep -Milliseconds 300
    [G]::MoveTo($script:Icon[0], $script:Icon[1]); Start-Sleep -Milliseconds 400
    & $Uncover
    Start-Sleep -Milliseconds 1500
    $hot = [OwnerDbl]::HotItem([W]::DefView())
    if ($hot -eq -1) { break }
    CloseNotepads
  }
  return $hot
}

Case 'double-click on a Windows desktop icon'
DeskFile $File | Out-Null
PfStart
Start-Sleep -Seconds 4
Pf settings set hideRealIcons false | Out-Null
Start-Sleep -Seconds 4
$script:Icon = IconPt $File
$script:Bare = FreeSpot 700 500
Check "Windows' own icons show and the test file's icon is found" ([W]::IconsVisible() -and $script:Icon -and $script:Bare) "icon=$($script:Icon -join ',') bare=$($script:Bare -join ',')"

# The pointer comes onto the icon in view of Explorer: this always worked.
[G]::MoveTo($script:Bare[0], $script:Bare[1]); Start-Sleep -Milliseconds 300
[G]::MoveTo($script:Icon[0], $script:Icon[1]); Start-Sleep -Milliseconds 600
DblHere 'the pointer comes onto the icon: the file opens and the fences stay' ([OwnerDbl]::HotItem([W]::DefView()))

$hot = RestOnIcon { CloseNotepads }
DblHere 'a window over the icon is closed, the pointer rests on the icon: the file opens and the fences stay' $hot

Case 'the same after show desktop'
$hot = RestOnIcon { Chord 0x5B 0x44 }
DblHere 'after Win+D, the pointer rests on the icon: the file opens and the fences stay' $hot

Case 'quick hide still works beside the icons'
$all = VisFences
[G]::MoveTo($script:Bare[0], $script:Bare[1]); Start-Sleep -Milliseconds 300
[G]::Click($script:Icon[0], $script:Icon[1])
Start-Sleep -Milliseconds 900
DblClick $script:Bare[0] $script:Bare[1]
$hidden = FencesNow 0
Check 'an icon selected, then a double-click on the bare desktop hides the fences' ($all -gt 0 -and $hidden -eq 0) "fences $all -> $hidden"
Start-Sleep -Milliseconds 900
DblClick $script:Bare[0] $script:Bare[1]
$shown = FencesNow $all
Check 'and the next double-click there shows them again' ($shown -eq $all) "fences $hidden -> $shown"
Pf fence show-all | Out-Null
Pf settings set hideRealIcons true | Out-Null
