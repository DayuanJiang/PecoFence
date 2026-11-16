# 插件的媒体接口 in the Store edition's shape: the build under test registered as a package of
# its own (packaging/msix/AppxManifest.xml with a test identity, no startup task and no alias,
# loose files: Add-AppxPackage -Register) runs with package identity,
# full trust and medium integrity as the Store build does. The now-playing plugin of
# assets/now-playing shows what Edge plays. Everything is removed at the end. Either build.
# The job is no administrator: developer mode goes on before and off after, over ssh:
#   box.sh ssh <box> 'New-Item HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock -Force | Out-Null; Set-ItemProperty HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock AllowDevelopmentWithoutDevLicense 1 -Type DWord'
#   run.sh --extra packaging/msix scripts/wintest/tests/plugin-media-store.ps1
#   box.sh ssh <box> 'Remove-ItemProperty HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock AllowDevelopmentWithoutDevLicense'
$Kit = 'C:\pf\kit\assets'
$Pkg = "$PfRun\pkg"
$Data = "$env:APPDATA\PecoFence"
$EdgeExe = "${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe"
function PkgCli { $env:PECOFENCE_INSTANCE = ''; return (& "$Pkg\pecofence-cli.exe" @args 2>&1 | Out-String) }
function FenceText($id) { $j = (PkgCli fence get $id) | ConvertFrom-Json; if ($j.fence) { $j = $j.fence }; return "$($j.script.output)" }
function WaitText($id, $pattern, $sec) {
  for ($k = 0; $k -lt $sec * 2; $k++) { $t = FenceText $id; if ($t -match $pattern) { return $t }; Start-Sleep -Milliseconds 500 }
  return (FenceText $id)
}

Case 'setup: the build as a package with its own identity'
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
$dev = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock' -ErrorAction SilentlyContinue).AllowDevelopmentWithoutDevLicense
$fresh = -not (Test-Path $Data)
Check 'developer mode is on, and no PecoFence data is in AppData yet' ($dev -eq 1 -and $fresh) "developer mode=$dev fresh=$fresh"
New-Item -ItemType Directory -Force "$Pkg\Assets" | Out-Null
Copy-Item "$PfBin\*" $Pkg -Force
Add-Type -AssemblyName System.Drawing
$bmp = New-Object Drawing.Bitmap 150, 150; $g = [Drawing.Graphics]::FromImage($bmp); $g.Clear([Drawing.Color]::RoyalBlue); $g.Dispose()
foreach ($n in 'StoreLogo', 'Square150x150Logo', 'Square44x44Logo', 'Wide310x150Logo', 'SmallTile', 'LargeTile', 'SplashScreen') { $bmp.Save("$Pkg\Assets\$n.png", [Drawing.Imaging.ImageFormat]::Png) }
$bmp.Dispose()
$m = [IO.File]::ReadAllText("$PfExtra\AppxManifest.xml")
$m = $m -replace '__IDENTITY_NAME__', 'PecoFence.MediaTest' -replace '__IDENTITY_PUBLISHER__', 'CN=PecoFenceTest' -replace '__VERSION__', '0.5.0.0' -replace '__PUBLISHER_DISPLAY_NAME__', 'PecoFence test'
$m = [regex]::Replace($m, '(?s)<Extensions>.*</Extensions>', '')
[IO.File]::WriteAllText("$Pkg\AppxManifest.xml", $m)
# Placed before the first start: the package then keeps its data in the real AppData folder.
$to = "$Data\plugins\now-playing\1.0.0"; New-Item -ItemType Directory -Force $to | Out-Null
Copy-Item "$Kit\now-playing\*" $to -Force
Set-Content "$Data\plugins\now-playing\current.txt" '1.0.0' -Encoding ASCII -NoNewline
$err = $null
try { Add-AppxPackage -Register "$Pkg\AppxManifest.xml" -ErrorAction Stop } catch { $err = "$_" }
$p = Get-AppxPackage PecoFence.MediaTest
Check 'registered' ([bool]$p -and -not $err) "$err"
Start-Process explorer.exe "shell:AppsFolder\$($p.PackageFamilyName)!PecoFence"
$h = WaitWin 'PecoFence.Control' 30 -Hidden
Start-Sleep -Seconds 5
$proc = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
Check 'it runs from the package' ($h -ne [IntPtr]::Zero -and $proc -and $proc.Path -like "$Pkg*") "control=$h path=$($proc.Path)"

Case 'the package reads the media session'
PkgCli settings set language en | Out-Null
Set-Content "$PfRun\idle.cmd" '@echo off' -Encoding ASCII
$f = (PkgCli fence create --title Store --script "$PfRun\idle.cmd" --rect '80,60,300,440') | ConvertFrom-Json
$fid = "$(if ($f.fence) { $f.fence.id } else { $f.id })"
PkgCli fence set $fid scriptPath 'plugin:now-playing' | Out-Null
$t = WaitText $fid 'Nothing is playing' 20
Check 'the plugin runs in the package: Nothing is playing' ($t -match 'Nothing is playing') $t
New-Item -ItemType Directory -Force "$PfRun\media-page" | Out-Null
Copy-Item "$Kit\media-page\*" "$PfRun\media-page" -Force
$rate = 8000; $n = $rate * 180 * 2; $wav = New-Object byte[] (44 + $n)
$parts = @(@(0, [Text.Encoding]::ASCII.GetBytes('RIFF')), @(4, [BitConverter]::GetBytes([int](36 + $n))), @(8, [Text.Encoding]::ASCII.GetBytes('WAVEfmt ')),
  @(16, [BitConverter]::GetBytes([int]16)), @(20, [BitConverter]::GetBytes([int16]1)), @(22, [BitConverter]::GetBytes([int16]1)),
  @(24, [BitConverter]::GetBytes([int]$rate)), @(28, [BitConverter]::GetBytes([int]($rate * 2))), @(32, [BitConverter]::GetBytes([int16]2)),
  @(34, [BitConverter]::GetBytes([int16]16)), @(36, [Text.Encoding]::ASCII.GetBytes('data')), @(40, [BitConverter]::GetBytes([int]$n)))
foreach ($q in $parts) { [Array]::Copy($q[1], 0, $wav, $q[0], $q[1].Length) }
[IO.File]::WriteAllBytes("$PfRun\media-page\tone.wav", $wav)
$url = 'file:///' + ("$PfRun\media-page" -replace '\\', '/') + '/play.html'
Start-Process $EdgeExe -ArgumentList @('--no-first-run', '--no-default-browser-check', "--user-data-dir=$PfRun\edge-media", '--autoplay-policy=no-user-gesture-required', '--window-position=760,60', '--window-size=640,420', $url) | Out-Null
$t = WaitText $fid 'PecoFence test tone' 30
Check 'the packaged build shows what Edge plays, with its time' ($t -match 'PecoFence test tone' -and $t -match '\d+:\d\d / 3:00') $t
$covers = @(Get-ChildItem "$Data\plugins-data\now-playing\media-cover-*" -ErrorAction SilentlyContinue)
$covers += @(Get-ChildItem "$env:LOCALAPPDATA\Packages\$($p.PackageFamilyName)\LocalCache\Roaming\PecoFence\plugins-data\now-playing\media-cover-*" -ErrorAction SilentlyContinue)
Check 'and its cover' ($covers.Count -ge 1) "covers=$($covers.FullName -join ', ')"

Case 'cleanup: the package and its data'
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
PfQuit
Start-Sleep -Seconds 3
Get-Process pecofence, pecofence-watchdog -ErrorAction SilentlyContinue | Where-Object { $_.Path -like "$Pkg*" } | Stop-Process -Force -ErrorAction SilentlyContinue
if ($p) { Remove-AppxPackage $p.PackageFullName -ErrorAction SilentlyContinue }
if ($fresh) { Remove-Item $Data -Recurse -Force -ErrorAction SilentlyContinue }
Check 'the package is gone' (-not (Get-AppxPackage PecoFence.MediaTest)) ''
