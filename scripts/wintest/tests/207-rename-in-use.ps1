# Feedback #207 item 3: `rename failed error=os error 32` (another program has the file open).
#   1 the same item renamed with the name box twice, a box closed without a change between, as in
#     the reporter's log, for a text file, a picture and a folder: nothing of PecoFence holds them
#   2 a file another program holds open: what the user gets (Explorer shows 「文件正在使用」 with
#     重试), and the rename once the program lets go
# Runs on a 1600x900 box at 100 %.
trap { PfTrap $_; continue }
Add-Type -AssemblyName System.Drawing
# Visible top-level windows whose title matches: "hwnd class title" lines.
Add-Type -TypeDefinition @'
using System; using System.Collections.Generic; using System.Runtime.InteropServices; using System.Text;
public static class RiWin {
  delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowTextW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  public static IntPtr[] Titled(string pattern) {
    var list = new List<IntPtr>();
    var re = new System.Text.RegularExpressions.Regex(pattern);
    EnumWindows((h, l) => { if (IsWindowVisible(h)) { var s = new StringBuilder(512); GetWindowTextW(h, s, 512); if (re.IsMatch(s.ToString())) list.Add(h); } return true; }, IntPtr.Zero);
    return list.ToArray();
  }
}
'@

function RiKey($vk, $ms = 400) { [G]::Press($vk); Start-Sleep -Milliseconds $ms }
function RiBox() {
  for ($i = 0; $i -lt 60; $i++) {
    $h = [G]::FindWindowW('PecoFence.Rename', [NullString]::Value)
    if ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) { return $h }
    Start-Sleep -Milliseconds 50
  }
  return [IntPtr]::Zero
}
# The item's cell clicked, F2: the name box.
function RiOpen($name) {
  $order = @(@(PfJson item list --fence R) | ForEach-Object { $_.name })
  $p = CellPt 'R' ([array]::IndexOf($order, $name))
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  RiKey 0x71 600
  $box = RiBox
  # The first click of a run can go to bringing the fence forward: once more.
  if ($box -eq [IntPtr]::Zero) { [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500; RiKey 0x71 600; $box = RiBox }
  return $box
}
function RiRename($name, $new) {
  $box = RiOpen $name
  if ($box -eq [IntPtr]::Zero) { return $false }
  [G]::KeyDown(0x11); [G]::Press(0x41); [G]::KeyUp(0x11); Start-Sleep -Milliseconds 100
  [G]::Type($new); Start-Sleep -Milliseconds 300
  RiKey 0x0D 1500
  return $true
}
function Busy() { return @(PfLogAll | Where-Object { $_ -match 'rename failed' }) }
# Visible dialogs (#32770) of any process: "process: title".
function Dialogs() { return @([W]::Tops('#32770') | ForEach-Object { "$([W]::ProcOf($_)): $([W]::Title($_))" }) }

Case 'setup'
CloseWindows
Set-Content -LiteralPath (Join-Path $PfDesk 'ri-text.txt') -Value 'text'
$bmp = New-Object Drawing.Bitmap 64, 64
$g = [Drawing.Graphics]::FromImage($bmp); $g.Clear([Drawing.Color]::Orange); $g.Dispose()
$bmp.Save((Join-Path $PfDesk 'ri-pic.png'), [Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
New-Item -ItemType Directory -Force (Join-Path $PfDesk 'ri-folder') | Out-Null
Set-Content -LiteralPath (Join-Path $PfDesk 'ri-folder\inside.txt') -Value 'x'
Set-Content -LiteralPath (Join-Path $PfDesk 'ri-lock.txt') -Value 'locked'
PfStart -Script @('sleep 3600000')
ClearFences | Out-Null
Pf fence create --title R --rect '300,120,420,300' | Out-Null
Pf item move (Join-Path $PfDesk 'ri-text.txt') (Join-Path $PfDesk 'ri-pic.png') (Join-Path $PfDesk 'ri-folder') (Join-Path $PfDesk 'ri-lock.txt') --to R | Out-Null
ParkMouse; Start-Sleep -Seconds 3

Case '1 renamed twice, a box closed between'
foreach ($t in @(@('ri-text', 'ri-text 2', 'ri-text 3', 'ri-text 3.txt'), @('ri-pic', 'ri-pic 2', 'ri-pic 3', 'ri-pic 3.png'), @('ri-folder', 'ri-folder 2', 'ri-folder 3', 'ri-folder 3'))) {
  $box = RiOpen $t[0]; RiKey 0x1B 600
  $ok1 = RiRename $t[0] $t[1]
  $box = RiOpen $t[1]; RiKey 0x1B 600
  $ok2 = RiRename $t[1] $t[2]
  ParkMouse
  $on = Test-Path -LiteralPath (Join-Path $PfDesk $t[3])
  Check "$($t[0]): renamed twice with the name box" ($ok1 -and $ok2 -and $on -and (Busy).Count -eq 0) "boxes $ok1 $ok2; $($t[3]) on the desktop: $on; rename failures in the log: $((Busy) -join ' | ')"
}

Case '2 a file another program holds open'
$lock = Join-Path $PfDesk 'ri-lock.txt'
$fs = [IO.File]::Open($lock, 'Open', 'ReadWrite', 'None')
$ok = RiRename 'ri-lock' 'ri-lock renamed'
Start-Sleep -Seconds 2
$dlgs = Dialogs
Shot '2-in-use'
Check 'the rename of a file in use is not done' ((Test-Path -LiteralPath $lock) -and -not (Test-Path -LiteralPath (Join-Path $PfDesk 'ri-lock renamed.txt'))) "box $ok; dialogs: $($dlgs -join ' | '); log: $((Busy) -join ' | ')"
$inUse = @([RiWin]::Titled('^(File In Use|文件正在使用)$'))
Check 'the user is told the file is open in another program, with Try Again' ($inUse.Count -ge 1) "File In Use windows: $($inUse.Count); dialogs: $($dlgs -join ' | ')"
$fs.Close()
Start-Sleep -Milliseconds 500
if ($inUse.Count -ge 1) {
  $btn = UiaFind $inUse[0] 'Try Again'
  if (-not $btn) { $btn = UiaFind $inUse[0] '重试' }
  Log "Try Again button: $([bool]$btn)"
  if ($btn) { $btn.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke() }
  Start-Sleep -Milliseconds 2500
}
ParkMouse
$names = @(@(PfJson item list --fence R) | ForEach-Object { $_.name })
Check 'the program let go, Try Again: renamed, and the fence shows the new name' ((Test-Path -LiteralPath (Join-Path $PfDesk 'ri-lock renamed.txt')) -and ($names -contains 'ri-lock renamed')) "fence R: $($names -join ', '); dialogs now: $((Dialogs) -join ' | ')"
foreach ($h in @([RiWin]::Titled('^(File In Use|文件正在使用)$'))) { [G]::SetForegroundWindow($h) | Out-Null; RiKey 0x1B 400 }

PfQuit
foreach ($n in 'ri-text 3.txt', 'ri-pic 3.png', 'ri-lock.txt', 'ri-lock renamed.txt', 'ri-text.txt', 'ri-pic.png') { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
foreach ($n in 'ri-folder 3', 'ri-folder') { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Recurse -Force -ErrorAction SilentlyContinue }
