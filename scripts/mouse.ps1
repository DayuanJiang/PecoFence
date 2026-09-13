param([string]$Action, [int]$X, [int]$Y, [int]$X2 = 0, [int]$Y2 = 0)
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class M {
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr v);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
}
"@
[M]::SetProcessDpiAwarenessContext([IntPtr]-4) | Out-Null
function Down { [M]::mouse_event(0x2,0,0,0,[UIntPtr]::Zero) }
function Up { [M]::mouse_event(0x4,0,0,0,[UIntPtr]::Zero) }
function RDown { [M]::mouse_event(0x8,0,0,0,[UIntPtr]::Zero) }
function RUp { [M]::mouse_event(0x10,0,0,0,[UIntPtr]::Zero) }
[M]::SetCursorPos($X,$Y) | Out-Null; Start-Sleep -Milliseconds 80
switch ($Action) {
  "click" { Down; Start-Sleep -Milliseconds 40; Up }
  "rclick" { RDown; Start-Sleep -Milliseconds 40; RUp }
  "dblclick" { Down; Start-Sleep -Milliseconds 40; Up; Start-Sleep -Milliseconds 70; Down; Start-Sleep -Milliseconds 40; Up }
  "drag" {
    Down; Start-Sleep -Milliseconds 120
    $steps = 20
    for ($i = 1; $i -le $steps; $i++) {
      $cx = [int]($X + ($X2 - $X) * $i / $steps); $cy = [int]($Y + ($Y2 - $Y) * $i / $steps)
      [M]::SetCursorPos($cx,$cy) | Out-Null; Start-Sleep -Milliseconds 15
    }
    Start-Sleep -Milliseconds 120; Up
  }
  "move" { }
}
