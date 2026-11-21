# Bing Wallpaper's desktop widget (GitHub #247, feedback #198): two full-screen windows of
# another program in the wallpaper layer that draw nothing of the wallpaper
# (assets/wallpaper-overlay-standin.cs stands in for them). With Liquid Glass they were taken
# for a live wallpaper that cannot be read: the fences turned into see-through panes and a card
# said so. They are no overlay of a player either: a player under them is still found.
#   run.sh tests/wallpaper-overlay.ps1      (release build, about 4 minutes)
# Each case starts PecoFence without a test script, so cards can show.

function Compile($src, $exe, $manifest) {
  $cp = New-Object CodeDom.Compiler.CompilerParameters
  $cp.GenerateExecutable = $true; $cp.OutputAssembly = $exe
  $cp.CompilerOptions = '/target:winexe' + $(if ($manifest) { " /win32manifest:$manifest" } else { '' })
  $cp.ReferencedAssemblies.Add('System.dll') | Out-Null
  Add-Type -TypeDefinition (Get-Content $src -Raw) -CompilerParameters $cp
}
Get-Process standin, overlay -ErrorAction SilentlyContinue | Stop-Process -Force
Compile 'C:\pf\kit\assets\live-wallpaper-standin.cs' "$PfRun\standin.exe" 'C:\pf\kit\assets\live-wallpaper-standin.manifest'
Compile 'C:\pf\kit\assets\wallpaper-overlay-standin.cs' "$PfRun\overlay.exe" $null
Check 'the stand-ins compiled' ((Test-Path "$PfRun\standin.exe") -and (Test-Path "$PfRun\overlay.exe")) "Windows build $([Environment]::OSVersion.Version.Build)"

# ---- the stand-ins (their logs go into the job log)
$script:Procs = @{}
function StandInLog($n) { return @(Get-Content "$PfRun\$n.log" -ErrorAction SilentlyContinue) }
function StandInStart($n, [string[]]$Opts = @()) {
  Remove-Item "$PfRun\$n.log" -ErrorAction SilentlyContinue
  $script:Procs[$n] = Start-Process "$PfRun\$n.exe" -ArgumentList (@('--log', "$PfRun\$n.log") + $Opts) -PassThru
  for ($i = 0; $i -lt 50 -and -not (StandInLog $n | Where-Object { $_ -match ' ready' }); $i++) { Start-Sleep -Milliseconds 200 }
  StandInLog $n | ForEach-Object { Log "${n}: $_" }
}
function StandInStop($n, $quitEvent) {
  $p = $script:Procs[$n]
  if (-not $p -or $p.HasExited) { return }
  $e = New-Object Threading.EventWaitHandle($false, 'AutoReset', "Local\$quitEvent"); $e.Set() | Out-Null; $e.Dispose()
  for ($i = 0; $i -lt 25 -and -not $p.HasExited; $i++) { Start-Sleep -Milliseconds 200 }
  if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force }
}
function OverlayStart() { StandInStart 'overlay' }
function OverlayStop() { StandInStop 'overlay' 'PfOverlayQuit' }
function PlayerStart([string[]]$Opts = @()) { StandInStart 'standin' $Opts }
function PlayerStop() { StandInStop 'standin' 'PfLiveQuit' }

function LiveState() { return "$(@((PfJson status).liveWallpapers)[0].state)" }
function WaitState($want, $sec = 30) {
  for ($i = 0; $i -lt $sec * 2; $i++) { $s = LiveState; if ($s -match $want) { return $s }; Start-Sleep -Milliseconds 500 }
  return (LiveState)
}
function NoticeWin() {
  $n = [G]::FindWindowW('PecoFence.Notice', [NullString]::Value)
  if ($n -ne [IntPtr]::Zero -and [G]::IsWindowVisible($n)) { return $n }
  return [IntPtr]::Zero
}
function NoticeTitle() { $n = NoticeWin; if ($n -eq [IntPtr]::Zero) { return '' }; return [L]::Text($n) }
# Like PfStart, without --test-script: cards may show.
function PfStartPlain() {
  if (Get-Process pecofence -ErrorAction SilentlyContinue) { PfQuit }
  $cur = PfLogPath
  if (Test-Path $cur) {
    New-Item -ItemType Directory -Force "$PfRun\logs" | Out-Null
    $n = @(Get-ChildItem "$PfRun\logs\app-*.log" -ErrorAction SilentlyContinue).Count + 1
    Copy-Item $cur "$PfRun\logs\app-$n.log"
  }
  $env:PECOFENCE_INSTANCE = $PfInst
  $env:RUST_LOG = 'pecofence=debug'
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $PfApp
  $h = WaitWin 'PecoFence.Control' 20 -Hidden
  Log "started plain $PfInst control=$h"
  Start-Sleep -Seconds 4
}
# The lines of this run's log about live wallpapers.
function LiveLines() { return @(Get-Content (PfLogPath) -Encoding UTF8 -ErrorAction SilentlyContinue | Where-Object { $_ -match 'live wallpaper|wallpaper layer' }) }
function Off($a, $b) { return ((0, 1, 2 | ForEach-Object { [math]::Abs($a[$_] - $b[$_]) }) | Measure-Object -Maximum).Maximum }
# The empty lower part of the fence's plate.
function Plate($name) { return MeanRgb (Crop $name 450 330 260 60) }

# ---- setup
Case 'setup'
CloseWindows
# A picture with detail as the Windows wallpaper: a gradient and circles.
$bmp = New-Object Drawing.Bitmap 1600, 900; $g = [Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = 'AntiAlias'
$br = New-Object Drawing.Drawing2D.LinearGradientBrush((New-Object Drawing.Point 0, 0), (New-Object Drawing.Point 1600, 900), [Drawing.Color]::FromArgb(30, 90, 200), [Drawing.Color]::FromArgb(250, 150, 60))
$g.FillRectangle($br, 0, 0, 1600, 900)
for ($i = 0; $i -lt 14; $i++) { $g.FillEllipse((New-Object Drawing.SolidBrush ([Drawing.Color]::FromArgb(200, 255, 255, 255))), 140 + $i * 110, 160 + (($i * 67) % 240), 70, 70) }
$g.Dispose(); $bmp.Save("$PfRun\picture.bmp", [Drawing.Imaging.ImageFormat]::Bmp); $bmp.Dispose()
[G]::SystemParametersInfoW(0x14, 0, "$PfRun\picture.bmp", 3) | Out-Null
$names = @('季度报告.txt', '图.txt', '预算表 2026.txt')
foreach ($n in $names) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue; DeskFile $n | Out-Null }
PfStart -Lang 'zh-CN'
Pf settings set themeStyle liquidGlass | Out-Null
Pf settings set theme light | Out-Null
ClearFences
Pf fence create --title Glass --rect '430,150,300,260' | Out-Null
foreach ($n in $names) { Pf item move "$PfDesk\$n" --to Glass | Out-Null }
Check 'a Liquid Glass fence is up' ((Pf settings get themeStyle) -match 'liquidGlass' -and (FenceGet 'Glass').title -eq 'Glass') "$((PfJson fence list | ForEach-Object { $_.title }) -join ', ')"

Case 'no widget'
PfStartPlain
ParkMouse
Start-Sleep -Seconds 4
$clear = Plate 'no-widget'
Shot 'screen-no-widget'
Check 'no widget: no live wallpaper, no card' ((LiveState) -eq 'none' -and (NoticeWin) -eq [IntPtr]::Zero) "state=$(LiveState) card='$(NoticeTitle)'"

Case 'the widget alone'
OverlayStart
Check 'the widget stand-in made its two windows in Progman' (@(StandInLog 'overlay' | Where-Object { $_ -match 'class=BingWallpaper_Widget_' }).Count -eq 2) ((StandInLog 'overlay') -join ' | ')
# A live wallpaper is found within a second or two and a card follows a few seconds later.
$s = WaitState 'notCapturable' 15
Start-Sleep -Seconds 6
ParkMouse
Start-Sleep -Seconds 1
$withWidget = Plate 'widget'
Shot 'screen-widget'
Check 'the widget is no live wallpaper: state none, no card' ($s -eq 'none' -and (NoticeWin) -eq [IntPtr]::Zero) "state=$s card='$(NoticeTitle)'"
Check 'the glass shows the Windows wallpaper as without the widget' ((Off $withWidget $clear) -le 6) "plate $($withWidget -join ' '), without the widget $($clear -join ' ')"
$found = @(LiveLines | Where-Object { $_ -match 'live wallpaper found.*class=BingWallpaper' })
Check 'the log names no widget window as a live wallpaper' ($found.Count -eq 0) (($found | Select-Object -First 2) -join ' | ')
$told = @(LiveLines | Where-Object { $_ -match 'wallpaper layer.*BingWallpaper_Widget_' })
Check 'the log says once that the widget is an overlay, not a wallpaper' ($told.Count -eq 1) (($told | Select-Object -First 3) -join ' | ')

Case 'a readable player under the widget'
OverlayStop
PfStartPlain
# The player first, then the widget: the widget's windows come before the player's in Progman.
PlayerStart @('--gdi', '--still')
OverlayStart
$s = WaitState 'captured' 25
Start-Sleep -Seconds 2
ParkMouse
Start-Sleep -Seconds 1
$overPlayer = Plate 'widget-over-gdi-player'
Shot 'screen-widget-over-gdi-player'
$cls = "$(@(LiveLines | Where-Object { $_ -match 'live wallpaper found' }) | Select-Object -Last 1)"
Check 'the player under the widget is found and read' ($s -eq 'captured' -and $cls -match 'class=PfLiveStandIn') "state=$s line='$cls'"
Check 'the glass shows the player, not the Windows wallpaper' ((Off $overPlayer $clear) -gt 12) "plate $($overPlayer -join ' '), Windows wallpaper $($clear -join ' ')"

Case 'an unreadable player under the widget'
PlayerStop
OverlayStop
PfStartPlain
PlayerStart @('--still')
OverlayStart
$s = WaitState 'notCapturable' 40
$cls = "$(@(LiveLines | Where-Object { $_ -match 'live wallpaper found' }) | Select-Object -Last 1)"
Check 'a scene player under the widget: it is the one found, and it cannot be read' ($s -eq 'notCapturable' -and $cls -match 'class=PfLiveStandIn') "state=$s line='$cls'"

Case 'the widget closes'
PlayerStop
$s = WaitState 'none' 30
Check 'with the player ended and the widget still there: no live wallpaper' ($s -eq 'none') "state=$s"
OverlayStop
Start-Sleep -Seconds 2
Check 'the widget ended: still none' ((LiveState) -eq 'none') "state=$(LiveState)"
Log ((LiveLines) -join "`n")
PfQuit
