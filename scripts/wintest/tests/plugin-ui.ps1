# 插件 interface 2, plan v6 rows 1, 3 and 4 and the owner's decision 3 A (colours), with the test
# plugin assets\plugin-ui\uidemo. A right-click on a row with a menu of its own shows its entries
# (and the fence's otherwise), and a choice sends command {id, via: "menu", row}; a box of
# several lines (Enter is a new line; Ctrl+Enter sends change and submit; Esc and a click
# elsewhere send change; at most `max` characters; a restart waits while it is open); a row's
# look under the pointer; enter and leave of hover: "event" rows (at most 10 a second, the last
# row told); the wheel over wheel: true cells and rows goes to the plugin and the fence stays,
# elsewhere it scrolls. Then 设置 › 小工具: plain 安装 buttons, grey tags (启动程序 yellow), a grey
# warning for a plugin that asks for nothing; and the gray text on light-text fences (#C5C5C5).
# Debug build (CDP on the settings page), English interface (--var Lang=zh-CN for Chinese). Run it on the build before too
# for the pictures to compare: its new checks fail there.
#   run.sh --bin target/wintest-bin-debug scripts/wintest/tests/plugin-ui.ps1
$Feed = 'C:\pf\run\uifeed'
# --var Lang=zh-CN: the Chinese interface, for the pictures.
if (-not (Get-Variable Lang -ErrorAction SilentlyContinue)) { $Lang = 'en' }

function CellRects($w) {
  $v = DumpVal $w 'script_cells'
  if (-not $v -or $v -eq '-') { return @() }
  return @($v -split ';' | ForEach-Object {
    $pair = $_ -split ':'; $n = $pair[1] -split ','
    [pscustomobject]@{ i = [int]$pair[0]; l = [double]$n[0]; t = [double]$n[1]; r = [double]$n[2]; b = [double]$n[3] }
  })
}
function Hit($row, $cell = 0) { return @(CellRects (DumpOf 'UI' 5) | Where-Object { $_.i -eq $row * 100 + $cell }) | Select-Object -First 1 }
function ContentPt($x, $y) {
  $w = DumpOf 'UI' 5; $sc = [double](DumpVal $w 'content_y') / 36.0
  return @([int]($w.l + [double](DumpVal $w 'content_x') + $x * $sc), [int]($w.t + [double](DumpVal $w 'content_y') + $y * $sc))
}
function HitPt($row, $cell = 0) { $r = Hit $row $cell; if (-not $r) { Log "no hit $row/$cell"; return $null }; return ContentPt (($r.l + $r.r) / 2) (($r.t + $r.b) / 2) }
# Clicks (or right-clicks) the middle of row `row`, cell `cell`; nothing when it is not drawn there.
function ClickHit($row, $cell = 0, [switch]$Right) {
  $p = HitPt $row $cell
  if (-not $p) { return }
  if ($Right) { [G]::RClick($p[0], $p[1]) } else { [G]::Click($p[0], $p[1]) }
}
# The text of the edit in the open text box.
function BoxText($box) {
  $script:BoxT = ''
  [L]::EnumChildWindows($box, { param($h, $l) if ([L]::Cls($h) -eq 'Edit') { $script:BoxT = [W]::Text($h) }; $true }, [IntPtr]::Zero) | Out-Null
  return $script:BoxT
}
function Out() { return (FenceOut $script:Ui) }
function Ev() { return @((Out) -split "`n" | Where-Object { $_ -like 'event:*' })[0] }
function Counts() { return @((Out) -split "`n" | Where-Object { $_ -like 'counts:*' })[0] }
function Count($name) { if ((Counts) -match "$name (\d+)") { return [int]$Matches[1] }; return -1 }
function WaitEv($pattern, $sec = 6) {
  for ($k = 0; $k -lt $sec * 4; $k++) { if ((Ev) -match $pattern) { return $true }; Start-Sleep -Milliseconds 250 }
  return $false
}
# The brightest grey pixel of a screen box (its channels within 6 of each other: not the glass
# or the wallpaper) as R G B.
function Brightest($x, $y, $w, $h) {
  $b = [Px]::Grab($x, $y, $w, $h); $best = 0; $top = -1
  foreach ($c in [Px]::Pixels($b)) {
    $ch = @((($c -shr 16) -band 255), (($c -shr 8) -band 255), ($c -band 255)); $m = $ch | Measure-Object -Maximum -Minimum
    if ($m.Maximum - $m.Minimum -le 6 -and $m.Maximum -gt $top) { $top = $m.Maximum; $best = $c }
  }
  $b.Dispose()
  return @((($best -shr 16) -band 255), (($best -shr 8) -band 255), ($best -band 255))
}
# The settings window and the plugin fence, cropped, for the pictures to compare.
function SettingsShot($name) { $r = Rect (WaitWin 'PecoFence.Settings' 2); ParkMouse; Start-Sleep -Milliseconds 500; (Crop $name $r.L $r.T ($r.R - $r.L) ($r.B - $r.T)).Dispose() }
function UiShot($name) { $w = DumpOf 'UI' 5; (Crop $name ($w.l - 8) ($w.t - 8) ($w.r - $w.l + 16) ($w.b - $w.t + 16)).Dispose() }
# A test index of three plugins: one that asks for nothing, one that goes online, one that starts a program.
function Publish() {
  $commit = 'c' * 40
  $entries = foreach ($p in @(@('pf-plain', 'PF Plain', @{}), @('pf-net', 'PF Net', @{ network = @('example.com') }), @('pf-sys', 'PF Sys', @{ system = @('ipconfig.exe') }))) {
    $dir = "$Feed\$commit\plugins\$($p[0])"; New-Item -ItemType Directory -Force $dir | Out-Null
    $m = [ordered]@{ id = $p[0]; version = '1.0.0'; api = 1; layer = 'light'; name = @{ en = $p[1] }; description = @{ en = 'A test plugin' }; author = 'tester'; minPecoFence = '0.4.0'; permissions = $p[2] }
    [IO.File]::WriteAllText("$dir\plugin.json", ($m | ConvertTo-Json -Depth 6))
    [IO.File]::WriteAllText("$dir\main.js", "pf.on('start', () => pf.show({ rows: [{ text: '$($p[0])' }] }));`r`n")
    [IO.File]::WriteAllText("$dir\README.md", "# $($p[0])`r`n")
    $m.files = @(foreach ($f in Get-ChildItem $dir -File) { [ordered]@{ name = $f.Name; size = $f.Length; sha256 = (Get-FileHash $f.FullName -Algorithm SHA256).Hash.ToLower() } })
    $m
  }
  [IO.File]::WriteAllText("$Feed\index.json", ([ordered]@{ format = 1; serial = 1; commit = $commit; plugins = @($entries) } | ConvertTo-Json -Depth 8))
}

Case 'setup'
CloseWindows
New-Item -ItemType Directory -Force $Feed | Out-Null
Publish
$script:FeedProc = PfFeed 8732 $Feed
$env:PECOFENCE_PLUGINS_URL = 'http://127.0.0.1:8732/index.json'
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
PfPlacePlugin 'C:\pf\kit\assets\plugin-ui\uidemo' 'uidemo'
PfStart -Lang $Lang -Script (DumpScript 400 30 3000)
$script:Config = Split-Path (PfJson status).configPath
Pf settings set tools.enabled true | Out-Null
Pf settings set theme light | Out-Null
ClearFences | Out-Null
$script:Ui = PfPluginFence 'UI' '80,40,380,620' 'uidemo'
Start-Sleep -Seconds 3
Check 'the test plugin shows its rows' ((Out) -match 'event: -' -and (Out) -match 'filler 30') ((Out) -split "`n" | Select-Object -First 3)

Case 'a row''s own menu'
ClickHit 2 -Right; $m = MenuWin 5; $texts = MenuTexts $m
Check 'a right-click on a row with a menu shows its entries in place of the fence''s' ($texts -match 'Pin it' -and $texts -match 'Remove it' -and $texts -notmatch 'Fence command') $texts
Shot 'row-menu'
if (-not (MenuPick 'Pin it' $m)) { CloseMenus }
Start-Sleep -Milliseconds 800
Check 'its entry sends command with via menu and the row''s id' (WaitEv 'command pin menu item1') (Ev)
ClickHit 3 -Right; $m = MenuWin 5; $texts = MenuTexts $m
Check 'a row without one shows the fence''s entries' ($texts -match 'Fence command' -and $texts -notmatch 'Pin it') $texts
MenuPick 'Fence command' $m | Out-Null; Start-Sleep -Milliseconds 800
Check 'and its entry sends command without a row' (WaitEv 'command fence-cmd menu -$') (Ev)
CloseMenus

Case 'a box of several lines'
$f = Hit 9; $one = Hit 3
Check 'the box is four lines tall' ($f -and ($f.b - $f.t) -gt 3.2 * ($one.b - $one.t)) "box $($f.t)..$($f.b), a row $($one.t)..$($one.b)"
ClickHit 9; $box = WaitWin 'PecoFence.Rename' 5
$t = if ($box -ne [IntPtr]::Zero) { BoxText $box } else { '' }
Check 'a click opens it with the text and its line break' ($t -eq "first line`r`nsecond line") "box=$box text='$t'"
UiShot 'text-box-open'
[G]::Press(0x0D); [G]::Type('third line'); Start-Sleep -Milliseconds 400
Check 'Enter starts a new line and keeps the box open' ((WaitWin 'PecoFence.Rename' 1) -ne [IntPtr]::Zero -and (Count 'change') -eq 0) (Counts)
Pf fence refresh $script:Ui | Out-Null; Start-Sleep -Milliseconds 800
$held = $null -ne (PfWaitLog 'restart or stop held while its input is open' 3)
Chord 0x11 0x0D; Start-Sleep -Milliseconds 1200
$runs = $null -ne (PfWaitLog 'the held restart runs now' 5)
Check 'a restart waits while the box is open and runs after' ($held -and $runs) "held=$held ran=$runs"
# The restarted plugin gets the text first (its counts start again).
Check 'Ctrl+Enter sends change with the line breaks, then submit' ((WaitEv 'submit t') -and (Out) -match 'got: first line\|second line\|third line' -and (Count 'change') -eq 1 -and (Count 'submit') -eq 1) "$(Ev) $(Counts)"
ClickHit 9; WaitWin 'PecoFence.Rename' 5 | Out-Null
[G]::Type('!'); [G]::Press(0x1B); Start-Sleep -Milliseconds 1000
Check 'Esc sends change, no submit' ((WaitEv 'change t 34') -and (Count 'submit') -eq 1 -and (Out) -match 'got: first line\|second line\|third line!') "$(Ev) $(Counts)"
ClickHit 9; WaitWin 'PecoFence.Rename' 5 | Out-Null
Set-Clipboard ('x' * 300); Chord 0x11 0x41; Chord 0x11 0x56; Start-Sleep -Milliseconds 300
DeskClick; Start-Sleep -Milliseconds 1000
Check 'a click elsewhere sends change; the box took at most max (200) characters' ((WaitEv 'change t 200') -and (WaitGoneWin 'PecoFence.Rename' 2)) (Ev)
UiShot 'text-box'

Case 'hover: a look of its own, enter and leave'
$look = Hit 6
Check 'a row with a look under the pointer reports the pointer' ($null -ne $look) "hit=$look"
$w = DumpOf 'UI' 5; $a = ContentPt 0 (Hit 5).b; $b = ContentPt 380 (Hit 7 1).t
$rest = OcrText $a[0] $a[1] ($w.r - $a[0]) ($b[1] - $a[1]) 'look-rest'
[G]::MoveTo($a[0] + 60, [int](($a[1] + $b[1]) / 2)); Start-Sleep -Milliseconds 700
$under = OcrText $a[0] $a[1] ($w.r - $a[0]) ($b[1] - $a[1]) 'look-hover'
UiShot 'hover-look'
Check 'under the pointer the row shows its hover text' ($rest -match 'plain look' -and $under -match 'LOOK UNDER POINTER') "rest='$rest' hover='$under'"
$p1 = HitPt 4; $p2 = HitPt 5
[G]::MoveTo($p1[0], $p1[1]); Start-Sleep -Milliseconds 700
Check 'onto a hover row: enter with its id' (WaitEv 'enter h1') (Ev)
[G]::MoveTo($p2[0], $p2[1]); Start-Sleep -Milliseconds 700
Check 'onto the next: leave the first, enter the second' ((WaitEv 'enter h2') -and (Count 'leave') -eq 1) "$(Ev) $(Counts)"
$e0 = Count 'enter'; $t0 = Get-Date
for ($i = 0; $i -lt 30; $i++) { [G]::MoveTo($p1[0], $p1[1]); Start-Sleep -Milliseconds 15; [G]::MoveTo($p2[0], $p2[1]); Start-Sleep -Milliseconds 15 }
[G]::MoveTo($p1[0], $p1[1]); $secs = ((Get-Date) - $t0).TotalSeconds; Start-Sleep -Milliseconds 800
$n = (Count 'enter') - $e0
Check 'sixty row changes reach the plugin at most 10 times a second, the last row told' ($n -ge 1 -and $n -le [math]::Ceiling($secs * 10) + 1 -and (WaitEv 'enter h1')) "enters=$n in $([math]::Round($secs, 2)) s, $(Ev)"
ParkMouse; Start-Sleep -Milliseconds 700
Check 'off the rows: leave' (WaitEv 'leave h1') (Ev)

Case 'a row of text cells with a menu and hover: "event"'
# Its cells take no pointer: the row has it, as cell 2 (after its two cells).
$p = HitPt 12 2
Check 'the row has the pointer where its cells are' ($null -ne $p) "hits $(DumpVal (DumpOf 'UI' 5) 'script_cells')"
if ($p) {
  [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 700
  Check 'onto it: enter with its id' (WaitEv 'enter crow') (Ev)
  [G]::RClick($p[0], $p[1]); $m = MenuWin 5; $texts = MenuTexts $m
  Check 'a right-click shows its own menu' ($texts -match 'Cell row entry' -and $texts -notmatch 'Fence command') $texts
  if (-not (MenuPick 'Cell row entry' $m)) { CloseMenus }
  Check 'its entry sends command with the row''s id' (WaitEv 'command cmenu menu crow') (Ev)
}
ParkMouse; Start-Sleep -Milliseconds 500
Check 'hover: "event" on a row without an id: the log says it is left out' ([bool](PfWaitLog 'the row has no id' 3)) ''

Case 'the wheel'
$scroll0 = DumpVal (DumpOf 'UI' 5) 'scroll'
$p = HitPt 7 1
Wheel $p[0] $p[1] 3; Start-Sleep -Milliseconds 900
$scroll1 = DumpVal (DumpOf 'UI' 5) 'scroll'
Check 'the wheel over a wheel cell goes to the plugin, the fence stays' ((Out) -match 'level 8' -and $scroll1 -eq $scroll0) "$(Ev), scroll $scroll0 -> $scroll1"
$p = HitPt 8
Wheel $p[0] $p[1] -2; Start-Sleep -Milliseconds 900
Check 'over a wheel row too, towards the user negative' ((Out) -match 'wheel row 6' -and (DumpVal (DumpOf 'UI' 5) 'scroll') -eq $scroll0) (Ev)
$g = Hit 11
$p = ContentPt 60 ($g.b + 40)
Wheel $p[0] $p[1] -3; Start-Sleep -Seconds 1
$scroll2 = DumpVal (DumpOf 'UI' 5) 'scroll'
Check 'elsewhere the wheel scrolls the fence' ([double]$scroll2 -gt [double]$scroll0) "scroll $scroll0 -> $scroll2"
Wheel $p[0] $p[1] 10; Start-Sleep -Seconds 1

Case 'gray text on a light-text fence'
Pf settings set theme dark | Out-Null; ParkMouse; Start-Sleep -Seconds 3
$g = Hit 11; $a = ContentPt ($g.l + 4) $g.t; $b = ContentPt ($g.l + 120) $g.b
$c = Brightest $a[0] $a[1] ($b[0] - $a[0]) ($b[1] - $a[1])
UiShot 'gray-dark'
Check 'gray is #C5C5C5 on light text' ([math]::Abs($c[0] - 0xC5) -le 10 -and [math]::Abs($c[2] - 0xC5) -le 10) "brightest=$($c -join ' ')"
Pf settings set theme light | Out-Null; Start-Sleep -Seconds 2

Case '设置 › 小工具: plain buttons, grey tags, a grey warning for a plugin that asks for nothing'
$ok = PfSettings; PageClick 'nav a[data-page=tools]'; Start-Sleep -Seconds 4
$buttons = Js "[...document.querySelectorAll('#pluginList .pl-row')].filter(r => r.textContent.startsWith('PF ')).map(r => r.querySelector('button').textContent + ':' + r.querySelector('button').className).join(' / ')"
Check 'the 安装 buttons of the list are plain' ($ok -and ($buttons -split ' / ').Count -eq 3 -and $buttons -notmatch 'accent') $buttons
$tags = Js "[...document.querySelectorAll('#pluginList .pl-tags span')].map(s => s.textContent + ':' + (s.className || 'grey')).join(' / ')"
$yellow = Js "[...document.querySelectorAll('#pluginList .pl-tags span.more')].map(s => s.closest('.pl-row').textContent.slice(0, 6)).join(',')"
Check 'tags are grey, only 启动程序 yellow' ($tags -match ':grey' -and $yellow -eq 'PF Sys') "$tags | yellow in: $yellow"
SettingsShot 'settings-list'
$i = [int](Js "[...document.querySelectorAll('#pluginList .pl-row')].findIndex(r => r.textContent.startsWith('PF Plain'))")
PageClick '#pluginList .pl-row button' $i; Start-Sleep -Milliseconds 800
$plain = Js "document.getElementById('plDoesBox').className + ' ok:' + document.getElementById('plOk').className"
SettingsShot 'settings-warning-plain'
PageClick '#plCancel'; Start-Sleep -Milliseconds 500
$i = [int](Js "[...document.querySelectorAll('#pluginList .pl-row')].findIndex(r => r.textContent.startsWith('PF Net'))")
PageClick '#pluginList .pl-row button' $i; Start-Sleep -Milliseconds 800
$net = Js "document.getElementById('plDoesBox').className"
SettingsShot 'settings-warning-net'
PageClick '#plCancel'; Start-Sleep -Milliseconds 500
Check 'the warning of a plugin that asks for nothing is grey, of one that goes online yellow; 安装并添加 stays blue' ($plain -match 'plain' -and $plain -match 'ok:accent' -and $net -notmatch 'plain') "plain: $plain, net: $net"

Case 'the end'
PfQuit
Stop-Process -Id $script:FeedProc.Id -Force -ErrorAction SilentlyContinue
Remove-Item Env:\PECOFENCE_PLUGINS_URL, Env:\PECOFENCE_WEBVIEW_ARGS -ErrorAction SilentlyContinue
