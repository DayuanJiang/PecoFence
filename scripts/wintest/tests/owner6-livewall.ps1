# Liquid Glass over a live wallpaper whose picture cannot be read (Wallpaper Engine's scene
# wallpapers; assets/live-wallpaper-standin.cs stands in for the player).
#   run.sh tests/owner6-livewall.ps1                         (Part A, release build)
#     the pane: a see-through pane in the mode's colour, 78 % at 默认 and 65 % at 更透明, names
#       readable on it, light and dark mode; the clear glass comes back when the player ends
#     the card: comes once, 「保留 Liquid Glass」 keeps the look, not again for the same player
#       (a second start of the player, a restart of PecoFence), 「改用 Fluent」 switches the look
#   run.sh --var Part=C tests/owner6-livewall.ps1            (release build, about 3 minutes)
#     the card leaves once what it says no longer holds (Fluent chosen elsewhere, the player
#       ended) and is not kept as answered; it waits while no fence shows the pane (only a
#       更厚实 or a 全透明 fence, during a Peek)
#   run.sh --bin target/wintest-bin-debug --var Part=P ...   (settings page over CDP)
#     the line under 主题风格 shows for an unreadable and for a readable player and goes with
#       the player and with Fluent
# --var Lng=en runs it in English (default zh-CN: the report's pictures).
if (-not $Part) { $Part = 'A' }
if (-not $Lng) { $Lng = 'zh-CN' }
$Zh = $Lng -eq 'zh-CN'
$TCard = if ($Zh) { 'Liquid Glass 换成了半透明玻璃' } else { 'Liquid Glass is now translucent glass' }

# ---- the stand-in player
Get-Process standin -ErrorAction SilentlyContinue | Stop-Process -Force
$cp = New-Object CodeDom.Compiler.CompilerParameters
$cp.GenerateExecutable = $true; $cp.OutputAssembly = "$PfRun\standin.exe"
$cp.CompilerOptions = '/target:winexe /win32manifest:C:\pf\kit\assets\live-wallpaper-standin.manifest'
$cp.ReferencedAssemblies.Add('System.dll') | Out-Null
Add-Type -TypeDefinition (Get-Content 'C:\pf\kit\assets\live-wallpaper-standin.cs' -Raw) -CompilerParameters $cp
Check 'the stand-in player compiled' (Test-Path "$PfRun\standin.exe") "Windows build $([Environment]::OSVersion.Version.Build)"

$script:Player = $null
function PlayerLog() { return @(Get-Content "$PfRun\standin.log" -ErrorAction SilentlyContinue) }
function PlayerStart([string[]]$Opts = @()) {
  Remove-Item "$PfRun\standin.log" -ErrorAction SilentlyContinue
  $script:Player = Start-Process "$PfRun\standin.exe" -ArgumentList (@('--log', "$PfRun\standin.log") + $Opts) -PassThru
  for ($i = 0; $i -lt 50 -and -not (PlayerLog | Where-Object { $_ -match ' ready ' }); $i++) { Start-Sleep -Milliseconds 200 }
  PlayerLog | ForEach-Object { Log "player: $_" }
}
function PlayerSignal($name) { $e = New-Object Threading.EventWaitHandle($false, 'AutoReset', "Local\$name"); $e.Set() | Out-Null; $e.Dispose() }
function PlayerStop() {
  if (-not $script:Player -or $script:Player.HasExited) { return }
  PlayerSignal 'PfLiveQuit'
  for ($i = 0; $i -lt 25 -and -not $script:Player.HasExited; $i++) { Start-Sleep -Milliseconds 200 }
  if (-not $script:Player.HasExited) { Stop-Process -Id $script:Player.Id -Force }
}
function LiveState() { return "$(@((PfJson status).liveWallpapers)[0].state)" }
function WaitState($want, $sec = 30) {
  for ($i = 0; $i -lt $sec * 2; $i++) { $s = LiveState; if ($s -match $want) { return $s }; Start-Sleep -Milliseconds 500 }
  return (LiveState)
}

# ---- cards (as in fence-menu-and-tips.ps1)
function NoticeWin() {
  $n = [G]::FindWindowW('PecoFence.Notice', [NullString]::Value)
  if ($n -ne [IntPtr]::Zero -and [G]::IsWindowVisible($n)) { return $n }
  return [IntPtr]::Zero
}
function WaitNotice($sec = 6) {
  for ($i = 0; $i -lt $sec * 5; $i++) { $n = NoticeWin; if ($n -ne [IntPtr]::Zero) { Start-Sleep -Milliseconds 400; return $n }; Start-Sleep -Milliseconds 200 }
  return [IntPtr]::Zero
}
function NoticeTitle() { $n = NoticeWin; if ($n -eq [IntPtr]::Zero) { return '' }; return [L]::Text($n) }
# The accent button is at the card's right, the plain one left of it.
function ClickAccent() { $r = Rect (NoticeWin); [G]::Click($r.R - 46, $r.B - 32); Start-Sleep -Milliseconds 900 }
function ClickPlain() { $r = Rect (NoticeWin); [G]::Click($r.R - 170, $r.B - 32); Start-Sleep -Milliseconds 900 }
function ShotNotice($name) { $r = Rect (NoticeWin); if ($r.R -gt $r.L) { Crop $name ($r.L - 12) ($r.T - 12) ($r.R - $r.L + 24) ($r.B - $r.T + 24) | Out-Null } }
# Starts the build like PfStart, without --test-script: cards may show.
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
function NoticesPath() { return (Join-Path (Split-Path ((PfJson paths).config)) 'notices.json') }
function NoticesJson() { $p = NoticesPath; if (Test-Path $p) { return (Get-Content $p -Raw) -replace '\s+', ' ' }; return '(none)' }

# ---- colours
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
# What a pane of colour $pane drawn at $alpha shows over $behind.
function Over($pane, $alpha, $behind) { return @(0, 1, 2 | ForEach-Object { [int][math]::Round($alpha * $pane[$_] + (1 - $alpha) * $behind[$_]) }) }
function Off($a, $b) { return ((0, 1, 2 | ForEach-Object { [math]::Abs($a[$_] - $b[$_]) }) | Measure-Object -Maximum).Maximum }
# The fence 'Glass' (默认) lies in the player's second band, 'Clear' (更透明) in its third; the
# plates' empty lower part, the wallpaper right of each fence, the row of names.
function Panes($name) {
  $o = [ordered]@{}
  $o.glass = MeanRgb (Crop "$name-glass" 450 330 260 60)
  $o.clear = MeanRgb (Crop "$name-clear" 850 330 260 60)
  $o.behindGlass = MeanRgb (Crop "$name-behind-glass" 745 330 40 60)
  $o.behindClear = MeanRgb (Crop "$name-behind-clear" 1145 330 40 60)
  $o.namesGlass = Crop "$name-names-glass" 440 250 280 36
  $o.namesClear = Crop "$name-names-clear" 840 250 280 36
  Crop "pane-$name" 400 120 780 320 | Out-Null
  Log "PANES $name state=$(LiveState) glass=$($o.glass -join ' ') over $($o.behindGlass -join ' '); clear=$($o.clear -join ' ') over $($o.behindClear -join ' ')"
  return $o
}
# The see-through pane of one mode: colour, share, names.
function CheckPane($mode, $pane) {
  $darkText = $mode -eq 'light'
  $p = Panes $mode
  $wantGlass = Over $pane 0.78 $p.behindGlass; $wantClear = Over $pane 0.654 $p.behindClear
  Check "$mode, 默认: the pane is the mode's colour at 78 % over the wallpaper behind it" ((Off $p.glass $wantGlass) -le 10) "pane $($p.glass -join ' '), expected $($wantGlass -join ' ') over $($p.behindGlass -join ' ')"
  Check "$mode, 更透明: the pane is the mode's colour at 65 % over the wallpaper behind it" ((Off $p.clear $wantClear) -le 10) "pane $($p.clear -join ' '), expected $($wantClear -join ' ') over $($p.behindClear -join ' ')"
  $inkG = InkOf $p.namesGlass $darkText; $inkC = InkOf $p.namesClear $darkText
  $cg = Contrast $inkG $p.glass; $cc = Contrast $inkC $p.clear
  # The worst wallpaper for this ink: black under the light pane, white under the dark one.
  $worst = if ($darkText) { @(0, 0, 0) } else { @(255, 255, 255) }
  $wg = Contrast $inkG (Over $pane 0.78 $worst); $wc = Contrast $inkC (Over $pane 0.654 $worst)
  Check "${mode}: the names read on both panes (4.5 to 1), over the worst wallpaper too" ($cg -ge 4.5 -and $cc -ge 4.5 -and $wg -ge 4.5 -and $wc -ge 4.5) "默认 $cg (ink $($inkG -join ' ')), 更透明 $cc (ink $($inkC -join ' ')); over the worst wallpaper: 默认 $wg, 更透明 $wc"
}

# ---- setup shared by both parts
Case 'setup'
CloseWindows
$old = (Get-ItemProperty 'HKCU:\Control Panel\Desktop').WallPaper
Log "wallpaper before: $old"
# A picture with detail as the Windows wallpaper: a gradient, circles and stripes.
$bmp = New-Object Drawing.Bitmap 1600, 900; $g = [Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = 'AntiAlias'
$br = New-Object Drawing.Drawing2D.LinearGradientBrush((New-Object Drawing.Point 0, 0), (New-Object Drawing.Point 1600, 900), [Drawing.Color]::FromArgb(30, 90, 200), [Drawing.Color]::FromArgb(250, 150, 60))
$g.FillRectangle($br, 0, 0, 1600, 900)
for ($i = 0; $i -lt 14; $i++) { $g.FillEllipse((New-Object Drawing.SolidBrush ([Drawing.Color]::FromArgb(200, 255, 255, 255))), 140 + $i * 110, 160 + (($i * 67) % 240), 70, 70) }
$g.Dispose(); $bmp.Save("$PfRun\picture.bmp", [Drawing.Imaging.ImageFormat]::Bmp); $bmp.Dispose()
[G]::SystemParametersInfoW(0x14, 0, "$PfRun\picture.bmp", 3) | Out-Null
$names = @('季度报告.txt', '图.txt', '很长很长的文件名称用来换行.txt', '预算表 2026.txt', '会议.txt', '照片合集与备份.txt')
$made = @(); foreach ($n in $names) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue; $made += (DeskFile $n) }
if ($Part -eq 'P' -and $PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
PfStart -Lang $Lng
Pf settings set themeStyle liquidGlass | Out-Null
Pf settings set theme light | Out-Null
ClearFences
$gx = if ($Part -eq 'P') { 60 } else { 430 }
Pf fence create --title Glass --rect "$gx,150,300,260" | Out-Null
foreach ($n in $names[0..2]) { Pf item move "$PfDesk\$n" --to Glass | Out-Null }
if ($Part -eq 'A') {
  Pf fence create --title Clear --rect '830,150,300,260' | Out-Null
  Pf fence set Clear opacity clear | Out-Null
  foreach ($n in $names[3..5]) { Pf item move "$PfDesk\$n" --to Clear | Out-Null }
}
ParkMouse
Start-Sleep -Seconds 3
Check 'Liquid Glass fences are up' ((Pf settings get themeStyle) -match 'liquidGlass' -and (FenceGet 'Glass').title -eq 'Glass') "$((PfJson fence list | ForEach-Object { $_.title }) -join ', ')"

if ($Part -eq 'A') {
Case 'no player'
PfStartPlain
ParkMouse
Start-Sleep -Seconds 4
$clearGlass = Panes 'no-player'
Check 'no player: clear glass, no card' ((LiveState) -eq 'none' -and (NoticeWin) -eq [IntPtr]::Zero) "state=$(LiveState) card='$(NoticeTitle)'"

Case 'the pane and the card, light mode'
PlayerStart @('--still')
$s = WaitState 'notCapturable' 40
$n = WaitNotice 15
Check 'a player whose picture cannot be read: one card says so' ($s -eq 'notCapturable' -and (NoticeTitle) -eq $TCard -and @([W]::Tops('PecoFence.Notice')).Count -eq 1) "state=$s card='$(NoticeTitle)' cards=$(@([W]::Tops('PecoFence.Notice')).Count)"
ShotNotice "card-light-$Lng"
Shot "screen-light-$Lng"
ParkMouse
Start-Sleep -Seconds 1
CheckPane 'light' @(244, 246, 252)
ClickPlain
Check '保留 Liquid Glass closes the card and keeps the look' ((WaitGoneWin 'PecoFence.Notice' 4) -and (Pf settings get themeStyle) -match 'liquidGlass') "card='$(NoticeTitle)' themeStyle=$(Pf settings get themeStyle)"
Check 'notices.json remembers the player' ((NoticesJson) -match '"liveWallpaperTold": \[ "standin.exe" \]') (NoticesJson)

Case 'the pane, dark mode'
Pf settings set theme dark | Out-Null
ParkMouse
Start-Sleep -Seconds 5
CheckPane 'dark' @(33, 41, 53)
Check 'no second card for the same player' ((NoticeWin) -eq [IntPtr]::Zero) "card='$(NoticeTitle)'"

Case 'the player ends and starts again'
PlayerStop
$s = WaitState 'none' 20
Start-Sleep -Seconds 4
$back = Panes 'player-ended'
Pf settings set theme light | Out-Null
Start-Sleep -Seconds 4
$back = Panes 'player-ended-light'
Check 'the player ended: the clear glass is back' ($s -eq 'none' -and (Off $back.glass $clearGlass.glass) -le 6 -and (Off $back.clear $clearGlass.clear) -le 6) "state=$s glass $($back.glass -join ' ') (before $($clearGlass.glass -join ' ')), clear $($back.clear -join ' ') (before $($clearGlass.clear -join ' '))"
PlayerStart @('--still')
$s = WaitState 'notCapturable' 40
Start-Sleep -Seconds 10
Check 'the same player again: the pane, no card' ($s -eq 'notCapturable' -and (NoticeWin) -eq [IntPtr]::Zero) "state=$s card='$(NoticeTitle)'"
PfStartPlain
$s = WaitState 'notCapturable' 40
Start-Sleep -Seconds 10
Check 'PecoFence started again: no card' ($s -eq 'notCapturable' -and (NoticeWin) -eq [IntPtr]::Zero) "state=$s card='$(NoticeTitle)'"

Case 'switch to Fluent'
# A player the card was never answered for: the record is emptied.
$np = NoticesPath
PfQuit
(Get-Content $np -Raw) -replace '"standin.exe"', '' | Set-Content $np -Encoding ASCII
PfStartPlain
ParkMouse
$s = WaitState 'notCapturable' 40
$n = WaitNotice 15
Check 'the card comes for a player it was not answered for' ((NoticeTitle) -eq $TCard) "state=$s card='$(NoticeTitle)' notices=$(NoticesJson)"
ClickAccent
Start-Sleep -Seconds 2
Check '改用 Fluent switches the look to Fluent and closes the card' ((Pf settings get themeStyle) -match 'fluent' -and (WaitGoneWin 'PecoFence.Notice' 4)) "themeStyle=$(Pf settings get themeStyle) card='$(NoticeTitle)'"
Crop "fluent-$Lng" 400 120 780 320 | Out-Null
Start-Sleep -Seconds 8
Check 'Fluent: no card comes' ((NoticeWin) -eq [IntPtr]::Zero -and (NoticesJson) -match 'standin.exe') "card='$(NoticeTitle)' notices=$(NoticesJson)"
Log (@(PfLogAll | Where-Object { $_ -match 'live wallpaper card|LiveWallpaper|not readable; Liquid Glass' }) -join "`n")
}

if ($Part -eq 'C') {
# The card says "the fences are see-through panes now, use Fluent instead?". It leaves once that
# no longer holds, without being kept as answered, and it comes only while a fence shows the pane.
function CardWaits() { return "$(@(PfLogAll | Where-Object { $_ -match 'live wallpaper card due' }) | Select-Object -Last 1)" -replace '^\S+\s+\S+\s+\S+\s+', '' }
function WaitNoNotice($sec = 12) {
  for ($i = 0; $i -lt $sec * 5; $i++) { if ((NoticeWin) -eq [IntPtr]::Zero) { return $true }; Start-Sleep -Milliseconds 200 }
  return $false
}
Case 'the card leaves once what it says no longer holds'
PfStartPlain
ParkMouse
PlayerStart @('--still')
$s = WaitState 'notCapturable' 40
$n = WaitNotice 15
Check 'the card is up over the see-through pane' ($s -eq 'notCapturable' -and (NoticeTitle) -eq $TCard) "state=$s card='$(NoticeTitle)'"
Pf settings set themeStyle fluent | Out-Null
Start-Sleep -Seconds 3
Shot "fluent-chosen-$Lng"
Check 'Fluent chosen elsewhere: the card leaves at once, not kept as answered' ((NoticeWin) -eq [IntPtr]::Zero -and (NoticesJson) -notmatch 'standin.exe') "card='$(NoticeTitle)' notices=$(NoticesJson)"
Pf settings set themeStyle liquidGlass | Out-Null
$n = WaitNotice 20
Check 'Liquid Glass again: the card is back (it was not answered)' ((NoticeTitle) -eq $TCard) "card='$(NoticeTitle)'"
PlayerStop
$gone = WaitNoNotice 14
Shot "player-ended-$Lng"
Check 'the player ended: the card leaves, the look stays, not kept as answered' ($gone -and (Pf settings get themeStyle) -match 'liquidGlass' -and (NoticesJson) -notmatch 'standin.exe') "card='$(NoticeTitle)' themeStyle=$(Pf settings get themeStyle) notices=$(NoticesJson)"
Log (@(PfLogAll | Where-Object { $_ -match 'live wallpaper card' }) -join "`n")

Case 'the card waits while no fence shows the pane'
Pf fence set Glass opacity solid | Out-Null
# The card of the case above may have timed out meanwhile on a build that kept it: start with
# no player answered for.
$np = NoticesPath
PfQuit
if (Test-Path $np) { (Get-Content $np -Raw) -replace '"standin.exe"', '' | Set-Content $np -Encoding ASCII }
PfStartPlain
ParkMouse
PlayerStart @('--still')
$s = WaitState 'notCapturable' 40
Start-Sleep -Seconds 12
Shot "solid-only-$Lng"
Check 'only a 更厚实 fence (its glass stays opaque): no card' ($s -eq 'notCapturable' -and (NoticeWin) -eq [IntPtr]::Zero) "state=$s card='$(NoticeTitle)' | $(CardWaits)"
Pf fence set Glass opacity transparent | Out-Null
Start-Sleep -Seconds 8
Check 'only a 全透明 fence (no glass at rest): no card' ((NoticeWin) -eq [IntPtr]::Zero) "card='$(NoticeTitle)' | $(CardWaits)"
# During a Peek the glass is opaque: the fence at 默认 does not count until the Peek ends.
[G]::MoveTo(1300, 700)
PeekKey
Start-Sleep -Milliseconds 1200
Pf fence set Glass opacity default | Out-Null
Start-Sleep -Seconds 8
$dim = [G]::FindWindowW('PecoFence.PeekDimmer', [NullString]::Value)
$held = $dim -ne [IntPtr]::Zero -and [G]::IsWindowVisible($dim)
Check 'during a Peek: no card' ($held -and (NoticeWin) -eq [IntPtr]::Zero) "peek up=$held card='$(NoticeTitle)' | $(CardWaits)"
PeekKey
Start-Sleep -Milliseconds 800
ParkMouse
$n = WaitNotice 20
ShotNotice "card-after-wait-$Lng"
Check 'the Peek ended, the fence at 默认 shows the pane: the card comes' ((NoticeTitle) -eq $TCard) "card='$(NoticeTitle)' | $(CardWaits)"
ClickPlain
Check 'answered with 保留 Liquid Glass: kept in notices.json as before' ((WaitGoneWin 'PecoFence.Notice' 4) -and (NoticesJson) -match 'standin.exe') "card='$(NoticeTitle)' notices=$(NoticesJson)"
Log (@(PfLogAll | Where-Object { $_ -match 'live wallpaper card' }) -join "`n")
}

if ($Part -eq 'P') {
Case 'the settings line'
function Note() { return (Js "(() => { const n = document.getElementById('liveWallpaperNote'); return !n ? '(no element)' : n.hidden ? '' : n.textContent; })()") }
function WaitNote($regex, $sec = 20) {
  for ($i = 0; $i -lt $sec * 2; $i++) { $t = Note; if ("$t" -match $regex) { return "$t" }; Start-Sleep -Milliseconds 500 }
  return "$(Note)"
}
function ShotLine($name) {
  $r = PageRect '#desktopPreview'
  if ($r) { Crop $name ($r.x - 60) ($r.y - 50) ($r.w + 100) ($r.h + 110) | Out-Null }
}
$ok = PfSettings
$sw = [G]::FindWindowW('PecoFence.Settings', [NullString]::Value)
# Beside the fence: a live wallpaper is read only where a fence shows.
[L]::MoveWindow($sw, 560, 30, 1000, 800, $true) | Out-Null
Start-Sleep -Seconds 1
Check 'settings open over CDP; no line without a player' ($ok -and (Note) -eq '') "debug=$PfDebugBuild note='$(Note)'"
PlayerStart @('--still')
$s = WaitState 'notCapturable' 40
$t = WaitNote '.' 20
$want = if ($Zh) { '^这种动态壁纸的画面读不到，Liquid Glass 栅栏现在是半透明玻璃' } else { '^The picture of this kind of live wallpaper cannot be read' }
Check 'a player whose picture cannot be read: the line under 主题风格 says so' ($s -eq 'notCapturable' -and $t -match $want) "state=$s note='$t'"
ShotLine "line-unreadable-$Lng"
PlayerStop
$t = WaitNote '^$' 20
Check 'the player ended: the line goes' ($t -eq '' -and (WaitState 'none' 10) -eq 'none') "note='$t' state=$(LiveState)"
PlayerStart @('--gdi')
$s = WaitState 'captured' 40
$t = WaitNote '.' 20
$want = if ($Zh) { '磨砂玻璃.*“更透明”' } else { 'frosted glass.*More transparent' }
Check 'a player whose picture can be read: the line names 更透明' ($s -eq 'captured' -and $t -match $want) "state=$s note='$t'"
ShotLine "line-readable-$Lng"
Pf settings set themeStyle fluent | Out-Null
$t = WaitNote '^$' 10
Check 'Fluent: no line' ($t -eq '') "note='$t'"
Pf settings set themeStyle liquidGlass | Out-Null
$t = WaitNote '.' 20
Check 'Liquid Glass again: the line is back' ($t -match $want) "note='$t'"
PlayerStop
$t = WaitNote '^$' 20
Check 'the readable player ended: the line goes' ($t -eq '') "note='$t'"
}

Case 'cleanup'
PlayerStop
Get-Process standin -ErrorAction SilentlyContinue | Stop-Process -Force
PfQuit
foreach ($p in $made) { Remove-Item -LiteralPath $p -Force -ErrorAction SilentlyContinue }
[G]::SystemParametersInfoW(0x14, 0, $old, 3) | Out-Null
Check 'the player ended, the wallpaper is as it was, the test files are gone' (-not (Get-Process standin -ErrorAction SilentlyContinue) -and (Get-ItemProperty 'HKCU:\Control Panel\Desktop').WallPaper -eq $old -and -not (Test-Path -LiteralPath $made[0])) "wallpaper=$old"
