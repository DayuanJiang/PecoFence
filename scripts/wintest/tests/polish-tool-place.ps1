# Fence polish round 2026-11-12: a tool made on Settings > Tools is placed clear of the Settings
# window. Before, the note went to the middle of the screen, fully under the window, and the
# clock and the weather to the top left, partly under it. The window is tried where it opens by
# itself, at the left edge of the screen and at the right edge. The tools are plugins now:
# stand-ins of notes, clock, weather, github and pc-status are installed (no plugin index is
# reached), and each row's 添加到桌面 makes its fence. A note from the tray menu (a notes
# plugin fence) and one from the CLI keep their place. The page of a release build is clicked
# by its labels (OCR). The steps expect the box's 1600x900 screen: the window then leaves 320 px
# on each side.
trap { PfTrap $_; continue }
function ToolIds() { return @(@(PfJson fence list) | Where-Object { $_ -and $_.tool } | ForEach-Object { $_.id }) }
# The tool fence that was not among $before, waited for up to $sec seconds ($null when none came).
function NewTool($before, $sec = 12) {
  for ($i = 0; $i -lt $sec * 2; $i++) {
    $id = @(ToolIds | Where-Object { $before -notcontains $_ }) | Select-Object -First 1
    if ($id) { Start-Sleep -Milliseconds 400; return (FenceGet $id) }
    Start-Sleep -Milliseconds 500
  }
  Log 'NewTool: no new tool fence'
  return $null
}
function WinText($h) { $r = Rect $h; return "$($r.L),$($r.T),$($r.R),$($r.B)" }
function ToolText($f) { if (-not $f) { return 'none' }; $w = $f.windowRect; return "$($w.x),$($w.y),$($w.x + $w.w),$($w.y + $w.h)" }
# How many pixels of fence $f's window the window $h covers (-1 without a fence).
function Under($f, $h) {
  if (-not $f) { return -1 }
  $w = $f.windowRect; $r = Rect $h
  $dx = [math]::Min($w.x + $w.w, $r.R) - [math]::Max($w.x, $r.L)
  $dy = [math]::Min($w.y + $w.h, $r.B) - [math]::Max($w.y, $r.T)
  if ($dx -le 0 -or $dy -le 0) { return 0 }
  return $dx * $dy
}
function CheckClear($what, $f) {
  Check "$what is not under the Settings window" ((Under $f $s) -eq 0) "$what=$(ToolText $f) settings=$(WinText $s) covered=$(Under $f $s) px"
}
# The centre of the button $label in the row titled $title of the Tools page.
# The plugin rows go below the page's fold: the page is scrolled to its top first, and down when
# the row is not in sight.
function RowButton($title, $label) {
  $sr = Rect $s
  Wheel ($sr.L + 600) ($sr.T + 400) 10; Start-Sleep -Milliseconds 600
  $t = OcrFind $title ($sr.L + 260) $sr.T 520 ($sr.B - $sr.T)
  if (-not $t) {
    Wheel ($sr.L + 600) ($sr.T + 400) -6; Start-Sleep -Milliseconds 600
    $t = OcrFind $title ($sr.L + 260) $sr.T 520 ($sr.B - $sr.T)
  }
  if (-not $t) { return $null }
  return (OcrFind $label ($sr.R - 280) ($t[1] - 24) 280 90)
}
# Clicks that button and returns the tool fence it made.
function AddTool($title, $label = '^Add to desktop$') {
  $before = ToolIds
  $b = RowButton $title $label
  if (-not $b) { return $null }
  [G]::Click($b[0], $b[1])
  return (NewTool $before)
}
# The notes plugin's row: its fence.
function AddNote($text) { return (AddTool '^Notes\W+Dayuan') }
# 新建便签 of the tray menu: a fence of the notes plugin. The tray's overflow window is opened
# first and left to settle: the menu's place follows the icon while it slides in.
function TrayNote() {
  $before = ToolIds
  TrayIconRect | Out-Null; Start-Sleep -Milliseconds 1500
  TrayPick '^New note' | Out-Null
  return (NewTool $before)
}
# The Settings window moved to x = $left (its size and top kept), the Tools page shown again.
function MoveSettings($left) {
  $r = Rect $s
  [L]::MoveWindow($s, $left, $r.T, $r.R - $r.L, $r.B - $r.T, $true) | Out-Null
  Start-Sleep -Seconds 1
  Log "settings window moved: $(WinText $s)"
}

Case 'the first-run fences leave no room beside the window'
CloseWindows
Set-Culture en-US
$env:PECOFENCE_PLUGINS_URL = 'http://127.0.0.1:9/index.json'
foreach ($id in 'notes', 'clock', 'weather', 'github', 'pc-status') { PfPlacePlugin "C:\pf\kit\assets\plugin-migrate\$id" $id }
PfStart -Lang en
Start-Sleep -Seconds 4
$screenW = [G]::GetSystemMetrics(0)
Pf settings open-ui | Out-Null
$s = WaitWin 'PecoFence.Settings' 30
Start-Sleep -Seconds 6
$sr = Rect $s
Log "settings window as it opens: $(WinText $s) on a ${screenW}x$([G]::GetSystemMetrics(1)) screen"
foreach ($f in @(PfJson fence list)) { if ($f -and $f.id) { Log "fence '$($f.title)' [$($f.kind)] $(ToolText (FenceGet $f.id))" } }
$nav = OcrFind '^Tools$' $sr.L $sr.T 260 ($sr.B - $sr.T)
[G]::Click($nav[0], $nav[1]); Start-Sleep -Seconds 2
$note = AddNote 'first run'
Shot 'place-1-first-run'
$half = if ($note) { $note.windowRect.w * $note.windowRect.h / 2 } else { 0 }
Check 'with fences on both sides of the window, the window covers less than half of the new note' ($note -and (Under $note $s) -lt $half) "note=$(ToolText $note) settings=$(WinText $s) covered=$(Under $note $s) px of $($half * 2)"

Case 'the window where it opens by itself'
ClearFences | Out-Null
$note = AddNote 'as it opens'
$clock = AddTool '^Clock\W+Dayuan'
$weather = AddTool '^Weather\W+Dayuan'
Start-Sleep -Seconds 1
Shot 'place-2-as-it-opens'
CheckClear 'the note' $note
CheckClear 'the clock' $clock
CheckClear 'the weather fence' $weather
Check 'the note is beside the window, level with its top' ($note -and ($note.windowRect.x -ge $sr.R -or $note.windowRect.x + $note.windowRect.w -le $sr.L) -and $note.windowRect.y -eq $sr.T) "note=$(ToolText $note) settings=$(WinText $s)"

Case 'the window at the left edge of the screen'
ClearFences | Out-Null
MoveSettings 0
$sr = Rect $s
$git = AddTool '^GitHub activity\W+Dayuan'
$status = AddTool '^PC status\W+Dayuan'
# 用自己的脚本: its file dialog names the script.
Set-Content C:\pf\run\place.ps1 -Value 'Write-Output "placed"' -Encoding UTF8
$before = ToolIds
$b = RowButton '^Your own script$' 'Choose script'
[G]::Click($b[0], $b[1])
$dlg = [IntPtr]::Zero
for ($i = 0; $i -lt 40 -and $dlg -eq [IntPtr]::Zero; $i++) { Start-Sleep -Milliseconds 250; $dlg = @(ProcTops '#32770' 'pecofence') | Select-Object -First 1; if (-not $dlg) { $dlg = [IntPtr]::Zero } }
Start-Sleep -Milliseconds 800
[QS]::ViaFileName($dlg, 'C:\pf\run\place.ps1') | Out-Null
$own = NewTool $before
Start-Sleep -Seconds 1
Shot 'place-3-left-edge'
CheckClear 'the GitHub fence' $git
CheckClear 'the PC status fence' $status
CheckClear 'the fence of a script chosen in the file dialog' $own
Check 'the GitHub fence is on the right of the window, the side with more room' ($git -and $git.windowRect.x -ge $sr.R) "github=$(ToolText $git) settings=$(WinText $s)"

Case 'the window at the right edge of the screen'
ClearFences | Out-Null
MoveSettings ($screenW - ($sr.R - $sr.L))
$sr = Rect $s
$note = AddNote 'right edge'
$clock = AddTool '^Clock\W+Dayuan'
$weather = AddTool '^Weather\W+Dayuan'
Start-Sleep -Seconds 1
Shot 'place-4-right-edge'
CheckClear 'the note' $note
CheckClear 'the clock' $clock
CheckClear 'the weather fence' $weather
Check 'the note is on the left of the window, the side with more room' ($note -and $note.windowRect.x + $note.windowRect.w -le $sr.L) "note=$(ToolText $note) settings=$(WinText $s)"

Case 'the tray menu and the CLI keep their placement'
# The window still covers the right half of the screen, where a note from the tray menu goes
# (above the tray icon) and where the CLI puts one (the middle of the screen).
ClearFences | Out-Null
$tray = TrayNote
$trayCovered = Under $tray $s
$wa = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea
Pf fence create --tool note --title CliNote | Out-Null; Start-Sleep -Milliseconds 800
$cli = PfFence 'CliNote'
Check 'a note from the CLI lands at the middle of the screen as before, under the window' ($cli -and $cli.windowRect.x -eq $wa.X + [int]($wa.Width / 2) - 140 -and $cli.windowRect.y -eq $wa.Y + [int]($wa.Height / 2) - 120 -and (Under $cli $s) -gt 0) "note=$(ToolText $cli) work area=$($wa.Width)x$($wa.Height) settings=$(WinText $s) covered=$(Under $cli $s) px"
Shot 'place-5-tray-cli'
$placed = @(PfLogAll | Where-Object { $_ -match 'new fence from the settings page' })
Log "placement log lines: $($placed.Count)"
foreach ($l in $placed) { Log "  $($l.Substring(0, [math]::Min(460, $l.Length)))" }
$open = WinText $s
[L]::PostMessageW($s, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
Start-Sleep -Seconds 2
ClearFences | Out-Null
$alone = TrayNote
Check 'a note from the tray menu lands where it does without the Settings window, under the window' ($tray -and $alone -and (ToolText $tray) -eq (ToolText $alone) -and $trayCovered -gt 0) "pick='$LastTrayMenu' with the window at ${open}: $(ToolText $tray), covered=$trayCovered px; without it: $(ToolText $alone)"
