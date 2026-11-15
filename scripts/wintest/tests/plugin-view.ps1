# 插件的轻量层, the view: every part of a light plugin's view and what the user does in it, with
# the test plugins of assets/plugin-view (viewdemo: five pages of parts; viewnote: a note). A
# click, a double-click, a check box, a to-do, a slider drag, a button that hangs the plugin and
# 重新启动, a one-line box typed in English and in Chinese and left with Esc, a note edited and a
# to-do ticked without the editor (the plugin gets the text), a restart held while the note is
# edited, the wheel under pinned rows. Then screenshots of every page in the light and the dark
# theme, beside the same rows in a script fence where script fences have them. Run it again at
# 150 % (box.sh's WINTEST_SCALE) for those screenshots: fences are sized by the scale.
#   run.sh --bin target/wintest-bin-debug scripts/wintest/tests/plugin-view.ps1
$Assets = 'C:\pf\kit\assets\plugin-view'

# A script fence at `rect` (x,y,w,h px) running `path`; its id.
function NewScriptFence([string]$title, [string]$rect, [string]$path) {
  $f = PfJson fence create --title $title --script $path --rect $rect
  return "$(if ($f.fence) { $f.fence.id } else { $f.id })"
}
function CellRects($w, $field = 'script_cells') {
  $v = DumpVal $w $field
  if (-not $v -or $v -eq '-') { return @() }
  return @($v -split ';' | ForEach-Object {
    $pair = $_ -split ':'; $n = $pair[1] -split ','
    [pscustomobject]@{ i = [int]$pair[0]; l = [double]$n[0]; t = [double]$n[1]; r = [double]$n[2]; b = [double]$n[3] }
  })
}
# Row `row`, cell `cell` of the fence titled `title` as drawn now (content DIPs).
function Hit($title, $row, $cell = 0) {
  $w = DumpOf $title 5
  return @(CellRects $w | Where-Object { $_.i -eq $row * 100 + $cell }) | Select-Object -First 1
}
# The display scale of a dumped window: its title row is 36 DIPs tall.
function Dpi($w) { return [double](DumpVal $w 'content_y') / 36.0 }
function ContentPt($w, $x, $y) {
  $sc = Dpi $w
  return @([int]($w.l + [double](DumpVal $w 'content_x') + $x * $sc), [int]($w.t + [double](DumpVal $w 'content_y') + $y * $sc))
}
# The screen point of a spot in row `row`, cell `cell` (fx, fy: 0 to 1 across it).
function HitPt($title, $row, $cell = 0, $fx = 0.5, $fy = 0.5) {
  $w = DumpOf $title 5
  $r = @(CellRects $w | Where-Object { $_.i -eq $row * 100 + $cell }) | Select-Object -First 1
  if (-not $r) { Log "no hit $row/$cell in $(DumpVal $w 'script_cells')"; return $null }
  return ContentPt $w ($r.l + ($r.r - $r.l) * $fx) ($r.t + ($r.b - $r.t) * $fy)
}
function ClickHit($title, $row, $cell = 0) {
  $p = HitPt $title $row $cell
  if (-not $p) { return $false }
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700; ParkMouse
  return $true
}
function WaitOut([string]$fid, $pattern, $sec = 8) {
  for ($k = 0; $k -lt $sec * 4; $k++) { if ((FenceOut $fid) -match $pattern) { return $true }; Start-Sleep -Milliseconds 250 }
  return $false
}
function Ev([string]$fid) { return @((FenceOut $fid) -split "`n" | Where-Object { $_ -like 'event:*' })[0] }
function Page([string]$p) { ClickHit 'View' 0 (@('rows', 'cells', 'draw', 'input', 'list').IndexOf($p)) | Out-Null; WaitOut $script:View "event: click page:$p" | Out-Null; Start-Sleep -Milliseconds 600 }
# The mean colour around content DIPs (x, y) of the window of `title`.
function RgbAt($title, $x, $y) {
  $w = DumpOf $title 5; $p = ContentPt $w $x $y
  $b = [Px]::Grab($p[0] - 2, $p[1] - 2, 5, 5); $c = MeanRgb $b; $b.Dispose()
  return $c
}
# A crop of the fences `titles` side by side (their union, a margin around), for the report.
function Pair([string]$name, [string[]]$titles) {
  $ws = @($titles | ForEach-Object { DumpOf $_ 5 } | Where-Object { $_ })
  if (-not $ws) { return }
  $l = ($ws | Measure-Object l -Minimum).Minimum - 12; $t = ($ws | Measure-Object t -Minimum).Minimum - 12
  $r = ($ws | Measure-Object r -Maximum).Maximum + 12; $b = ($ws | Measure-Object b -Maximum).Maximum + 12
  ParkMouse
  Start-Sleep -Milliseconds 500
  (Crop $name $l $t ($r - $l) ($b - $t)).Dispose()
}

Case 'setup'
CloseWindows
# Microsoft Pinyin for pf, for the one-line box (taken away again at the end).
$langs = Get-WinUserLanguageList
$script:AddedZh = -not ($langs | Where-Object { $_.LanguageTag -like 'zh-Hans*' })
if ($AddedZh) { $langs.Add('zh-Hans-CN'); Set-WinUserLanguageList $langs -Force }
# The test plugins go in before the start: PecoFence reads the installed plugins once as it starts.
$script:Config = "$PfApp\config"
PfPlacePlugin "$Assets\viewdemo" 'viewdemo'; PfPlacePlugin "$Assets\viewnote" 'viewnote'
PfStart -Lang en -Script (DumpScript 400 30 3000)
Check 'the portable configuration is where the plugins went' ((Split-Path (PfJson status).configPath) -eq $script:Config) (PfJson status).configPath
Pf settings set tools.enabled true | Out-Null
Pf settings set theme light | Out-Null
ClearFences | Out-Null
# The picture of the data folder: red left, blue right.
Add-Type -AssemblyName System.Drawing
$data = "$script:Config\plugins-data\viewdemo"; New-Item -ItemType Directory -Force $data | Out-Null
$bmp = New-Object Drawing.Bitmap 200, 80; $g = [Drawing.Graphics]::FromImage($bmp)
$g.FillRectangle([Drawing.Brushes]::Red, 0, 0, 100, 80); $g.FillRectangle([Drawing.Brushes]::Blue, 100, 0, 100, 80); $g.Dispose()
$bmp.Save("$data\pic.png"); $bmp.Dispose()
Copy-Item "$data\pic.png" "$PfRun\pic.png"
$script:View = PfPluginFence 'View' '60,20,380,720' 'viewdemo'
$ok = WaitOut $script:View 'event: -' 15
# Fences sized for the scale: 380 x 720 px at 100 %.
$k = Dpi (DumpOf 'View' 8); if (-not $k) { $k = 1.0 }
$fw = [int](380 * $k); $fh = [int][math]::Min(720 * $k, 860)
if ($k -ne 1.0) { Pf fence move $script:View --rect "60,20,$fw,$fh" | Out-Null; Start-Sleep -Seconds 2 }
Check 'the plugin fence shows the plugin''s view' $ok "scale=$k $((FenceOut $script:View) -split "`n" | Select-Object -First 4)"

Case 'rows: a click, a double-click, a to-do, a blur'
$out = FenceOut $script:View
Check 'every text row is there' ($out -match 'small text' -and $out -match 'huge' -and $out -match 'bold red' -and $out -match 'Disk C') (($out -split "`n") -join ' / ')
ClickHit 'View' 11 | Out-Null
Check 'a click on a row with an id sends click' (WaitOut $script:View 'event: click row1') (Ev $script:View)
$p = HitPt 'View' 11
DoubleClick $p[0] $p[1]; Start-Sleep -Milliseconds 800; ParkMouse
Check 'a double-click sends dblclick' (WaitOut $script:View 'event: dblclick row1') (Ev $script:View)
ClickHit 'View' 15 | Out-Null
Check 'a click on a to-do sends change with its new state' (WaitOut $script:View 'event: change todo1 true') (Ev $script:View)
$w = DumpOf 'View' 5
$r = Hit 'View' 13
$p0 = ContentPt $w $r.l $r.t; $p1 = ContentPt $w $r.r $r.b
$blurred = [Px]::Grab($p0[0], $p0[1], $p1[0] - $p0[0], $p1[1] - $p0[1])
[G]::MoveTo(($p0[0] + $p1[0]) / 2, ($p0[1] + $p1[1]) / 2); Start-Sleep -Milliseconds 900
$sharp = [Px]::Grab($p0[0], $p0[1], $p1[0] - $p0[0], $p1[1] - $p0[1])
$diff = [Px]::Diff($blurred, $sharp, 0, $blurred.Width - 1, 0, $blurred.Height - 1)
Shot 'rows-blur-revealed'
ParkMouse; Start-Sleep -Milliseconds 900
$again = [Px]::Grab($p0[0], $p0[1], $p1[0] - $p0[0], $p1[1] - $p0[1])
$back = [Px]::Diff($blurred, $again, 0, $blurred.Width - 1, 0, $blurred.Height - 1)
Check 'a blurred row is sharp under the pointer and blurred again after' ($diff -gt 3 -and $back -lt 1.5) "diff=$([math]::Round($diff, 2)) back=$([math]::Round($back, 2))"
$blurred.Dispose(); $sharp.Dispose(); $again.Dispose()
Shot 'rows'
# A narrow fence: the fit row takes its short form.
Pf fence move $script:View --rect "60,20,$([int](128 * $k)),$fh" | Out-Null; Start-Sleep -Seconds 2
Pair "rows-narrow-$([int](100 * $k))" @('View')
Pf fence move $script:View --rect "60,20,$fw,$fh" | Out-Null; Start-Sleep -Seconds 2

Case 'cells: buttons, a check box, a slider, under'
Page 'cells'
ClickHit 'View' 2 0 | Out-Null
Check 'a button sends click' (WaitOut $script:View 'event: click b1') (Ev $script:View)
ClickHit 'View' 2 1 | Out-Null
Check 'an icon button sends click' (WaitOut $script:View 'event: click b2') (Ev $script:View)
ClickHit 'View' 2 3 | Out-Null
Check 'a check box sends change false' (WaitOut $script:View 'event: change c1 false') (Ev $script:View)
ClickHit 'View' 2 3 | Out-Null
Check 'and true again' (WaitOut $script:View 'event: change c1 true') (Ev $script:View)
$p = HitPt 'View' 2 0
DoubleClick $p[0] $p[1]; Start-Sleep -Milliseconds 800; ParkMouse
Check 'a double-click on a button sends dblclick after its clicks' (WaitOut $script:View 'event: dblclick b1') (Ev $script:View)
$a = HitPt 'View' 3 1 0.2; $b = HitPt 'View' 3 1 0.9
[G]::MoveTo($a[0], $a[1]); Start-Sleep -Milliseconds 200; MDown; Start-Sleep -Milliseconds 300
$mid = Ev $script:View
Glide $a[0] $a[1] $b[0] $b[1] 12 30; Start-Sleep -Milliseconds 300; MUp; Start-Sleep -Milliseconds 800; ParkMouse
$v = if ((Ev $script:View) -match 'change s1 ([\d.]+)') { [double]$Matches[1] } else { -1 }
Check 'a slider sends change on the press and where it was let go' ($mid -match 'change s1 0\.[1-3]' -and $v -gt 0.8) "press: $mid, release: $(Ev $script:View)"
$top = Hit 'View' 4 1; $under = Hit 'View' 4 2
Check 'under puts a cell under the one before it' ($under.t -ge $top.b - 1 -and [math]::Abs($under.l - $top.l) -lt 2) "top=$($top.l),$($top.b) under=$($under.l),$($under.t)"
Shot 'cells'

Case 'a plugin that hangs: the reason and 重新启动'
ClickHit 'View' 5 2 | Out-Null
Check 'the fence says the plugin stopped' (WaitOut $script:View 'The plugin stopped: one run took longer than 50 ms' 5) ((FenceOut $script:View) -split "`n" | Select-Object -First 2)
Shot 'stopped'
ClickHit 'View' 1 0 | Out-Null
Check '重新启动 starts it again' (WaitOut $script:View 'event: -' 8) ((FenceOut $script:View) -split "`n" | Select-Object -First 2)

Case 'draw: pictures, a chart, a canvas'
Page 'draw'
$img = Hit 'View' 2
$c = if ($img) { RgbAt 'View' ($img.l + ($img.r - $img.l) * 0.25) (($img.t + $img.b) / 2) } else { @(0, 0, 0) }
Check 'the picture row shows the data folder''s picture' ($c[0] -gt 180 -and $c[2] -lt 90) "rgb=$($c -join ' ')"
$cv = Hit 'View' 5
$c = if ($cv) { RgbAt 'View' ($cv.l + 40) ($cv.t + 30) } else { @(0, 0, 0) }
Check 'the canvas fills its rect' ($c[2] -gt 180 -and $c[0] -lt 90) "rgb=$($c -join ' ') at $($cv.l + 40),$($cv.t + 30)"
Shot 'draw'

Case 'input: a one-line box, English and Chinese'
Page 'input'
ClickHit 'View' 2 | Out-Null
$box = WaitWin 'PecoFence.Rename' 5
Check 'a click opens the one-line box' ($box -ne [IntPtr]::Zero) "box=$box"
Shot 'input-open'
[G]::Type('hello'); Start-Sleep -Milliseconds 300; [G]::Press(0x0D); Start-Sleep -Milliseconds 800
Check 'Enter sends text and the row shows it' ((WaitOut $script:View 'event: text q hello') -and (FenceOut $script:View) -match 'typed: hello') (Ev $script:View)
ClickHit 'View' 2 | Out-Null
WaitWin 'PecoFence.Rename' 5 | Out-Null
[L]::Ime('00000804'); Start-Sleep -Milliseconds 1500
if ([L]::Layout() -notlike '*0804') { Chord 0x5B 0x20; Start-Sleep -Milliseconds 1500 }
Log "layout in the box: $([L]::Layout())"
foreach ($ch in 'NIHAO'.ToCharArray()) { [G]::Press([uint16][int]$ch); Start-Sleep -Milliseconds 150 }
Start-Sleep -Milliseconds 1200
Shot 'input-composing'
[G]::Press(0x20); Start-Sleep -Milliseconds 800
[L]::Ime('00000409'); Start-Sleep -Milliseconds 600
[G]::Press(0x0D); Start-Sleep -Milliseconds 800
$typed = @((FenceOut $script:View) -split "`n" | Where-Object { $_ -like 'typed:*' })[0]
Check 'Chinese typed with Microsoft Pinyin reaches the plugin' ($typed -match '[一-鿿]') "$typed layout=$([L]::Layout())"
ClickHit 'View' 2 | Out-Null
WaitWin 'PecoFence.Rename' 5 | Out-Null
[G]::Type('xyz'); Start-Sleep -Milliseconds 300; [G]::Press(0x1B); Start-Sleep -Milliseconds 800
Check 'Esc closes the box and sends nothing' ((FenceOut $script:View) -notmatch 'xyz' -and (WaitGoneWin 'PecoFence.Rename' 3)) (Ev $script:View)
$p = HitPt 'View' 2
[G]::Click($p[0], $p[1]); WaitWin 'PecoFence.Rename' 5 | Out-Null
[G]::Type('abc'); Start-Sleep -Milliseconds 300
DeskClick; Start-Sleep -Milliseconds 900
Check 'a click elsewhere sends the text too' (WaitOut $script:View 'event: text q abc') (Ev $script:View)
Shot 'input'

Case 'list: the wheel under pinned rows'
Page 'list'
$w = DumpOf 'View' 5
$before = [double](DumpVal $w 'scroll'); $nav = Hit 'View' 0 0
$p = HitPt 'View' 5
Wheel $p[0] $p[1] -3; Start-Sleep -Seconds 1; ParkMouse
$w2 = DumpOf 'View' 5
$after = [double](DumpVal $w2 'scroll')
Check 'the wheel scrolls the rows under the pinned ones' ($after -gt $before) "scroll $before -> $after"
Check 'the rows above the pane stay' ((Hit 'View' 0 0).t -eq $nav.t) "nav top $($nav.t) -> $((Hit 'View' 0 0).t)"
Shot 'list'

Case 'note: edited in place, a to-do ticked without the editor'
$script:Note = PfPluginFence 'Note' "$([int](60 + $fw + 40)),20,$fw,$([int](300 * $k))" 'viewnote'
$ok = $null -ne (PfWaitLog 'its view holds a note now' 10)
Start-Sleep -Seconds 1
Check 'a note view makes the fence a note' ($ok -and (FenceGet $script:Note).noteText -eq '') "note=$((FenceGet $script:Note).noteText)"
$wn = DumpOf 'Note' 5
$p = ContentPt $wn 60 20
[G]::Click($p[0], $p[1]); $ed = WaitWin 'PecoFence.NoteEdit' 5 -Hidden
Check 'a click opens the note editor' ($ed -ne [IntPtr]::Zero) ''
[G]::Type('first line'); [G]::Press(0x0D); [G]::Type('[ ] milk'); Start-Sleep -Milliseconds 500
Pf fence refresh $script:Note | Out-Null; Start-Sleep -Milliseconds 800
$held = $null -ne (PfWaitLog 'restart or stop held while its input is open' 3)
Shot 'note-editing'
[G]::Press(0x1B); Start-Sleep -Milliseconds 1200
$runs = $null -ne (PfWaitLog 'the held restart runs now' 5)
Check 'a restart waits while the note is edited, and runs after' ($held -and $runs) "held=$held ran=$runs"
$saved = (FenceGet $script:Note).noteText
Check 'the note is saved as a note fence''s' ($saved -match '^first line\n\[ \] milk$') "note=$saved"
Check 'the plugin gets the text' (WaitOut $script:Note 'got: note first line\|\[ \] milk' 8) (FenceOut $script:Note)
$wn = DumpOf 'Note' 5
$box = @(CellRects $wn 'note_boxes')[0]
if ($box) { $p = ContentPt $wn (($box.l + $box.r) / 2) (($box.t + $box.b) / 2); [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900; ParkMouse }
Check 'a to-do ticks without the editor and the plugin gets it' ((WaitGoneWin 'PecoFence.NoteEdit' 1) -and (FenceGet $script:Note).noteText -match '\n\[x\] milk' -and (WaitOut $script:Note 'got: note first line\|\[x\] milk' 8)) "note=$((FenceGet $script:Note).noteText) out=$(FenceOut $script:Note)"
$cfg = Get-Content "$script:Config\config.json" -Raw -Encoding UTF8 | ConvertFrom-Json
Check 'the text is in the config''s notes' ("$($cfg.notes.$script:Note)" -match 'milk') ''
Shot 'note'

Case 'screenshots: each page beside a script fence, light and dark'
$rowsPs = "$PfRun\rows.ps1"; $cellsPs = "$PfRun\cells.ps1"; $drawPs = "$PfRun\draw.ps1"
Set-Content $rowsPs -Encoding UTF8 -Value @(
  'Write-Output ":: rows | action=a :: cells | action=a :: draw | action=a :: input | action=a :: list | action=a"',
  'Write-Output "event: - | color=gray"', 'Write-Output "normal text"',
  'Write-Output "#1e90ff with an emoji | color=#1e90ff icon=☀️"', 'Write-Output "click me | action=a tooltip=tip"',
  'Write-Output "---"', 'Write-Output "[ ] a to-do | action=a"', 'Write-Output "[x] a done to-do | action=a"')
Set-Content $cellsPs -Encoding UTF8 -Value @(
  'Write-Output ":: rows | action=a :: cells | action=a checked=true :: draw | action=a :: input | action=a :: list | action=a"',
  'Write-Output "event: - | color=gray"',
  ('Write-Output ":: Button | action=b :: ' + [char]0xE768 + ' | action=p :: On | action=o checked=true :: Check | checkbox=true action=c"'),
  'Write-Output ":: Vol :: | slider=0.4 action=s :: 40% | width=40 align=right"',
  'Write-Output ":: A long text that fills the room and ends with an ellipsis | fill=true action=w :: Top | action=t :: Under | under=true action=u"',
  ('Write-Output ":: | image=""' + "$PfRun\pic.png" + '"" height=40 width=80 action=i :: secret cell :: Hang | action=h"'))
Set-Content $drawPs -Encoding UTF8 -Value @(
  'Write-Output ":: rows | action=a :: cells | action=a :: draw | action=a checked=true :: input | action=a :: list | action=a"',
  'Write-Output "event: - | color=gray"', ('Write-Output " | image=""' + "$PfRun\pic.png" + '"" height=60"'))
$script:Script = NewScriptFence 'Script' "$([int](60 + $fw + 40)),20,$fw,$fh" $rowsPs
Pf fence delete $script:Note | Out-Null
$sk = "$([int](100 * $k))"
foreach ($theme in 'light', 'dark') {
  Pf settings set theme $theme | Out-Null; Start-Sleep -Seconds 2
  foreach ($pg in 'rows', 'cells', 'draw', 'input', 'list') {
    Page $pg
    $ps = @{ rows = $rowsPs; cells = $cellsPs; draw = $drawPs }[$pg]
    if ($ps) { Pf fence set $script:Script scriptPath $ps | Out-Null; Start-Sleep -Seconds 3; Pair "pair-$pg-$theme-$sk" @('View', 'Script') }
    else { Pair "page-$pg-$theme-$sk" @('View') }
  }
  $script:Note = PfPluginFence 'Note' "$([int](60 + $fw + 40)),20,$fw,$([int](300 * $k))" 'viewnote'
  Pf fence delete $script:Script | Out-Null; Start-Sleep -Seconds 2
  Pair "page-note-$theme-$sk" @('Note')
  Pf fence delete $script:Note | Out-Null
  $script:Script = NewScriptFence 'Script' "$([int](60 + $fw + 40)),20,$fw,$fh" $rowsPs
}
Check 'screenshots taken' ((Get-ChildItem "$PfShots\pair-*.png").Count -ge 6) "$((Get-ChildItem "$PfShots\*-$sk.png").Name -join ' ')"

Case 'the end'
PfQuit
if ($AddedZh) { $langs = Get-WinUserLanguageList; $zh = $langs | Where-Object { $_.LanguageTag -like 'zh-Hans*' }; foreach ($z in @($zh)) { [void]$langs.Remove($z) }; Set-WinUserLanguageList $langs -Force }
