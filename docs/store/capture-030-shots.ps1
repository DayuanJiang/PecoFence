# Store screenshots for 0.3.0 (scripts/wintest steps file, see docs/store/V2.md): the Peek search
# list and dialog jump, captured from the release build at 2560x1440 and 200 % like the earlier
# Store captures (the overview fixture's folders, wallpaper and fence layout). Run on a
# 2560x1680 screen: the frames are its top 2560x1440. They go to $PfShots as store-search.png and
# store-dialog.png.
Add-Type -AssemblyName System.Windows.Forms
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class SD {
  [DllImport("user32.dll")] public static extern uint GetDpiForSystem();
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr p);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr a, int x, int y, int w, int t, uint f);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
}
"@
# Physical pixels for window positions and the screen grab (the earlier captures did the same).
Log "dpi aware: $([SD]::SetProcessDpiAwarenessContext([IntPtr](-4)))"
function Grab($name) {
  $b = New-Object Drawing.Bitmap 2560, 1440
  $g = [Drawing.Graphics]::FromImage($b); $g.CopyFromScreen(0, 0, 0, 0, $b.Size)
  $b.Save("$PfShots\$name.png", [Drawing.Imaging.ImageFormat]::Png); $g.Dispose(); $b.Dispose(); Log "grab $name"
}
$src = "$PfExtra\store"
$S = [Environment]::GetFolderPath("MyDocuments")

Case 'setup'
CloseWindows
New-Item -ItemType Directory -Force $S | Out-Null
foreach ($d in 'Projects', 'Inspiration', 'Today') { Copy-Item -LiteralPath "$src\$d" -Destination $S -Recurse -Force }
# The dialog shot opens Campaign: a few files so the list is not empty.
foreach ($f in 'Campaign plan.docx', 'Poster draft.png', 'Press release.txt', 'Budget.xlsx') { Set-Content -LiteralPath (Join-Path "$S\Projects\Campaign" $f) -Value 'x' }
Set-ItemProperty 'HKCU:\Control Panel\Desktop' -Name WallpaperStyle -Value '10'
Set-ItemProperty 'HKCU:\Control Panel\Desktop' -Name TileWallpaper -Value '0'
[G]::SystemParametersInfoW(0x14, 0, "$src\paper-light.png", 3) | Out-Null
# The screen is 2560x1680: the frame is its top 2560x1440, the taskbar and the desktop inbox
# fence sit in the strip below it.
# Quick access entries left by earlier test jobs on this box (only these names; folders stay).
$qa = (New-Object -ComObject Shell.Application).Namespace('shell:::{679f85cb-0220-4080-b29b-5540cc05aab6}')
foreach ($it in @($qa.Items())) {
  if ($it.Name -in @('项目资料', 'notes', 'with space', '其他', '照片')) {
    $verbs = @($it.Verbs() | ForEach-Object { $_.Name })
    Log "quick access: $($it.Name) verbs=$($verbs -join ',')"
    foreach ($v in @($it.Verbs())) { if (($v.Name -replace '&', '') -match 'Unpin from|Remove from') { $v.DoIt(); Log "  removed via $($v.Name)" } }
  }
}
$screen = [Windows.Forms.Screen]::PrimaryScreen
$scale = [SD]::GetDpiForSystem() / 96.0
Log "screen $($screen.Bounds) work $($screen.WorkingArea) device $($screen.DeviceName) scale $scale"
Check 'screen is 2560x1680 at 200 %' ($screen.Bounds.Width -eq 2560 -and $screen.Bounds.Height -eq 1680 -and $scale -eq 2) "$($screen.Bounds) scale=$scale"
$work = $screen.WorkingArea
$t = Get-Content -LiteralPath "$src\template.json" -Raw -Encoding UTF8 | ConvertFrom-Json
$fp = [pscustomobject]@{ devicePath = $screen.DeviceName; workDip = @(($work.Width / $scale), ($work.Height / $scale)); dpi = [int]($scale * 96) }
$t.layouts[0].fingerprint = @($fp)
foreach ($f in $t.layouts[0].fences) {
  $g = $f.geometry
  $g.monitor = $screen.DeviceName
  $g.x = $g.x / $scale; $g.y = $g.y / $scale; $g.w = $g.w / $scale; $g.h = $g.h / $scale
  $g.workW = $work.Width / $scale; $g.workH = $work.Height / $scale
  $f.expandedH = $g.h
  if ($f.source -and $f.source.path) { $f.source.path = Join-Path $S (Split-Path $f.source.path -Leaf) }
  if ($f.kind -eq 'inbox') { $g.x = 40 / $scale; $g.y = 1470 / $scale }
}
$t.settings.peek.enabled = $true
$t.settings.hideRealIcons = $true
New-Item -ItemType Directory -Force "$PfApp\config" | Out-Null
[IO.File]::WriteAllText("$PfApp\config\config.json", ($t | ConvertTo-Json -Depth 40), (New-Object Text.UTF8Encoding($false)))
PfStart -Lang 'en'
Start-Sleep -Seconds 6
Pf settings set tools.dialogJump true | Out-Null
Start-Sleep -Seconds 2
Grab 'store-desktop'

Case 'search'
# The app in front: the same brief window as the Peek capture.
$board = New-Object Windows.Forms.Form
$board.FormBorderStyle = 'None'; $board.StartPosition = 'Manual'; $board.ShowInTaskbar = $false
$board.Bounds = New-Object Drawing.Rectangle 100, 200, 2360, 1120
$board.Text = 'Terra - Creative brief'
$board.BackgroundImage = [Drawing.Image]::FromFile("$src\brief-window.png"); $board.BackgroundImageLayout = 'Stretch'
$board.Show(); [Windows.Forms.Application]::DoEvents()
[SD]::SetForegroundWindow($board.Handle) | Out-Null
Start-Sleep -Seconds 1
[Windows.Forms.Application]::DoEvents()
PeekKey
Start-Sleep -Seconds 2
[G]::Type('study')
[G]::MoveTo(1700, 150)
Start-Sleep -Seconds 3
[Windows.Forms.Application]::DoEvents()
Grab 'store-search'
$t2 = (Get-Content -LiteralPath (PfLogPath) -Raw -Encoding UTF8)
Check 'search list shown' ($t2 -match 'search') ''
[G]::Press(0x1B); Start-Sleep -Milliseconds 800
[G]::Press(0x1B); Start-Sleep -Seconds 1
$board.Close(); $board.Dispose()

Case 'dialog'
Start-Process explorer.exe "`"$S\Projects\Campaign`""; Start-Sleep -Seconds 4
$ex = [G]::FindWindowW('CabinetWClass', [NullString]::Value)
[L]::MoveWindow($ex, 1340, 60, 1120, 470, $true) | Out-Null; Start-Sleep -Milliseconds 600
Start-Process notepad.exe -WorkingDirectory C:\pf; Start-Sleep -Seconds 2
$np = [G]::FindWindowW('Notepad', [NullString]::Value)
[L]::MoveWindow($np, 130, 330, 1450, 1080, $true) | Out-Null; Start-Sleep -Milliseconds 600
TitleClick $np
Chord 0x11 0x4F; Start-Sleep -Seconds 3
$dlg = [G]::FindWindowW('#32770', 'Open')
Log "dialog before: $([QS]::Address($dlg))"
# Show every file type in the list (a pattern typed into the name box filters the list).
[QS]::SetFileName($dlg, '*.*'); [L]::PostMessageW($dlg, 0x111, [IntPtr]1, [IntPtr]0) | Out-Null
Start-Sleep -Seconds 1
[QS]::SetFileName($dlg, '')
TitleClick $ex; Start-Sleep -Seconds 1
$dr = Rect $dlg
[G]::Click($dr.L + 200, $dr.T + 30)
$got = WaitAddr $dlg 'Campaign'
Start-Sleep -Seconds 2
[G]::MoveTo(2300, 1150)
Start-Sleep -Milliseconds 500
Grab 'store-dialog'
Check 'dialog jumped to Campaign' ($got -like '*Campaign') "address=$got"
[L]::PostMessageW($dlg, 0x111, [IntPtr]2, [IntPtr]0) | Out-Null
Start-Sleep -Seconds 1
CloseWindows
PfQuit
# The demo folders were only for these frames.
foreach ($d in 'Projects', 'Inspiration', 'Today') { Remove-Item -LiteralPath (Join-Path $S $d) -Recurse -Force -ErrorAction SilentlyContinue }
Log "demo folders removed from $S"
