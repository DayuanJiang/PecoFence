# Owner round 2026-11-13: the see-through pane that Liquid Glass shows over a live wallpaper
# whose picture cannot be read, on a fence with a 色调 (tint). The tint lies over the pane at
# 22 % whatever the pane's share, and a light one (黄, 青) lightens the dark pane: in dark
# mode at 更透明 over a white wallpaper the white names fell under 4.5 to 1. A tinted fence
# in dark mode now gets a pane of 74 % at least; without a tint, and in light mode, the pane
# is as it was (默认 78 %, 更透明 65 %).
#   run.sh tests/owner8-pane-tint.ps1      (release build, 1600x900 at 100 %, about 2 minutes)
# The player is assets/live-wallpaper-standin.cs with a white picture: the worst wallpaper
# under the dark pane, so the contrast is measured on the screen and not worked out.
trap { PfTrap $_; continue }

# ---- the stand-in player, white
Get-Process standin -ErrorAction SilentlyContinue | Stop-Process -Force
$src = Get-Content 'C:\pf\kit\assets\live-wallpaper-standin.cs' -Raw
$white = $src -replace 'new byte\[\] \{ 40, 40, 220 \}, new byte\[\] \{ 60, 180, 40 \}, new byte\[\] \{ 220, 90, 40 \}, new byte\[\] \{ 200, 60, 160 \},', 'new byte[] { 255, 255, 255 }, new byte[] { 255, 255, 255 }, new byte[] { 255, 255, 255 }, new byte[] { 255, 255, 255 },'
$cp = New-Object CodeDom.Compiler.CompilerParameters
$cp.GenerateExecutable = $true; $cp.OutputAssembly = "$PfRun\standin.exe"
$cp.CompilerOptions = '/target:winexe /win32manifest:C:\pf\kit\assets\live-wallpaper-standin.manifest'
$cp.ReferencedAssemblies.Add('System.dll') | Out-Null
Add-Type -TypeDefinition $white -CompilerParameters $cp
Check 'the stand-in player compiled with a white picture' ((Test-Path "$PfRun\standin.exe") -and $white -ne $src) "source changed: $($white -ne $src)"

$script:Player = $null
function PlayerLog() { return @(Get-Content "$PfRun\standin.log" -ErrorAction SilentlyContinue) }
function PlayerStart([string[]]$Opts = @()) {
  Remove-Item "$PfRun\standin.log" -ErrorAction SilentlyContinue
  $script:Player = Start-Process "$PfRun\standin.exe" -ArgumentList (@('--log', "$PfRun\standin.log") + $Opts) -PassThru
  for ($i = 0; $i -lt 50 -and -not (PlayerLog | Where-Object { $_ -match ' ready ' }); $i++) { Start-Sleep -Milliseconds 200 }
  PlayerLog | ForEach-Object { Log "player: $_" }
}
function PlayerStop() {
  if (-not $script:Player -or $script:Player.HasExited) { return }
  $e = New-Object Threading.EventWaitHandle($false, 'AutoReset', 'Local\PfLiveQuit'); $e.Set() | Out-Null; $e.Dispose()
  for ($i = 0; $i -lt 25 -and -not $script:Player.HasExited; $i++) { Start-Sleep -Milliseconds 200 }
  if (-not $script:Player.HasExited) { Stop-Process -Id $script:Player.Id -Force }
}
function LiveState() { return "$(@((PfJson status).liveWallpapers)[0].state)" }
function WaitState($want, $sec = 30) {
  for ($i = 0; $i -lt $sec * 2; $i++) { $s = LiveState; if ($s -match $want) { return $s }; Start-Sleep -Milliseconds 500 }
  return (LiveState)
}

# ---- colours (as in owner6-livewall.ps1)
function Lum($rgb) {
  $l = 0.0; $w = @(0.2126, 0.7152, 0.0722)
  for ($i = 0; $i -lt 3; $i++) { $c = $rgb[$i] / 255.0; if ($c -le 0.03928) { $c = $c / 12.92 } else { $c = [math]::Pow(($c + 0.055) / 1.055, 2.4) }; $l += $w[$i] * $c }
  return $l
}
function Contrast($a, $b) { $x = Lum $a; $y = Lum $b; return [math]::Round(([math]::Max($x, $y) + 0.05) / ([math]::Min($x, $y) + 0.05), 2) }
# The ink of the names in a crop: its darkest pixel (dark text) or its lightest (light text).
function InkOf($bmp, [bool]$darkText) {
  $best = $null; $bl = if ($darkText) { 2.0 } else { -1.0 }
  foreach ($c in [Px]::Pixels($bmp)) {
    $rgb = @((($c -shr 16) -band 255), (($c -shr 8) -band 255), ($c -band 255)); $l = Lum $rgb
    if (($darkText -and $l -lt $bl) -or (-not $darkText -and $l -gt $bl)) { $bl = $l; $best = $rgb }
  }
  return $best
}
function Off($a, $b) { return ((0, 1, 2 | ForEach-Object { [math]::Abs($a[$_] - $b[$_]) }) | Measure-Object -Maximum).Maximum }
# What the plate shows: the pane $pane at $alpha over $behind, the glass layer (white, 3 / 255
# at the same share), then the tint at 22 % ($null: none).
function Plate($pane, $alpha, $behind, $tint) {
  return @(0, 1, 2 | ForEach-Object {
      $c = $alpha * $pane[$_] + (1 - $alpha) * $behind[$_]
      $c += (255 - $c) * (3 / 255) * $alpha
      if ($tint) { $c += ($tint[$_] - $c) * 0.22 }
      [int][math]::Round($c)
    })
}
function Rgb($hex) { return @([Convert]::ToInt32($hex.Substring(0, 2), 16), [Convert]::ToInt32($hex.Substring(2, 2), 16), [Convert]::ToInt32($hex.Substring(4, 2), 16)) }
# The fence at $x (300 x 260 at y 150): the mean of its empty lower part, the ink of its names
# and their contrast.
function PaneOf($x, [bool]$darkText, $name) {
  $o = [ordered]@{}
  $o.pane = MeanRgb (Crop "$name-pane" ($x + 20) 330 260 60)
  $o.ink = InkOf (Crop "$name-names" ($x + 10) 250 280 36) $darkText
  $o.contrast = Contrast $o.ink $o.pane
  return $o
}
# The nine 色调 presets of the fence options.
$Tints = [ordered]@{ '红' = 'E74856'; '橙' = 'F7630C'; '黄' = 'FFB900'; '绿' = '10893E'; '青' = '00B7C3'; '蓝' = '0078D4'; '紫' = '8764B8'; '粉' = 'E3008C'; '灰' = '7A7574' }
function SetTint($hex) { Pf fence set Tinted tint "`"#$hex`"" | Out-Null; ParkMouse; Start-Sleep -Milliseconds 1500 }

Case 'setup'
CloseWindows
$names = @('季度报告.txt', '图.txt', '很长很长的文件名称用来换行.txt', '预算表 2026.txt', '会议.txt', '照片合集与备份.txt')
$made = @(); foreach ($n in $names) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue; $made += (DeskFile $n) }
PfStart -Lang 'zh-CN'
Pf settings set featureTips false | Out-Null
Pf settings set themeStyle liquidGlass | Out-Null
Pf settings set theme dark | Out-Null
ClearFences
Pf fence create --title Tinted --rect '430,150,300,260' | Out-Null
Pf fence set Tinted opacity clear | Out-Null
foreach ($n in $names[0..2]) { Pf item move "$PfDesk\$n" --to Tinted | Out-Null }
Pf fence create --title Plain --rect '830,150,300,260' | Out-Null
Pf fence set Plain opacity clear | Out-Null
foreach ($n in $names[3..5]) { Pf item move "$PfDesk\$n" --to Plain | Out-Null }
PlayerStart @('--still')
$s = WaitState 'notCapturable' 40
ParkMouse
Start-Sleep -Seconds 4
$behind = MeanRgb (Crop 'behind' 745 330 40 60)
Check 'a player whose picture cannot be read shows a white wallpaper behind the Liquid Glass fences' ($s -eq 'notCapturable' -and (Off $behind @(255, 255, 255)) -le 3) "state=$s, wallpaper beside the fence $($behind -join ' ')"
$Dark = @(33, 41, 53); $Light = @(244, 246, 252)

Case 'dark mode, 更透明: every preset tint'
$rows = @(); $worst = 99.0; $worstName = ''
foreach ($t in $Tints.Keys) {
  SetTint $Tints[$t]
  $p = PaneOf 430 $false "dark-clear-$($Tints[$t])"
  $rows += "$t $($p.contrast) (pane $($p.pane -join ' '))"
  if ($p.contrast -lt $worst) { $worst = $p.contrast; $worstName = $t }
}
Log ("dark, 更透明, names against the tinted pane over white: " + ($rows -join '; '))
Check 'dark mode, 更透明, over a white wallpaper: the names read (4.5 to 1) on the pane with each of the nine tints' ($worst -ge 4.5) "lowest: $worstName $worst to 1; $($rows -join '; ')"
SetTint 'FFB900'
$y = PaneOf 430 $false 'dark-clear-yellow'
Crop 'pane-dark-yellow-clear' 400 120 780 320 | Out-Null
$want74 = Plate $Dark 0.74 $behind (Rgb 'FFB900'); $want65 = Plate $Dark 0.654 $behind (Rgb 'FFB900')
Check 'dark mode, 更透明, 黄: the pane is 74 % under the tint (65 % gave 3.7 to 1)' ((Off $y.pane $want74) -le 8 -and $y.contrast -ge 4.5) "pane $($y.pane -join ' '), at 74 %: $($want74 -join ' '), at 65 %: $($want65 -join ' '); names $($y.contrast) to 1 (ink $($y.ink -join ' '))"
$plain = PaneOf 830 $false 'dark-clear-plain'
$wantPlain = Plate $Dark 0.654 $behind $null
Check 'dark mode, 更透明, no tint: the pane stays 65 % and the names read' ((Off $plain.pane $wantPlain) -le 8 -and $plain.contrast -ge 4.5) "pane $($plain.pane -join ' '), expected $($wantPlain -join ' '); names $($plain.contrast) to 1"

Case 'dark mode, 默认'
Pf fence set Tinted opacity default | Out-Null
Pf fence set Plain opacity default | Out-Null
ParkMouse; Start-Sleep -Milliseconds 2000
$yd = PaneOf 430 $false 'dark-default-yellow'; $pd = PaneOf 830 $false 'dark-default-plain'
Crop 'pane-dark-yellow-default' 400 120 780 320 | Out-Null
$wantY = Plate $Dark 0.78 $behind (Rgb 'FFB900'); $wantP = Plate $Dark 0.78 $behind $null
Check 'dark mode, 默认: the pane stays 78 % with and without a tint, the names read on both' ((Off $yd.pane $wantY) -le 8 -and (Off $pd.pane $wantP) -le 8 -and $yd.contrast -ge 4.5 -and $pd.contrast -ge 4.5) "黄: pane $($yd.pane -join ' ') (expected $($wantY -join ' ')), names $($yd.contrast) to 1; no tint: pane $($pd.pane -join ' ') (expected $($wantP -join ' ')), names $($pd.contrast) to 1"

Case 'light mode, 更透明'
Pf fence set Tinted opacity clear | Out-Null
Pf fence set Plain opacity clear | Out-Null
Pf settings set theme light | Out-Null
ParkMouse; Start-Sleep -Seconds 5
$yl = PaneOf 430 $true 'light-clear-yellow'
Crop 'pane-light-yellow-clear' 400 120 780 320 | Out-Null
$wantL = Plate $Light 0.654 $behind (Rgb 'FFB900')
Check 'light mode, 更透明, 黄: the pane stays 65 % under the tint, the names read' ((Off $yl.pane $wantL) -le 8 -and $yl.contrast -ge 4.5) "pane $($yl.pane -join ' '), expected $($wantL -join ' '); names $($yl.contrast) to 1 (ink $($yl.ink -join ' '))"

Case 'cleanup'
PlayerStop
Get-Process standin -ErrorAction SilentlyContinue | Stop-Process -Force
PfQuit
foreach ($p in $made) { Remove-Item -LiteralPath $p -Force -ErrorAction SilentlyContinue }
Check 'the player ended and the test files are gone' (-not (Get-Process standin -ErrorAction SilentlyContinue) -and -not (Test-Path -LiteralPath $made[0])) ''
