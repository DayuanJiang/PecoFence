# linux.do request (topic 2998063, post 141): the text of the tools (小工具) was too small to
# read. The clock's time and date now follow the fence size; notes and script fences have
# 文字大小 (small / normal / large / extra large) in 栅栏选项 and `fence set textSize`.
# Every way of making each tool (tray menu, fence menu, CLI) x every size and action: check
# boxes and clickable rows (their hit areas), the note editor's font, a resize by mouse and by
# CLI, a restart. Parts (run.sh --var Part=...): N notes, S script fences, C the clock, P the
# settings page (debug build), B what the old build does, A the before/after recordings.
Case 'setup'
CloseWindows
Set-Culture en-US
GreyWallpaper
if (-not $Part) { $Part = 'N' }
if ($Part -eq 'P' -and $PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
. C:\pf\kit\rec.ps1
Add-Type -ReferencedAssemblies System.Drawing @'
using System; using System.Text; using System.Drawing; using System.Drawing.Imaging;
using System.Runtime.InteropServices;
public static class TT {
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindowExW(IntPtr p, IntPtr a, string c, string t);
  [DllImport("user32.dll")] static extern IntPtr SendMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("gdi32.dll")] static extern int GetObjectW(IntPtr h, int c, byte[] buf);
  [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
  // lfHeight of the font an EDIT control draws with (negative: the character height in px).
  public static int FontHeight(IntPtr edit) {
    IntPtr f = SendMessageW(edit, 0x31, IntPtr.Zero, IntPtr.Zero);
    if (f == IntPtr.Zero) return 0;
    var b = new byte[92];
    if (GetObjectW(f, 92, b) == 0) return 0;
    return BitConverter.ToInt32(b, 0);
  }
  static int Lum(int c) { return (((c >> 16) & 255) * 299 + ((c >> 8) & 255) * 587 + (c & 255) * 114) / 1000; }
  // Rows of a screen region with text in them (a pixel `thr` lighter or darker than the region's
  // median),
  // as blocks "y0-y1" joined by spaces; a gap of 3 rows ends a block.
  public static string InkBlocks(int x, int y, int w, int h, int thr) {
    var b = new Bitmap(w, h, PixelFormat.Format32bppArgb);
    using (var g = Graphics.FromImage(b)) g.CopyFromScreen(x, y, 0, 0, new Size(w, h));
    var d = b.LockBits(new Rectangle(0, 0, w, h), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
    var px = new int[w * h]; Marshal.Copy(d.Scan0, px, 0, px.Length); b.UnlockBits(d); b.Dispose();
    var all = new int[px.Length]; for (int i = 0; i < px.Length; i++) all[i] = Lum(px[i]);
    var s = (int[])all.Clone(); Array.Sort(s); int med = s[s.Length / 2];
    var sb = new StringBuilder(); int start = -1, last = -10;
    for (int yy = 0; yy < h; yy++) {
      bool ink = false;
      for (int xx = 0; xx < w && !ink; xx++) if (Math.Abs(all[yy * w + xx] - med) >= thr) ink = true;
      if (ink) { if (start < 0 || yy - last > 3) { if (start >= 0) sb.Append(start + "-" + last + " "); start = yy; } last = yy; }
    }
    if (start >= 0) sb.Append(start + "-" + last);
    return sb.ToString().Trim();
  }
}
'@
$Sizes = @('small', 'normal', 'large', 'extraLarge')
# Check box edge (DIPs) and the editor's text (DIPs) per size: 16 and 14 at 标准, as before.
$BoxEdge = @{ small = 14; normal = 16; large = 21; extraLarge = 27 }
$TextDip = @{ small = 12; normal = 14; large = 18; extraLarge = 24 }
$NoteText = "[ ] first to-do`n[ ] second to-do`nplain line"
PfStart -Script (DumpScript 500 25 3000)
Start-Sleep -Seconds 2

function AllIds() { return @(@((PfJson fence list)) | Where-Object { $_ -and $_.id } | ForEach-Object { "$($_.id)" }) }
function NewId($before, $sec = 10) {
  for ($i = 0; $i -lt $sec * 2; $i++) {
    $new = @(AllIds | Where-Object { $before -notcontains $_ })
    if ($new.Count) { return $new[0] }
    Start-Sleep -Milliseconds 500
  }
  return $null
}
function ToolFences($kind) { return ,@(@((PfJson fence list)) | Where-Object { $_ -and $_.tool -eq $kind }) }
function PlainFence() { return (@(@((PfJson fence list)) | Where-Object { $_ -and -not $_.tool -and $_.kind -eq 'virtual' }) | Select-Object -First 1) }
# The tray menu, as a right-click on the tray icon posts it (WM_APP+3 to the control window).
function TrayOpen() {
  $ctl = [G]::FindWindowW('PecoFence.Control', [NullString]::Value)
  [W]::PostMessageW($ctl, 0x8003, [IntPtr](1500 -bor (830 -shl 16)), [IntPtr](0x7B -bor (1 -shl 16))) | Out-Null
  return (MenuWin 5)
}
function FenceMenu($f) {
  $wr = $f.windowRect
  [G]::RClick([int]($wr.x + $wr.w / 2), [int]($wr.y + 14)); Start-Sleep -Milliseconds 700
  return (MenuWin 5)
}
# The submenu that opens from row `$regex` of menu `$h` (it lists `$has`).
function SubMenu($h, $regex, $has) {
  $p = MenuPoint $h $regex
  if (-not $p) { Log "no '$regex' in: $(MenuTexts $h)"; return [IntPtr]::Zero }
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
  foreach ($m in @([W]::Tops('#32768'))) { if ((MenuTexts $m) -match $has) { return $m } }
  return [IntPtr]::Zero
}
# 新建小工具 > `$pick` from the tray menu or a fence menu.
function TrayTool($pick) { $sub = SubMenu (TrayOpen) '^New tool' 'Weather'; if ($sub -eq [IntPtr]::Zero) { return $null }; return (MenuPick $pick $sub) }
function FenceTool($pick) {
  $add = SubMenu (FenceMenu (PlainFence)) '^Add to desktop' '^New fence'
  if ($add -eq [IntPtr]::Zero) { return $null }
  $sub = SubMenu $add '^New tool' 'Weather'
  if ($sub -eq [IntPtr]::Zero) { return $null }
  return (MenuPick $pick $sub)
}
# Hit rects of a dump field (note_boxes, script_rows) as objects: i, l, t, r, b in content DIPs.
function DumpRects($w, $name) {
  $v = DumpVal $w $name
  if (-not $v -or $v -eq '-') { return @() }
  return @($v -split ';' | ForEach-Object {
    $pair = $_ -split ':'; $n = $pair[1] -split ','
    [pscustomobject]@{ i = [int]$pair[0]; l = [double]$n[0]; t = [double]$n[1]; r = [double]$n[2]; b = [double]$n[3] }
  })
}
# Screen point of content DIPs (x, y) of a dumped window.
function ContentPt($w, $x, $y) {
  $sc = [double](DumpVal $w 'content_y') / 36.0
  return @([int]($w.l + [double](DumpVal $w 'content_x') + $x * $sc), [int]($w.t + [double](DumpVal $w 'content_y') + $y * $sc))
}
function Note($id) { return (FenceGet $id).noteText }
function ScriptOut($id) { $f = FenceGet $id; return "$($f.script.output)" }
function EditFont() {
  $ed = WaitWin 'PecoFence.NoteEdit' 5
  if ($ed -eq [IntPtr]::Zero) { return $null }
  $edit = [TT]::FindWindowExW($ed, [IntPtr]::Zero, 'Edit', [NullString]::Value)
  return @{ top = $ed; height = [TT]::FontHeight($edit); dpi = [TT]::GetDpiForWindow($ed) }
}
# The bottom-right resize corner of a window (WM_NCHITTEST 17), inside its rect.
function CornerPt($w) {
  for ($d = 1; $d -le 12; $d++) { if ((NcHit $w.hwnd ($w.r - $d) ($w.b - $d)) -eq 17) { return @(($w.r - $d), ($w.b - $d)) } }
  return @(($w.r - 3), ($w.b - 3))
}
# The content surface of a dumped window in px: @(w, h).
function ContentSize($w) { if ($w -and $w.raw -match ' content=\((\d+), (\d+)\)') { return @([int]$Matches[1], [int]$Matches[2]) }; return @(0, 0) }
function ClockText($w) { $v = DumpVal $w 'clock_text'; if (-not $v) { return @(0, 0) }; $p = $v -split '/'; return @([double]$p[0], [double]$p[1]) }
# Height in px of the clock's time (the first block of text rows in its content).
function TimeInk($w) {
  $cy = 36; $v = DumpVal $w 'content_y'; if ($v) { $cy = [int]$v }
  $blocks = [TT]::InkBlocks($w.l + 4, $w.t + $cy, ($w.r - $w.l) - 8, ($w.b - $w.t) - $cy - 2, 90)
  Log "clock ink blocks: $blocks"
  if (-not $blocks) { return 0 }
  $first = ($blocks -split ' ')[0] -split '-'
  return ([int]$first[1] - [int]$first[0] + 1)
}

# ---- notes: one note at each size; fresh text before every check.
function NoteSizes($title, $id) {
  trap { PfTrap $_; continue }
  $prevTop = -1
  foreach ($s in $Sizes) {
    Pf fence set $id noteText $NoteText | Out-Null
    $r = PfJson fence set $id textSize $s
    Check "${title} ${s}: fence set textSize" ($r.fence.textSize -eq $s) "textSize=$($r.fence.textSize)"
    $w = DumpOf $title
    $boxes = @(DumpRects $w 'note_boxes')
    $edge = if ($boxes.Count) { $boxes[0].r - $boxes[0].l } else { 0 }
    Check "${title} ${s}: the window draws it (text_size $([array]::IndexOf($Sizes, $s)), check box $($BoxEdge[$s]) DIPs)" ((DumpVal $w 'text_size') -eq "$([array]::IndexOf($Sizes, $s))" -and $boxes.Count -eq 2 -and [math]::Abs($edge - $BoxEdge[$s]) -lt 0.6) "text_size=$(DumpVal $w 'text_size') boxes=$(DumpVal $w 'note_boxes')"
    if ($boxes.Count -eq 2) {
      Check "${title} ${s}: the second to-do sits lower than at the size before (taller lines)" ($boxes[1].t -gt $prevTop) "second box top=$($boxes[1].t) before=$prevTop"
      $prevTop = $boxes[1].t
      Shot "n-$title-$s"
      # 特大: a click near the far corner of its box, where the 16 DIP box of 标准 (and its 4 DIP
      # slack) does not reach, still ticks it.
      $b0 = $boxes[0]
      $at = if ($s -eq 'extraLarge') { ContentPt $w ($b0.l + 24) ($b0.t + 24) } else { ContentPt $w (($b0.l + $b0.r) / 2) (($b0.t + $b0.b) / 2) }
      [G]::Click($at[0], $at[1]); Start-Sleep -Milliseconds 700
      [G]::MoveTo(800, 880)
      $t = Note $id
      Check "${title} ${s}: a click on the first box ticks it, no editor" ($t -like '`[x`] first to-do*' -and (WaitWin 'PecoFence.NoteEdit' 1) -eq [IntPtr]::Zero) ("noteText=" + ($t -replace "`n", ' / '))
      $b1 = $boxes[1]
      $at = ContentPt $w (($b1.l + $b1.r) / 2) (($b1.t + $b1.b) / 2)
      [G]::Click($at[0], $at[1]); Start-Sleep -Milliseconds 700
      $t = Note $id
      Check "${title} ${s}: a click on the second box ticks it" ($t -match '(?m)^\[x\] second to-do') ("noteText=" + ($t -replace "`n", ' / '))
      # A click on the first line's text opens the editor at the note's size.
      $at = ContentPt $w ($b0.r + 50) (($b0.t + $b0.b) / 2)
      [G]::Click($at[0], $at[1]); Start-Sleep -Milliseconds 800
      $ef = EditFont
      $want = - [int][math]::Round($TextDip[$s] * $ef.dpi / 96.0)
      Check "${title} ${s}: the editor opens with $($TextDip[$s]) DIP text" ($ef -and $ef.height -eq $want) "font height=$($ef.height) want=$want dpi=$($ef.dpi)"
      if ($ef) {
        Shot "n-$title-$s-editor"
        Chord 0x11 0x23; [G]::Type(' edited'); Start-Sleep -Milliseconds 200
        [G]::Press(0x1B); Start-Sleep -Milliseconds 800
      }
      $t = Note $id
      Check "${title} ${s}: Esc saves the edit" ($t -like '*plain line edited') ("noteText=" + ($t -replace "`n", ' / '))
    }
  }
}

if ($Part -eq 'N') {
# The config schema with textSize, for site/schema/config.json.
$schema = & "$PfApp\pecofence-cli.exe" describe --schema Config --compact | Out-String
[IO.File]::WriteAllText("$PfShots\config-schema.json", $schema.Trim(), (New-Object Text.UTF8Encoding($false)))
Check 'the config schema has textSize' ($schema -match '"textSize"') "length=$($schema.Length)"
Case 'N1 note from the tray menu'
$before = AllIds
$m = TrayOpen
Check 'the tray menu has New note' ((MenuTexts $m) -match 'New note') (MenuTexts $m)
MenuPick '^New note' $m | Out-Null
$ed = WaitWin 'PecoFence.NoteEdit' 10
Check 'New note opens the editor at once (standard text)' ($ed -ne [IntPtr]::Zero -and (EditFont).height -eq -14) "editor=$ed font=$((EditFont).height)"
[G]::Type($NoteText); [G]::Press(0x1B); Start-Sleep -Milliseconds 800
$id = NewId $before
Pf fence rename $id NoteTray | Out-Null
Pf fence move NoteTray --x 80 --y 90 | Out-Null
Pf fence resize NoteTray --w 380 --h 270 | Out-Null
Check 'a new note has the standard text size' ((FenceGet $id).textSize -eq 'normal') "textSize=$((FenceGet $id).textSize)"
NoteSizes 'NoteTray' $id

Case 'N2 note from the fence menu'
$before = AllIds
$add = SubMenu (FenceMenu (PlainFence)) '^Add to desktop' '^New fence'
MenuPick '^New note' $add | Out-Null
$ed = WaitWin 'PecoFence.NoteEdit' 10
Check 'Add to desktop > New note opens the editor' ($ed -ne [IntPtr]::Zero)
[G]::Type($NoteText); [G]::Press(0x1B); Start-Sleep -Milliseconds 800
$id2 = NewId $before
Pf fence rename $id2 NoteMenu | Out-Null
Pf fence move NoteMenu --x 480 --y 90 | Out-Null
Pf fence resize NoteMenu --w 380 --h 270 | Out-Null
NoteSizes 'NoteMenu' $id2

Case 'N3 note from the CLI'
$r = PfJson fence create --tool note --title NoteCli --text 'x' --rect '880,90,380,270'
$id3 = "$($r.fence.id)"
Check 'fence create --tool note makes it' ($id3 -ne '') "id=$id3"
NoteSizes 'NoteCli' $id3
$e = (Pf fence set $id3 textSize 24) -join ' '
Check 'textSize 24 is refused and lists the sizes' ($e -match 'small' -and $e -match 'extraLarge') $e

Case 'N4 an empty note shows its hint at the chosen size; a restart keeps the sizes'
Pf fence set $id noteText '""' | Out-Null
Pf fence set $id textSize extraLarge | Out-Null
Shot 'n-empty-extraLarge'
Pf fence set $id noteText $NoteText | Out-Null
Pf fence set $id2 textSize large | Out-Null
Pf fence set $id3 textSize small | Out-Null
PfQuit
PfStart -Script (DumpScript 500 25 3000)
Start-Sleep -Seconds 2
foreach ($pair in @(@('NoteTray', $id, 'extraLarge', 3), @('NoteMenu', $id2, 'large', 2), @('NoteCli', $id3, 'small', 0))) {
  $w = DumpOf $pair[0]
  $boxes = @(DumpRects $w 'note_boxes')
  $edge = if ($boxes.Count) { $boxes[0].r - $boxes[0].l } else { 0 }
  Check "after a restart $($pair[0]) is still $($pair[2])" ((FenceGet $pair[1]).textSize -eq $pair[2] -and (DumpVal $w 'text_size') -eq "$($pair[3])" -and [math]::Abs($edge - $BoxEdge[$pair[2]]) -lt 0.6) "textSize=$((FenceGet $pair[1]).textSize) dump=$(DumpVal $w 'text_size') boxes=$(DumpVal $w 'note_boxes')"
}
$cfg = Get-Content "$PfApp\config\config.json" -Raw -ErrorAction SilentlyContinue
if (-not $cfg) { $cfg = (Get-ChildItem $PfApp -Recurse -Filter config.json | Select-Object -First 1 | Get-Content -Raw) }
Check 'config.json stores textSize only where it was chosen' ($cfg -match '"textSize":\s*"extraLarge"' -and $cfg -match '"textSize":\s*"small"' -and $cfg -notmatch '"textSize":\s*null') "matches=$(([regex]::Matches($cfg, '"textSize":\s*"\w+"') | ForEach-Object { $_.Value }) -join ', ')"
Shot 'n-after-restart'
$plain = (PlainFence).id
$e = (Pf fence set $plain textSize large) -join ' '
Check 'textSize on a fence that shows items is refused' ($e -match 'note and script') $e
}

# ---- script fences
if ($Part -eq 'S') {
$data = "$PfRun\todo-state.txt"
Remove-Item $data -ErrorAction SilentlyContinue
$own = @"
`$f = '$data'
if (`$env:PECOFENCE_ACTION -eq 'toggle') { if (Test-Path `$f) { Remove-Item `$f } else { Set-Content `$f 1 } }
`$mark = if (Test-Path `$f) { 'x' } else { ' ' }
Write-Output "[`$mark] water the plants | action=toggle"
Write-Output "copy me | copy=PF-COPIED-TEXT"
Write-Output "---"
Write-Output "plain line"
"@
Set-Content C:\pf\run\mine.ps1 -Value $own -Encoding UTF8
function WaitRows($title, $n, $sec = 20) {
  for ($i = 0; $i -lt $sec * 2; $i++) { $w = DumpOf $title 3; if (@(DumpRects $w 'script_rows').Count -ge $n) { return $w }; Start-Sleep -Milliseconds 300 }
  return $w
}
# The script rows at each size: the plates grow with the text.
function ScriptSizes($title, $id, $rows) {
  trap { PfTrap $_; continue }
  $prevH = 0
  foreach ($s in $Sizes) {
    $r = PfJson fence set $id textSize $s
    Check "${title} ${s}: fence set textSize" ($r.fence.textSize -eq $s) "textSize=$($r.fence.textSize)"
    Start-Sleep -Milliseconds 600
    $w = WaitRows $title $rows
    $plates = @(DumpRects $w 'script_rows')
    $h = if ($plates.Count) { $plates[0].b - $plates[0].t } else { 0 }
    Check "${title} ${s}: the window draws it, its first clickable row taller than at the size before" ((DumpVal $w 'text_size') -eq "$([array]::IndexOf($Sizes, $s))" -and $plates.Count -ge $rows -and $h -gt $prevH) "text_size=$(DumpVal $w 'text_size') rows=$(DumpVal $w 'script_rows') prev_h=$prevH"
    $prevH = $h
    Shot "s-$title-$s"
  }
}

Case 'S1 PC status from the tray menu'
$before = AllIds
$picked = TrayTool '^PC status'
$sid = NewId $before 15
Check 'New tool > PC status makes a script fence' ($sid -and $picked) "picked=$picked id=$sid"
Pf fence rename $sid StatusTray | Out-Null
Pf fence move StatusTray --x 80 --y 90 | Out-Null
Pf fence resize StatusTray --w 440 --h 330 | Out-Null
$w = WaitRows 'StatusTray' 1 30
ScriptSizes 'StatusTray' $sid 1
# 特大: a click low on the 「C: free」 row (open=C:\) opens C:\ in File Explorer. (Its IP row
# shows its parameters as text: the example's unquoted tooltip=Click to copy, before this change
# too.)
CloseWindows
$w = WaitRows 'StatusTray' 1
$p = @(DumpRects $w 'script_rows')[0]
if ($p) { $at = ContentPt $w (($p.l + $p.r) / 2) ($p.b - 4); [G]::Click($at[0], $at[1]) }
$ex = ''
for ($i = 0; $i -lt 20 -and $ex -notmatch 'path=C:\\'; $i++) { Start-Sleep -Milliseconds 500; $ex = Explorers }
[G]::MoveTo(800, 880)
Check 'StatusTray extraLarge: a click low on the C: row opens C:\ in File Explorer' ($ex -match 'path=C:\\') "explorers=$($ex -replace "`n", ' ; ') plate=$($p | ConvertTo-Json -Compress)"
CloseWindows

Case 'S2 Weather from the fence menu'
$before = AllIds
$picked = FenceTool '^Weather'
$wid = NewId $before 15
Check 'Add to desktop > New tool > Weather makes a script fence' ($wid -and $picked) "picked=$picked id=$wid"
Pf fence rename $wid WeatherMenu | Out-Null
Pf fence move WeatherMenu --x 560 --y 90 | Out-Null
Pf fence resize WeatherMenu --w 380 --h 260 | Out-Null
Start-Sleep -Seconds 8
foreach ($s in $Sizes) {
  $r = PfJson fence set $wid textSize $s
  Start-Sleep -Milliseconds 600
  $w = DumpOf 'WeatherMenu'
  Check "WeatherMenu ${s}: set and drawn" ($r.fence.textSize -eq $s -and (DumpVal $w 'text_size') -eq "$([array]::IndexOf($Sizes, $s))") "textSize=$($r.fence.textSize) dump=$(DumpVal $w 'text_size') out=$((ScriptOut $wid) -replace "`n", ' / ')"
  Shot "s-weather-$s"
}

Case 'S3 own script from the CLI'
$r = PfJson fence create --script C:\pf\run\mine.ps1 --title Mine --rect '560,400,400,260'
$mid = "$($r.fence.id)"
Check 'fence create --script makes it' ($mid -ne '') "id=$mid"
$w = WaitRows 'Mine' 2 30
ScriptSizes 'Mine' $mid 2
foreach ($s in $Sizes) {
  Pf fence set $mid textSize $s | Out-Null
  Remove-Item $data -ErrorAction SilentlyContinue
  Pf fence refresh $mid | Out-Null
  Start-Sleep -Seconds 3
  $w = WaitRows 'Mine' 2
  $plates = @(DumpRects $w 'script_rows')
  Check "Mine ${s}: fresh state, the to-do is open" ((ScriptOut $mid) -like '`[ `] water*') (ScriptOut $mid)
  # The to-do row: a click low on its plate (at 特大 below where the 标准 row ends) ticks it.
  $p = $plates[0]
  $at = ContentPt $w (($p.l + $p.r) / 2) ($p.b - 3)
  [G]::Click($at[0], $at[1]); Start-Sleep -Seconds 3
  [G]::MoveTo(800, 880)
  Check "Mine ${s}: a click on the to-do row ticks it (the script ran with its action)" ((ScriptOut $mid) -like '`[x`] water*' -and (Test-Path $data)) "out=$((ScriptOut $mid) -replace "`n", ' / ') plate=$($p | ConvertTo-Json -Compress)"
  Set-Clipboard -Value 'nothing yet'
  $w = WaitRows 'Mine' 2
  $p = @(DumpRects $w 'script_rows')[1]
  $at = ContentPt $w (($p.l + $p.r) / 2) ($p.b - 3)
  [G]::Click($at[0], $at[1]); Start-Sleep -Milliseconds 800
  [G]::MoveTo(800, 880)
  Check "Mine ${s}: a click on the copy row copies its text" ("$(Get-Clipboard)".Trim() -eq 'PF-COPIED-TEXT') "clip='$(Get-Clipboard)' plate=$($p | ConvertTo-Json -Compress)"
  Shot "s-mine-$s-clicked"
}

Case 'S4 a restart keeps the sizes'
Pf fence set $sid textSize large | Out-Null
Pf fence set $mid textSize extraLarge | Out-Null
PfQuit
PfStart -Script (DumpScript 500 25 3000)
Start-Sleep -Seconds 4
$w = WaitRows 'Mine' 2
Check 'after a restart Mine is still extraLarge' ((FenceGet $mid).textSize -eq 'extraLarge' -and (DumpVal $w 'text_size') -eq '3') "textSize=$((FenceGet $mid).textSize) dump=$(DumpVal $w 'text_size')"
$w = DumpOf 'StatusTray'
Check 'after a restart StatusTray is still large' ((FenceGet $sid).textSize -eq 'large' -and (DumpVal $w 'text_size') -eq '2') "textSize=$((FenceGet $sid).textSize) dump=$(DumpVal $w 'text_size')"
Shot 's-after-restart'
}

# ---- the clock
function ClockChecks($label, $id) {
  trap { PfTrap $_; continue }
  Pf fence move $id --x 120 --y 120 | Out-Null
  Start-Sleep -Milliseconds 600
  $w = DumpOf 'Clock'
  $ct = ClockText $w
  Check "${label}: a new clock shows the time at 40 and the date at 14 DIPs, as before" ($ct[0] -eq 40 -and $ct[1] -eq 14) "clock_text=$(DumpVal $w 'clock_text') window=$($w.r - $w.l)x$($w.b - $w.t)"
  $ink0 = TimeInk $w
  Shot "c-$label-new"
  # Bigger with the mouse: one log line for the whole drag, with the size it ended at.
  # The app's own log (the shareable copy repeats each line).
  $logBefore = @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match 'clock text size' }).Count
  $c = CornerPt $w
  DragTo $c[0] $c[1] ($c[0] + 240) ($c[1] + 140) 20
  Start-Sleep -Milliseconds 800
  $w = DumpOf 'Clock'
  $ct = ClockText $w
  $ch = (ContentSize $w)[1]
  $want = [math]::Min(160, [math]::Round(40 * $ch / 104.0))
  $ink1 = TimeInk $w
  Check "${label}: dragged bigger, the time grows with the height ($want DIPs)" ($ct[0] -ge $want - 1 -and $ct[0] -le $want -and $ct[0] -gt 40 -and $ink1 -gt $ink0 * 1.6) "clock_text=$(DumpVal $w 'clock_text') content_h=$ch ink $ink0 -> $ink1 px"
  $lines = @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match 'clock text size' })
  Check "${label}: the drag logs the size once, at its end" ($lines.Count -eq $logBefore + 1 -and $lines[-1] -match "time_dip=$([int]$ct[0])\b") "new lines=$($lines.Count - $logBefore) last=$($lines[-1])"
  Shot "c-$label-bigger"
  # As small as a fence goes (150 x 150): smaller text than a new clock, the date at least 11
  # DIPs, and the time still inside the width.
  $c = CornerPt $w
  DragTo $c[0] $c[1] ($c[0] - 330) ($c[1] - 200) 20
  Start-Sleep -Milliseconds 800
  $w = DumpOf 'Clock'
  $ct = ClockText $w
  $edge = [TT]::InkBlocks($w.r - 11, $w.t + [int](DumpVal $w 'content_y'), 9, 60, 90)
  Check "${label}: dragged to the smallest fence, the text gets smaller and stays inside" ($ct[0] -lt 40 -and $ct[0] -ge 24 -and $ct[1] -ge 11 -and -not $edge) "clock_text=$(DumpVal $w 'clock_text') window=$($w.r - $w.l)x$($w.b - $w.t) ink at the right edge='$edge'"
  Shot "c-$label-small"
  # By the CLI; then very large: at most 160 DIPs.
  Pf fence resize $id --w 360 --h 200 | Out-Null
  Start-Sleep -Milliseconds 800
  $w = DumpOf 'Clock'
  $ct = ClockText $w
  $ch = (ContentSize $w)[1]
  $want = [math]::Round(40 * $ch / 104.0)
  Check "${label}: fence resize 360x200 gives $want DIPs" ($ct[0] -eq $want) "clock_text=$(DumpVal $w 'clock_text') content_h=$ch"
  Pf fence resize $id --w 1300 --h 760 | Out-Null
  Start-Sleep -Milliseconds 800
  $w = DumpOf 'Clock'
  $ct = ClockText $w
  Check "${label}: a huge clock stops at 160 / 56 DIPs" ($ct[0] -eq 160 -and $ct[1] -eq 56) "clock_text=$(DumpVal $w 'clock_text')"
  Shot "c-$label-huge"
  Pf fence resize $id --w 240 --h 140 | Out-Null
  Start-Sleep -Milliseconds 800
  $w = DumpOf 'Clock'
  $ct = ClockText $w
  Check "${label}: back to the size of a new clock, back to 40 / 14" ($ct[0] -eq 40 -and $ct[1] -eq 14) "clock_text=$(DumpVal $w 'clock_text')"
}

if ($Part -eq 'C') {
Case 'C1 clock from the tray menu'
$before = AllIds
$picked = TrayTool '^Clock$'
$cid = NewId $before
Check 'New tool > Clock in the tray menu makes a clock' ($cid -and $picked) "picked=$picked id=$cid"
ClockChecks 'tray' $cid
# Seconds make the time wider: in a tall narrow clock the width sets the size, and it fits.
Pf fence resize $cid --w 250 --h 320 | Out-Null
Start-Sleep -Milliseconds 800
$noSec = ClockText (DumpOf 'Clock')
Pf settings set tools.clockSeconds true | Out-Null
Start-Sleep -Milliseconds 1500
$w = DumpOf 'Clock'
$sec = ClockText $w
$cy = [int](DumpVal $w 'content_y')
$ink = [TT]::InkBlocks($w.r - 11, $w.t + $cy, 9, 120, 90)
Check 'with seconds the time gets smaller in a narrow clock, and none of it reaches the right edge' ($sec[0] -lt $noSec[0] -and -not $ink) "no seconds=$($noSec -join '/') seconds=$($sec -join '/') ink at the right edge='$ink'"
Shot 'c-seconds-narrow'
Pf settings set tools.clockSeconds false | Out-Null
Pf fence delete $cid | Out-Null
Start-Sleep -Seconds 1

Case 'C2 clock from the fence menu'
$before = AllIds
$picked = FenceTool '^Clock$'
$cid = NewId $before
Check 'Add to desktop > New tool > Clock makes a clock' ($cid -and $picked) "picked=$picked id=$cid"
ClockChecks 'menu' $cid
Pf fence delete $cid | Out-Null
Start-Sleep -Seconds 1

Case 'C3 clock from the CLI, and a restart'
$r = PfJson fence create --tool clock --title Clock
$cid = "$($r.fence.id)"
Check 'fence create --tool clock makes a clock' ($cid -ne '') "id=$cid"
ClockChecks 'cli' $cid
Pf fence resize $cid --w 420 --h 240 | Out-Null
Start-Sleep -Milliseconds 800
$beforeQuit = ClockText (DumpOf 'Clock')
$e = (Pf fence set $cid textSize large) -join ' '
Check 'textSize on a clock is refused (its text follows the fence size)' ($e -match 'clock') $e
PfQuit
PfStart -Script (DumpScript 500 25 3000)
Start-Sleep -Seconds 2
$after = ClockText (DumpOf 'Clock')
Check 'after a restart the clock shows the same text size' ($after[0] -eq $beforeQuit[0] -and $after[1] -eq $beforeQuit[1] -and $after[0] -gt 40) "before=$($beforeQuit -join '/') after=$($after -join '/')"
$l = @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match 'clock text size' }) | Select-Object -Last 1
Check 'the start logs the clock text size' ($l -match "time_dip=$([int]$after[0])\b") $l
Shot 'c-after-restart'
}

# ---- the settings page (debug build): 文字大小 for notes and script fences only.
if ($Part -eq 'P') {
Case 'P1 fence page'
Check 'a debug build (CDP)' $PfDebugBuild
$n = PfJson fence create --tool note --title PageNote --text "[ ] page to-do`nline" --rect '80,90,380,260'
$nid = "$($n.fence.id)"
Set-Content C:\pf\run\p.ps1 -Value 'Write-Output "row | copy=x"' -Encoding UTF8
$sp = PfJson fence create --script C:\pf\run\p.ps1 --title PageScript --rect '480,90,380,200'
$spid = "$($sp.fence.id)"
$ck = PfJson fence create --tool clock --title Clock --rect '880,90,260,140'
$ckid = "$($ck.fence.id)"
$ok = PfSettings
Check 'settings open over CDP' $ok
function PageFence($id) {
  Pf fence open-options $id | Out-Null
  Start-Sleep -Milliseconds 800
  return (JsWait "document.getElementById('fenceSel').value === '$id'" 10)
}
# Focuses the select and moves to `$value` with the arrow keys (real keyboard input).
function PageArrowPick($sel, $value) {
  PageClick '#fenceTextSize .t'
  $ix = Js "(() => { const s = document.querySelector('$sel'); s.scrollIntoView({ block: 'center' }); s.focus(); return [s.selectedIndex, [...s.options].findIndex(o => o.value === '$value')]; })()"
  $n = $ix[1] - $ix[0]
  $vk = if ($n -gt 0) { 0x28 } else { 0x26 }
  for ($k = 0; $k -lt [Math]::Abs($n); $k++) { [G]::Press($vk); Start-Sleep -Milliseconds 120 }
  Start-Sleep -Milliseconds 600
}
foreach ($pair in @(@($nid, 'PageNote'), @($spid, 'PageScript'))) {
  $shown = PageFence $pair[0]
  $vis = Js "getComputedStyle(document.getElementById('fenceTextSize')).display !== 'none'"
  Check "$($pair[1]): Fence options shows Text size" ($shown -and $vis) "shown=$shown visible=$vis"
  foreach ($s in @('extraLarge', 'small', 'large', 'normal')) {
    PageArrowPick '#fenceTextSizeSel' $s
    $w = DumpOf $pair[1]
    Check "$($pair[1]): picking $s on the page sets it" ((FenceGet $pair[0]).textSize -eq $s -and (DumpVal $w 'text_size') -eq "$([array]::IndexOf($Sizes, $s))") "textSize=$((FenceGet $pair[0]).textSize) dump=$(DumpVal $w 'text_size') page=$(Js "document.getElementById('fenceTextSizeSel').value")"
  }
  Pf fence set $pair[0] textSize large | Out-Null
  Start-Sleep -Milliseconds 800
  Check "$($pair[1]): a change from the CLI shows on the page" ((Js "document.getElementById('fenceTextSizeSel').value") -eq 'large')
}
$shown = PageFence $nid
Shot 'p-note-options'
$plain = (PlainFence).id
foreach ($pair in @(@($ckid, 'the clock'), @($plain, 'a fence with icons'))) {
  $shown = PageFence $pair[0]
  $vis = Js "getComputedStyle(document.getElementById('fenceTextSize')).display !== 'none'"
  Check "$($pair[1]): Fence options has no Text size" ($shown -and -not $vis) "shown=$shown visible=$vis"
}
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}

# ---- the old build: what it did (run with --bin of the build before the change).
if ($Part -eq 'B') {
Case 'B old build'
$r = PfJson fence create --tool clock --title Clock --rect '120,120,240,140'
Start-Sleep -Milliseconds 800
$w = DumpOf 'Clock'
$ink0 = TimeInk $w
$c = CornerPt $w
DragTo $c[0] $c[1] ($c[0] + 240) ($c[1] + 140) 20
Start-Sleep -Milliseconds 800
$w = DumpOf 'Clock'
$ink1 = TimeInk $w
Check 'old build: the time keeps its size when the clock is dragged bigger' ([math]::Abs($ink1 - $ink0) -le 1) "ink $ink0 -> $ink1 px, window $($w.r - $w.l)x$($w.b - $w.t)"
Shot 'b-clock-bigger'
$n = PfJson fence create --tool note --title NoteOld --text "[ ] first to-do" --rect '520,120,380,260'
$e = (Pf fence set NoteOld textSize extraLarge) -join ' '
Check 'old build: notes have no text size (unknown property)' ($e -match 'unknown fence property') $e
}

# ---- before/after recordings (the same steps on both builds; $Side = before | after).
if ($Part -eq 'A') {
Case 'A recordings'
if (-not $Side) { $Side = 'after' }
Pf fence create --tool clock --title Clock --rect '160,130,240,140' | Out-Null
Pf fence create --tool note --title Note --text "[ ] Buy milk`n[ ] Call the bank`nMeeting at 3 pm" --rect '820,130,380,250' | Out-Null
Start-Sleep -Seconds 2
[G]::MoveTo(500, 600)
RecStart "clock-$Side" -X 100 -Y 80 -W 720 -H 480 -Fps 15
RecMark '时钟是新建时的大小'
Start-Sleep -Milliseconds 1500
$w = DumpOf 'Clock'
$c = CornerPt $w
RecMark '拖右下角，把时钟拉大'
DragTo $c[0] $c[1] ($c[0] + 320) ($c[1] + 190) 40
RecMark '松开鼠标'
Start-Sleep -Milliseconds 2500
RecStop
[G]::MoveTo(1300, 600)
RecStart "note-$Side" -X 780 -Y 80 -W 720 -H 480 -Fps 15
RecMark '便签是标准文字大小'
Start-Sleep -Milliseconds 1500
RecMark '把文字大小选成特大'
Pf fence set Note textSize extraLarge | Out-Null
Start-Sleep -Milliseconds 2000
$w = DumpOf 'Note'
$boxes = @(DumpRects $w 'note_boxes')
RecMark '点第一行的方框打勾'
$at = if ($boxes.Count) { ContentPt $w (($boxes[0].l + $boxes[0].r) / 2) (($boxes[0].t + $boxes[0].b) / 2) } else { @(($w.l + 22), ($w.t + 47)) }
Glide 1300 600 $at[0] $at[1] 12 25
[G]::Click($at[0], $at[1])
Start-Sleep -Milliseconds 2000
RecStop
}

Case 'cleanup'
PfQuit
