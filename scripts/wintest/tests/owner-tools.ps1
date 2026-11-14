# Owner round 2026-11-12, tool fences (notes, clock, script fences), zh-CN.
# Parts (run.sh --var Part=...):
#   A  menu icons of 编辑脚本 / 编辑便签, a script that prints and then fails, the date of a
#      narrow clock, clicks that end the note editor and where the keyboard goes after it
#   B  text longer than a note or a script fence scrolls (wheel, scrollbar, whole lines)
#   C  the note editor looks like the note (same lines, boxes, background, no frame)
#   I  Chinese input (Microsoft Pinyin) in the note editor
#   R  the recordings for the before/after animations (no checks of the product)
trap { PfTrap $_; continue }
if (-not $Part) { $Part = 'A' }
Case 'setup'
CloseWindows
Set-Culture zh-CN
GreyWallpaper
$Anim0 = [Anim]::Get()
. C:\pf\kit\rec.ps1
Add-Type -ReferencedAssemblies System.Drawing @'
using System; using System.Text; using System.Drawing; using System.Drawing.Imaging;
using System.Runtime.InteropServices;
public static class OT {
  static int[] Grab(int x, int y, int w, int h) {
    var b = new Bitmap(w, h, PixelFormat.Format32bppArgb);
    using (var g = Graphics.FromImage(b)) g.CopyFromScreen(x, y, 0, 0, new Size(w, h));
    var d = b.LockBits(new Rectangle(0, 0, w, h), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
    var px = new int[w * h]; Marshal.Copy(d.Scan0, px, 0, px.Length); b.UnlockBits(d); b.Dispose();
    return px;
  }
  static int Lum(int c) { return (((c >> 16) & 255) * 299 + ((c >> 8) & 255) * 587 + (c & 255) * 114) / 1000; }
  static bool Red(int c) { int r = (c >> 16) & 255, g = (c >> 8) & 255, b = c & 255; return r - Math.Max(g, b) >= 60; }
  // Rows of a screen region with text in them (a pixel `thr` lighter or darker than the
  // region's median, or red when `red`), as blocks "y0-y1" joined by spaces; a gap of 3 rows
  // ends a block.
  public static string Blocks(int x, int y, int w, int h, int thr, bool red) {
    var px = Grab(x, y, w, h);
    var all = new int[px.Length]; for (int i = 0; i < px.Length; i++) all[i] = Lum(px[i]);
    var s = (int[])all.Clone(); Array.Sort(s); int med = s[s.Length / 2];
    var sb = new StringBuilder(); int start = -1, last = -10;
    for (int yy = 0; yy < h; yy++) {
      bool ink = false;
      for (int xx = 0; xx < w && !ink; xx++) { int i = yy * w + xx; if (red ? Red(px[i]) : Math.Abs(all[i] - med) >= thr) ink = true; }
      if (ink) { if (start < 0 || yy - last > 3) { if (start >= 0) sb.Append(start + "-" + last + " "); start = yy; } last = yy; }
    }
    if (start >= 0) sb.Append(start + "-" + last);
    return sb.ToString().Trim();
  }
  // Mean brightness difference of two screen regions of the same size.
  public static double Diff(int x0, int y0, int x1, int y1, int w, int h) {
    var a = Grab(x0, y0, w, h); var b = Grab(x1, y1, w, h); double sum = 0;
    for (int i = 0; i < a.Length; i++) sum += Math.Abs(Lum(a[i]) - Lum(b[i]));
    return sum / a.Length;
  }
  // Pixels of a region that differ from the same region grabbed before (`before`), by `thr`.
  public static int[] Snap(int x, int y, int w, int h) { return Grab(x, y, w, h); }
  public static int Changed(int[] before, int x, int y, int w, int h, int thr) {
    var now = Grab(x, y, w, h); int n = 0;
    for (int i = 0; i < now.Length; i++) if (Math.Abs(Lum(now[i]) - Lum(before[i])) >= thr) n++;
    return n;
  }
}
'@
$sd = "$PfRun\scripts"
New-Item -ItemType Directory -Force $sd | Out-Null
$utf8 = New-Object Text.UTF8Encoding($true)
[IO.File]::WriteAllText("$sd\出错的脚本.ps1", "Write-Output '开始同步'`r`nWrite-Output '已下载 3 个文件'`r`nthrow '连接服务器失败：超时'`r`n", $utf8)
[IO.File]::WriteAllText("$sd\只有退出代码.cmd", "@echo off`r`necho step one`r`nexit /b 3`r`n", (New-Object Text.UTF8Encoding($false)))
[IO.File]::WriteAllText("$sd\很多行.ps1", "1..60 | ForEach-Object { Write-Output ('第 {0} 行：一段输出' -f `$_) }`r`n", $utf8)
[IO.File]::WriteAllText("$sd\状态.ps1", "Write-Output '一行没有动作的普通文字'`r`nWrite-Output '第二行'`r`n", $utf8)
PfStart -Lang 'zh-CN' -Script (DumpScript 500 25 3000)
Start-Sleep -Seconds 3
ClearPfNotices
ClearFences
Pf settings set theme light | Out-Null
function NewTool($title, $rect, $kind, $arg) {
  if ($kind -eq 'script') { $r = PfJson fence create --script $arg --title $title --rect $rect }
  elseif ($kind -eq 'note') { $r = PfJson fence create --tool note --title $title --text $arg --rect $rect }
  else { $r = PfJson fence create --tool clock --title $title --rect $rect }
  return "$($r.fence.id)"
}
function ScriptOf($id) { $f = FenceGet $id; return $f.script }
function WaitRun($id, $sec = 20) { for ($i = 0; $i -lt $sec * 2; $i++) { $s = ScriptOf $id; if ($s.outcome) { return $s }; Start-Sleep -Milliseconds 500 }; return (ScriptOf $id) }
# Screen point of content DIPs (x, y) of a dumped window, and its content rect in px.
function CPt($w, $x, $y) { return @([int]($w.l + [double](DumpVal $w 'content_x') + $x), [int]($w.t + [double](DumpVal $w 'content_y') + $y)) }
function CRect($w) { $cx = [int](DumpVal $w 'content_x'); $cy = [int](DumpVal $w 'content_y'); return @(($w.l + $cx), ($w.t + $cy), ($w.r - $w.l - $cx), ($w.b - $w.t - $cy)) }
function EditorOpen() { $h = [G]::FindWindowW('PecoFence.NoteEdit', [NullString]::Value); return ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) }
function TitleMenuOf($w) { [G]::RClick([int](($w.l + $w.r) / 2), $w.t + 14); Start-Sleep -Milliseconds 700; return (MenuWin 5) }
# The icon cell of a menu row (left of its text) differs from another row's by this much.
function IconDiff($m, $a, $b) {
  $rows = @(MenuRows $m)
  $ra = $rows | Where-Object { $_.text -match $a } | Select-Object -First 1
  $rb = $rows | Where-Object { $_.text -match $b } | Select-Object -First 1
  if (-not $ra -or -not $rb) { return -1 }
  return [OT]::Diff($ra.L + 2, $ra.T + 2, $rb.L + 2, $rb.T + 2, 34, ($ra.B - $ra.T - 4))
}

if ($Part -eq 'A') {
Case 'A1 menu icons'
$sid = NewTool '出错的脚本' '60,60,300,170' 'script' "$sd\出错的脚本.ps1"
$nid = NewTool '便签' '400,60,260,220' 'note' "周会准备`n[ ] 整理上周的数据`n[x] 订会议室`n备注：周五前完成"
$cid = NewTool '时钟' '700,60,240,140' 'clock' ''
Start-Sleep -Seconds 2
$ws = DumpOf '出错的脚本'; $wn = DumpOf '便签'; $wc = DumpOf '时钟'
$m = TitleMenuOf $ws
$d = IconDiff $m '^编辑脚本' '^重命名'
$d2 = IconDiff $m '^打开脚本所在的文件夹' '^重命名'
Shot 'a1-script-menu'
Check '「编辑脚本」 and 「重命名…」 have different icons' ($d -ge 5) "icon difference $([math]::Round($d, 2)) (two different icons, for comparison: $([math]::Round($d2, 2))); menu: $(MenuTexts $m)"
CloseMenus
$m = TitleMenuOf $wn
$d = IconDiff $m '^编辑便签' '^重命名'
Shot 'a1-note-menu'
Check '「编辑便签」 and 「重命名…」 have different icons' ($d -ge 5) "icon difference $([math]::Round($d, 2))"
CloseMenus

Case 'A2 a script that prints and then fails'
$s = WaitRun $sid
$r = CRect $ws
ParkMouse 500 700
Start-Sleep -Milliseconds 600
$red = [OT]::Blocks($r[0], $r[1], $r[2] - 14, $r[3] - 30, 60, $true)
$ink = [OT]::Blocks($r[0], $r[1], $r[2] - 14, $r[3] - 30, 70, $false)
Crop 'a2-failed-script' ($ws.l - 10) ($ws.t - 10) ($ws.r - $ws.l + 20) ($ws.b - $ws.t + 20) | Out-Null
$redTop = if ($red) { [int](($red -split ' ')[0] -split '-')[0] } else { -1 }
$inkRows = @($ink -split ' ' | Where-Object { $_ }).Count
Check 'the run failed with what it printed kept' ($s.outcome -eq 'failed' -and "$($s.output)" -match '开始同步' -and "$($s.output)" -match '已下载 3 个文件') "outcome=$($s.outcome) output=$(("$($s.output)" -replace "`n", ' / '))"
Check 'the error line is shown in red, first' ($redTop -ge 0 -and $redTop -le 12) "red rows: '$red' (first text row at 0 to 12 px)"
Check 'the error and both printed lines show (three text rows)' ($inkRows -eq 3) "text rows: '$ink'"
$log = @(PfLogAll | Where-Object { $_ -match 'a failed run shows its error line above its output' })
Check 'the log says the error line is shown' ($log.Count -ge 1) "lines=$($log.Count) $(if ($log.Count) { $log[-1] -replace '^\S+\s+', '' })"
# A failure without any error text: what it printed stays, nothing red.
$eid = NewTool '只有退出代码' '60,260,300,150' 'script' "$sd\只有退出代码.cmd"
$s = WaitRun $eid
$we = DumpOf '只有退出代码'
$r = CRect $we
Start-Sleep -Milliseconds 600
$red = [OT]::Blocks($r[0], $r[1], $r[2] - 14, $r[3] - 30, 60, $true)
$ink = [OT]::Blocks($r[0], $r[1], $r[2] - 14, $r[3] - 30, 70, $false)
Check 'exit code 3 without an error text: the printed line shows, nothing red' ($s.outcome -eq 'failed' -and "$($s.output)" -match 'step one' -and -not $red -and @($ink -split ' ' | Where-Object { $_ }).Count -eq 1) "outcome=$($s.outcome) output=$($s.output) red='$red' rows='$ink'"

Case 'A3 the date of a narrow clock'
Pf fence move $cid --rect '700,60,120,90' | Out-Null
Start-Sleep -Milliseconds 1200
$wc = DumpOf '时钟'
$r = CRect $wc
$ink = [OT]::Blocks($r[0], $r[1], $r[2], $r[3], 60, $false)
$edge = [OT]::Blocks($r[0] + $r[2] - 9, $r[1], 8, $r[3], 60, $false)
Crop 'a3-clock-narrow' ($wc.l - 10) ($wc.t - 10) 260 130 | Out-Null
$rows = @($ink -split ' ' | Where-Object { $_ })
Check 'a clock 120 px wide shows the short date' ((DumpVal $wc 'clock_date') -eq 'short') "clock_date=$(DumpVal $wc 'clock_date') clock_text=$(DumpVal $wc 'clock_text')"
Check 'the time and one line of date, nothing at the right edge' ($rows.Count -eq 2 -and -not $edge) "text rows: '$ink' ink in the last 8 px: '$edge'"
$line = @(PfLogAll | Where-Object { $_ -match 'clock date: the form that fits' -and $_ -match 'date="?short' }) | Select-Object -Last 1
Check 'the short date is the month and day with the short weekday' ("$line" -match 'text="?\d+月\d+日 周.') ("$line" -replace '^\S+\s+', '')
Pf fence move $cid --rect '700,60,400,110' | Out-Null
Start-Sleep -Milliseconds 1200
$wc = DumpOf '时钟'
Check 'a wide clock shows the long date' ((DumpVal $wc 'clock_date') -eq 'long') "clock_date=$(DumpVal $wc 'clock_date')"
Crop 'a3-clock-wide' ($wc.l - 10) ($wc.t - 10) 430 130 | Out-Null
Pf fence move $cid --rect '700,60,240,140' | Out-Null
Start-Sleep -Milliseconds 900

Case 'A4 clicks that end the note editor'
$wn = DumpOf '便签'; $wc = DumpOf '时钟'; $ws = DumpOf '出错的脚本'
# A click on the desktop before each case: the desktop, not the note's fence, is the window
# under the editor.
DeskClick 500 700 | Out-Null; Start-Sleep -Milliseconds 500
$p = CPt $wn 120 100
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900
Check 'a click on the note opens its editor' (EditorOpen) "fg=$(Fg)"
[G]::Type(' 加一句'); [G]::Press(0x1B); Start-Sleep -Milliseconds 900
$t = (FenceGet $nid).noteText
Check 'Esc saves, and the note''s fence has the keyboard (not the taskbar)' (-not (EditorOpen) -and $t -match '加一句' -and (Fg) -match "^PecoFence\.Fence '便签'") "open=$(EditorOpen) fg=$(Fg)"
DeskClick 500 700 | Out-Null; Start-Sleep -Milliseconds 500
$p = CPt $wn 120 100
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900
[G]::Type(' 四'); Chord 0x11 0x0D; Start-Sleep -Milliseconds 900
$t = (FenceGet $nid).noteText
Check 'Ctrl+Enter saves, and the note''s fence has the keyboard' (-not (EditorOpen) -and $t -match '四' -and (Fg) -match "^PecoFence\.Fence '便签'") "open=$(EditorOpen) fg=$(Fg)"
DeskClick 500 700 | Out-Null; Start-Sleep -Milliseconds 500
$p = CPt $wn 120 100
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900
[G]::Type(' 再加'); Start-Sleep -Milliseconds 300
$p = CPt $wc 120 50
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900
$t = (FenceGet $nid).noteText
Check 'a click on the clock closes the editor and saves' (-not (EditorOpen) -and $t -match '再加') "open=$(EditorOpen) fg=$(Fg) note=$($t -replace "`n", ' / ')"
Check 'the keyboard is with the note''s fence after that click' ((Fg) -match "^PecoFence\.Fence '便签'") "fg=$(Fg)"
if (EditorOpen) { [G]::Press(0x1B); Start-Sleep -Milliseconds 600 }
DeskClick 500 700 | Out-Null; Start-Sleep -Milliseconds 500
$p = CPt $wn 120 100
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900
[G]::Type(' 三'); Start-Sleep -Milliseconds 300
$p = CPt $ws 120 40
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900
$t = (FenceGet $nid).noteText
Check 'a click on a script fence''s text closes the editor and saves' (-not (EditorOpen) -and $t -match '三') "open=$(EditorOpen) fg=$(Fg) note=$($t -replace "`n", ' / ')"
if (EditorOpen) { [G]::Press(0x1B); Start-Sleep -Milliseconds 600 }
# From one note's editor straight into another note: the first saves, the second opens and
# keeps the keyboard.
$n2 = NewTool '便签二' '400,320,260,160' 'note' "第二张便签"
Start-Sleep -Milliseconds 900
$w2 = DumpOf '便签二'
DeskClick 500 700 | Out-Null; Start-Sleep -Milliseconds 500
$p = CPt $wn 120 100
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900
[G]::Type(' 甲'); Start-Sleep -Milliseconds 300
$p = CPt $w2 200 60
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 1200
$open2 = EditorOpen; $fg2 = Fg
[G]::Type(' 乙'); [G]::Press(0x1B); Start-Sleep -Milliseconds 900
Check 'a click from one note''s editor into another note saves the first and edits the second' ($open2 -and $fg2 -match 'NoteEdit' -and (FenceGet $nid).noteText -match '甲' -and (FenceGet $n2).noteText -match '乙') "second editor open=$open2 fg=$fg2; first: '$(((FenceGet $nid).noteText -split "`n")[-1])' second: '$((FenceGet $n2).noteText)'"
# A click on the desktop still closes it, and the keyboard goes where the click went.
$p = CPt $wn 120 100
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900
[G]::Type(' 五'); Start-Sleep -Milliseconds 300
$dp = DeskClick 500 700; Start-Sleep -Milliseconds 700
$t = (FenceGet $nid).noteText
Check 'a click on the desktop closes the editor, saves, and keeps the keyboard there' (-not (EditorOpen) -and $t -match '五' -and (Fg) -notmatch "'便签'") "open=$(EditorOpen) fg=$(Fg)"
}

if ($Part -eq 'B') {
# Hit rects of a dump field (note_boxes, script_rows) as objects: i, l, t, r, b in content DIPs.
function DumpRects($w, $name) {
  $v = DumpVal $w $name
  if (-not $v -or $v -eq '-') { return @() }
  return @($v -split ';' | ForEach-Object {
    $pair = $_ -split ':'; $n = $pair[1] -split ','
    [pscustomobject]@{ i = [int]$pair[0]; l = [double]$n[0]; t = [double]$n[1]; r = [double]$n[2]; b = [double]$n[3] }
  })
}
function ScrollOf($title) { $w = DumpOf $title; return [double](DumpVal $w 'scroll') }
# Text rows of a fence's content (without the footer strip and the scrollbar strip).
function TextRows($w, $below = 30) { $r = CRect $w; return [OT]::Blocks($r[0], $r[1], $r[2] - 14, $r[3] - $below, 70, $false) }
function FirstRow($rows) { if (-not $rows) { return @(-1, -1) }; $p = ($rows -split ' ')[0] -split '-'; return @([int]$p[0], [int]$p[1]) }
function WheelOver($w, $notches) { $p = CPt $w 120 60; Wheel $p[0] $p[1] $notches; Start-Sleep -Milliseconds 900 }
[Anim]::Set($true)

Case 'B1 a script fence with more rows than it holds'
$mid = NewTool '很多行' '60,60,300,230' 'script' "$sd\很多行.ps1"
WaitRun $mid | Out-Null
Start-Sleep -Milliseconds 800
ParkMouse 500 700
$w = DumpOf '很多行'
$r = CRect $w
$rows0 = TextRows $w
$first0 = FirstRow $rows0
$foot0 = [OT]::Snap($r[0], $r[1] + $r[3] - 26, $r[2] - 14, 24)
Crop 'b1-many-0' ($w.l - 10) ($w.t - 10) ($w.r - $w.l + 20) ($w.b - $w.t + 20) | Out-Null
Check 'at first the rows start at the top, not scrolled' ((ScrollOf '很多行') -eq 0 -and $first0[0] -ge 0) "scroll=$(ScrollOf '很多行') rows: '$rows0'"
WheelOver $w -1
$s1 = ScrollOf '很多行'
$rows1 = TextRows $w
$first1 = FirstRow $rows1
Crop 'b1-many-1' ($w.l - 10) ($w.t - 10) ($w.r - $w.l + 20) ($w.b - $w.t + 20) | Out-Null
Check 'one wheel notch scrolls the rows' ($s1 -gt 20) "scroll $s1"
Check 'at rest the first row is whole and starts where the first row started' ($first1[0] -eq $first0[0] -and ($first1[1] - $first1[0]) -ge ($first0[1] - $first0[0]) - 1) "first row before '$($first0 -join '-')' after '$($first1 -join '-')' (px from the content top)"
$bar = [OT]::Blocks($r[0] + $r[2] - 9, $r[1], 6, $r[3] - 8, 40, $false)
Check 'the thin scrollbar shows at the right edge' ("$bar" -ne '') "scrollbar ink rows: '$bar'"
$foot = [OT]::Changed($foot0, $r[0], $r[1] + $r[3] - 26, $r[2] - 14, 24, 40)
Check 'the footer stays in place' ($foot -le 4) "changed px in the footer strip: $foot"
WheelOver $w -40
$sEnd = ScrollOf '很多行'
WheelOver $w -3
$sEnd2 = ScrollOf '很多行'
$rowsEnd = TextRows $w
$firstEnd = FirstRow $rowsEnd
Crop 'b1-many-end' ($w.l - 10) ($w.t - 10) ($w.r - $w.l + 20) ($w.b - $w.t + 20) | Out-Null
Check 'the wheel stops at the end, with a whole row at the top' ($sEnd -gt 500 -and $sEnd2 -eq $sEnd -and $firstEnd[0] -eq $first0[0]) "scroll $sEnd then $sEnd2; first row '$($firstEnd -join '-')'"
WheelOver $w 60
Check 'the wheel scrolls back to the start' ((ScrollOf '很多行') -eq 0) "scroll=$(ScrollOf '很多行')"
$log = @(PfLogAll | Where-Object { $_ -match "wheel over a tool fence's text" })
Check 'the log has the wheel steps' ($log.Count -ge 3) "lines=$($log.Count)"
# The thumb: drag it down, the rows follow and rest on a whole row.
WheelOver $w -1
Start-Sleep -Milliseconds 300
$tx = $r[0] + $r[2] - 6
$ty = -1
for ($y = $r[1] + 12; $y -lt $r[1] + $r[3] - 12; $y++) { if ([OT]::Blocks($tx - 3, $y, 6, 1, 40, $false)) { $ty = $y + 6; break } }
if ($ty -gt 0) {
  [G]::MoveTo($tx, $ty); Start-Sleep -Milliseconds 400; MDown; Start-Sleep -Milliseconds 200
  Glide $tx $ty $tx ($ty + 50) 10 30
  Start-Sleep -Milliseconds 200; MUp; Start-Sleep -Milliseconds 900
}
ParkMouse 500 700
$sDrag = ScrollOf '很多行'
$firstDrag = FirstRow (TextRows $w)
Check 'dragging the thumb scrolls, and the rows rest on a whole row' ($ty -gt 0 -and $sDrag -gt $s1 + 100 -and $firstDrag[0] -eq $first0[0]) "thumb at y=$ty; scroll $s1 -> $sDrag; first row '$($firstDrag -join '-')'"

Case 'B2 a note longer than its fence'
$long = (1..14 | ForEach-Object { if ($_ % 3 -eq 0) { "[ ] 第 $_ 件事：给客户回邮件" } else { "第 $_ 行：周会准备的内容" } }) -join "`n"
$nid = NewTool '长便签' '400,60,260,200' 'note' $long
Start-Sleep -Milliseconds 900
$w = DumpOf '长便签'
$rows0 = TextRows $w 0
$first0 = FirstRow $rows0
$boxes0 = @(DumpRects $w 'note_boxes')
WheelOver $w -1
$s1 = ScrollOf '长便签'
$w = DumpOf '长便签'
$first1 = FirstRow (TextRows $w 0)
$boxes1 = @(DumpRects $w 'note_boxes')
Crop 'b2-note-1' ($w.l - 10) ($w.t - 10) ($w.r - $w.l + 20) ($w.b - $w.t + 20) | Out-Null
Check 'one wheel notch scrolls the note, a whole line at the top' ($s1 -gt 20 -and $first1[0] -eq $first0[0]) "scroll $s1; first row before '$($first0 -join '-')' after '$($first1 -join '-')'"
# A to-do box after scrolling: the click ticks the line it shows.
$box = $boxes1 | Where-Object { $_.t -ge 4 } | Select-Object -First 1
if ($box) {
  $p = CPt $w (($box.l + $box.r) / 2) (($box.t + $box.b) / 2)
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 800
}
$t = @((FenceGet $nid).noteText -split "`n")
$ticked = @(0..($t.Count - 1) | Where-Object { $t[$_] -like '`[x`]*' })
Check 'a click on a to-do box of the scrolled note ticks that line' ($box -and $ticked.Count -eq 1 -and $ticked[0] -eq $box.i -and -not (EditorOpen)) "box of line $($box.i) at $($box.l),$($box.t); ticked lines: $($ticked -join ','); boxes before the wheel: $($boxes0.Count), after: $($boxes1.Count)"
Check 'the note keeps its place after the tick' ((ScrollOf '长便签') -eq $s1) "scroll=$(ScrollOf '长便签')"
ParkMouse 500 700
# A note that fits: no scrolling, no scrollbar.
$sid2 = NewTool '短便签' '700,60,260,200' 'note' "第一行`n第二行"
Start-Sleep -Milliseconds 900
$w2 = DumpOf '短便签'
WheelOver $w2 -3
$r2 = CRect $w2
$bar = [OT]::Blocks($r2[0] + $r2[2] - 9, $r2[1], 6, $r2[3] - 8, 40, $false)
Check 'a note that fits does not scroll and shows no scrollbar' ((ScrollOf '短便签') -eq 0 -and "$bar" -eq '') "scroll=$(ScrollOf '短便签') scrollbar ink: '$bar'"
ParkMouse 500 700

Case 'B3 animations off: the wheel moves the text at once'
[Anim]::Set($false)
Start-Sleep -Milliseconds 500
$w = DumpOf '长便签'
$p = CPt $w 120 60
Wheel $p[0] $p[1] 3
Start-Sleep -Milliseconds 700
$sA = ScrollOf '长便签'
Wheel $p[0] $p[1] -1
Start-Sleep -Milliseconds 700
$sB = ScrollOf '长便签'
Check 'with animations off the wheel still scrolls the note' ($sA -eq 0 -and $sB -gt 20) "scroll $sA -> $sB"
[Anim]::Set($true)
ParkMouse 500 700
}

if ($Part -eq 'C') {
function DumpRects($w, $name) {
  $v = DumpVal $w $name
  if (-not $v -or $v -eq '-') { return @() }
  return @($v -split ';' | ForEach-Object {
    $pair = $_ -split ':'; $n = $pair[1] -split ','
    [pscustomobject]@{ i = [int]$pair[0]; l = [double]$n[0]; t = [double]$n[1]; r = [double]$n[2]; b = [double]$n[3] }
  })
}
function NoteLines($id) { return @((FenceGet $id).noteText -split "`n") }
$NoteText = "周会准备`n[ ] 整理上周的数据`n[x] 订会议室`n[ ] 给客户回邮件：确认报价单和交付日期，附上合同扫描件`n`n备注：周五前完成"
function FreshNote() {
  if ($script:Nid) { Pf fence delete $script:Nid | Out-Null; Start-Sleep -Milliseconds 400 }
  $script:Nid = NewTool '便签' '60,60,260,240' 'note' $NoteText
  Start-Sleep -Milliseconds 900
  ParkMouse 600 700
  return (DumpOf '便签')
}
function EndEdit() { if (EditorOpen) { [G]::Press(0x1B); Start-Sleep -Milliseconds 700 } }
# Opens the editor by a click at content DIPs (x, y) and says how much of the note's picture
# changed: pixels that differ from the note before the click (the caret is a few), and whether
# the rows of text are where they were.
function EditLook($w, $x, $y) {
  $r = CRect $w
  $before = [OT]::Snap($r[0], $r[1], $r[2], $r[3])
  $rows0 = [OT]::Blocks($r[0], $r[1], $r[2] - 14, $r[3], 70, $false)
  $p = CPt $w $x $y
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 250
  [G]::MoveTo($r[0] + $r[2] + 60, $r[1] + $r[3] + 60)
  $changed = 0
  foreach ($i in 1..4) { Start-Sleep -Milliseconds 180; $c = [OT]::Changed($before, $r[0], $r[1], $r[2], $r[3], 40); if ($c -gt $changed) { $changed = $c } }
  $rows1 = [OT]::Blocks($r[0], $r[1], $r[2] - 14, $r[3], 70, $false)
  # The caret stands in the last line and is taller than its letters: the lines above it
  # must be where they were, and the last one may only have grown by the caret.
  $a = @($rows0 -split ' '); $b = @($rows1 -split ' ')
  $same = ($a.Count -eq $b.Count -and $a.Count -ge 2)
  for ($i = 0; $same -and $i -lt $a.Count - 1; $i++) { if ($a[$i] -ne $b[$i]) { $same = $false } }
  if ($same) {
    $la = $a[-1] -split '-'; $lb = $b[-1] -split '-'
    if ([int]$lb[0] -gt [int]$la[0] -or [int]$lb[1] -lt [int]$la[1] -or [int]$la[0] - [int]$lb[0] -gt 5 -or [int]$lb[1] - [int]$la[1] -gt 5) { $same = $false }
  }
  return [pscustomobject]@{ open = (EditorOpen); changed = $changed; rows0 = $rows0; rows1 = $rows1; same = $same }
}
[Anim]::Set($true)

Case 'C1 editing changes nothing but the caret'
foreach ($combo in @(@('liquidGlass', 'light'), @('liquidGlass', 'dark'), @('fluent', 'light'), @('fluent', 'dark'))) {
  Pf settings set themeStyle $combo[0] | Out-Null
  Pf settings set theme $combo[1] | Out-Null
  Start-Sleep -Milliseconds 1500
  $w = FreshNote
  Crop "c1-$($combo[0])-$($combo[1])-0" ($w.l - 10) ($w.t - 10) ($w.r - $w.l + 20) ($w.b - $w.t + 20) | Out-Null
  $look = EditLook $w 150 190
  Crop "c1-$($combo[0])-$($combo[1])-1" ($w.l - 10) ($w.t - 10) ($w.r - $w.l + 20) ($w.b - $w.t + 20) | Out-Null
  Check "$($combo -join ' '): the editor opens and only the caret is new (at most 60 px change)" ($look.open -and $look.changed -ge 4 -and $look.changed -le 60 -and $look.same) "open=$($look.open) changed px=$($look.changed) rows before '$($look.rows0)' while editing '$($look.rows1)'"
  EndEdit
}
Pf settings set themeStyle liquidGlass | Out-Null
Pf settings set theme light | Out-Null
Start-Sleep -Milliseconds 1200

Case 'C2 the caret goes where the click was; typing shows at once'
$w = FreshNote
$boxes0 = @(DumpRects $w 'note_boxes')
# Between 「周会」 and 「准备」: two 14 DIP characters after the 16 DIP edge.
$p = CPt $w (16 + 29) 11
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
[G]::Type('X'); Start-Sleep -Milliseconds 500
$r = CRect $w
$shown = [OT]::Blocks($r[0], $r[1], 110, 22, 70, $false)
$wE = DumpOf '便签'
$boxesE = @(DumpRects $wE 'note_boxes')
$same = ($boxes0.Count -eq $boxesE.Count)
for ($i = 0; $i -lt $boxes0.Count -and $same; $i++) { if ($boxes0[$i].l -ne $boxesE[$i].l -or $boxes0[$i].t -ne $boxesE[$i].t) { $same = $false } }
Check 'the to-do boxes keep their places while the note is edited' ($same -and $boxes0.Count -eq 3 -and (EditorOpen)) "boxes before: $(DumpVal $w 'note_boxes'); editing: $(DumpVal $wE 'note_boxes')"
[G]::Press(0x1B); Start-Sleep -Milliseconds 800
$l = NoteLines $Nid
Check 'the typed letter went in where the click was' ($l[0] -eq '周会X准备') "first line: '$($l[0])'"

Case 'C3 a to-do typed while editing shows its box at once; Backspace after a box removes it'
$w = FreshNote
$p = CPt $w 200 190
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
[G]::Press(0x23); [G]::Press(0x0D); [G]::Type('[ ] 新的待办'); Start-Sleep -Milliseconds 700
$wE = DumpOf '便签'
Check 'a line typed as 「[ ] …」 shows a check box while still editing' ((EditorOpen) -and @(DumpRects $wE 'note_boxes').Count -eq 4) "editing=$(EditorOpen) boxes: $(DumpVal $wE 'note_boxes')"
# Once the text is deleted the caret stands right after the new box: one more Backspace
# takes the box and leaves the line.
foreach ($i in 1..4) { [G]::Press(0x08) }; Start-Sleep -Milliseconds 300
[G]::Press(0x08); Start-Sleep -Milliseconds 600
$wE = DumpOf '便签'
Check 'Backspace right after a check box removes the box' ((EditorOpen) -and @(DumpRects $wE 'note_boxes').Count -eq 3) "boxes: $(DumpVal $wE 'note_boxes')"
[G]::Type('普通一行'); [G]::Press(0x1B); Start-Sleep -Milliseconds 800
$l = NoteLines $Nid
Check 'the line is saved as plain text' ($l[-1] -eq '普通一行') "last line: '$($l[-1])' of $($l.Count)"

Case 'C4 keys and the mouse in the edited note'
$w = FreshNote
# Up from the last line goes through the empty line to the second row of the wrapped to-do.
$p = CPt $w 17 190
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
[G]::Press(0x24); [G]::Press(0x26); [G]::Press(0x26); [G]::Type('Q'); Start-Sleep -Milliseconds 400
[G]::Press(0x1B); Start-Sleep -Milliseconds 800
$l = NoteLines $Nid
$q = $l[3].IndexOf('Q')
Check 'Up moves by the lines on screen (into the middle of a wrapped line)' ($q -gt 8 -and $q -lt $l[3].Length - 1) "line 4: '$($l[3])' (Q at $q)"
# Home, End and Left on a to-do: the caret keeps clear of the marker that is not shown.
$w = FreshNote
$p = CPt $w 120 37
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
[G]::Press(0x24); [G]::Type('H'); [G]::Press(0x23); [G]::Type('E'); Start-Sleep -Milliseconds 300
[G]::Press(0x24); [G]::Press(0x25); [G]::Type('L'); Start-Sleep -Milliseconds 300
Chord 0x11 0x23; [G]::Press(0x0D); [G]::Type('[ ] 末'); Chord 0x11 0x24; [G]::Type('T'); Start-Sleep -Milliseconds 300
[G]::Press(0x1B); Start-Sleep -Milliseconds 800
$l = NoteLines $Nid
Check 'Home and End on a to-do go to the start and the end of its text; Left from the start goes to the line before' ($l[1] -eq '[ ] H整理上周的数据E' -and $l[0] -eq 'T周会准备L') "lines 1 and 2: '$($l[0])' / '$($l[1])'"
$w = FreshNote
Pf fence set $Nid noteText "[ ] 第一件事`n第二行" | Out-Null; Start-Sleep -Milliseconds 700
$p = CPt $w 150 37
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
Chord 0x11 0x24; [G]::Type('C'); Start-Sleep -Milliseconds 300
[G]::Press(0x1B); Start-Sleep -Milliseconds 800
$l = NoteLines $Nid
Check 'Ctrl+Home in a note that starts with a to-do types into the to-do''s text' ($l[0] -eq '[ ] C第一件事') "first line: '$($l[0])'"
# Ctrl+A and typing replace everything; Ctrl+Z brings it back.
$w = FreshNote
$p = CPt $w 150 190
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
Chord 0x11 0x41; Start-Sleep -Milliseconds 300
$r = CRect $w
Crop 'c4-select-all' ($w.l - 10) ($w.t - 10) ($w.r - $w.l + 20) ($w.b - $w.t + 20) | Out-Null
[G]::Type('全新'); Start-Sleep -Milliseconds 300
[G]::Press(0x1B); Start-Sleep -Milliseconds 800
Check 'Ctrl+A then typing replaces the text' ((FenceGet $Nid).noteText -eq '全新') "note: '$((FenceGet $Nid).noteText -replace "`n", ' / ')'"
# A drag selects; typing replaces the selection.
$w = FreshNote
$p = CPt $w 150 190
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
$a = CPt $w 17 11; $b = CPt $w (16 + 29) 11
[G]::MoveTo($a[0], $a[1]); Start-Sleep -Milliseconds 200; MDown; Start-Sleep -Milliseconds 150
Glide $a[0] $a[1] $b[0] $b[1] 6 30
Start-Sleep -Milliseconds 150; MUp; Start-Sleep -Milliseconds 400
Crop 'c4-drag-select' ($w.l - 10) ($w.t - 10) ($w.r - $w.l + 20) ($w.b - $w.t + 20) | Out-Null
[G]::Type('例'); Start-Sleep -Milliseconds 300
[G]::Press(0x1B); Start-Sleep -Milliseconds 800
$l = NoteLines $Nid
Check 'a drag over 「周会」 selects it and typing replaces it' ($l[0] -eq '例准备') "first line: '$($l[0])'"
# A right-click in the edited text opens the text menu, and the editor stays.
$w = FreshNote
$p = CPt $w 150 190
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
[G]::RClick($p[0], $p[1]); Start-Sleep -Milliseconds 900
$m = MenuWin 3
Check 'a right-click in the edited note opens the text menu (paste, select all)' ($m -ne [IntPtr]::Zero -and (EditorOpen)) "menu=$m editing=$(EditorOpen) rows=$(@(MenuRows $m).Count)"
CloseMenus
EndEdit
# A click on the title still closes the editor.
$w = FreshNote
$p = CPt $w 150 190
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
[G]::Type(' 末尾'); Start-Sleep -Milliseconds 300
[G]::Click($w.l + 120, $w.t + 14); Start-Sleep -Milliseconds 800
Check 'a click on the note''s own title saves and closes the editor' (-not (EditorOpen) -and (NoteLines $Nid)[-1] -eq '备注：周五前完成 末尾') "editing=$(EditorOpen) last line '$((NoteLines $Nid)[-1])'"

Case 'C5 a long note: the caret is kept in view, the scrollbar only when the text overflows'
$long = (1..14 | ForEach-Object { "第 $_ 行：周会准备的内容" }) -join "`n"
$lid = NewTool '长便签' '400,60,260,200' 'note' $long
Start-Sleep -Milliseconds 900
$wl = DumpOf '长便签'
$m = TitleMenuOf $wl
MenuPick '^编辑便签' $m | Out-Null
Start-Sleep -Milliseconds 900
$wE = DumpOf '长便签'
$sc = [double](DumpVal $wE 'scroll')
Check '「编辑便签」 puts the caret after the last line and scrolls there' ((EditorOpen) -and $sc -gt 100) "editing=$(EditorOpen) scroll=$sc"
[G]::Type('最后'); Chord 0x11 0x24; Start-Sleep -Milliseconds 700
$wE = DumpOf '长便签'
Check 'Ctrl+Home scrolls back to the first line' ((EditorOpen) -and [double](DumpVal $wE 'scroll') -eq 0) "scroll=$(DumpVal $wE 'scroll')"
[G]::Press(0x1B); Start-Sleep -Milliseconds 800
Check 'the long note saved what was typed at its end' ((NoteLines $lid)[-1] -eq '第 14 行：周会准备的内容最后') "last line '$((NoteLines $lid)[-1])'"
$w = DumpOf '便签'
$r = CRect $w
$p = CPt $w 150 190
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900
[G]::MoveTo($r[0] + $r[2] + 60, $r[1] + $r[3] + 60); Start-Sleep -Milliseconds 300
$bar = [OT]::Blocks($r[0] + $r[2] - 9, $r[1], 6, $r[3] - 8, 40, $false)
Check 'a note that fits shows no scrollbar while it is edited' ((EditorOpen) -and "$bar" -eq '') "scrollbar ink: '$bar'"
EndEdit
ParkMouse 600 700
}

if ($Part -eq 'I') {
# Microsoft Pinyin in the note editor: the letters being composed show at the caret, the
# candidate window sits under it, and the chosen word goes into the note. The pf user's
# language list gets zh-Hans-CN for the run and loses it again at the end.
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class OIme {
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, IntPtr pid);
  [DllImport("user32.dll")] static extern IntPtr GetKeyboardLayout(uint thread);
  public static string Layout() { var t = GetWindowThreadProcessId(GetForegroundWindow(), IntPtr.Zero); return GetKeyboardLayout(t).ToInt64().ToString("X"); }
}
"@
Case 'I1 Chinese input in the note editor'
$langs = Get-WinUserLanguageList
$AddedZh = -not ($langs | Where-Object { $_.LanguageTag -like 'zh-Hans*' })
if ($AddedZh) { $langs.Add('zh-Hans-CN'); Set-WinUserLanguageList $langs -Force }
Start-Sleep -Seconds 2
$nid = NewTool '便签' '60,60,300,240' 'note' "周会准备`n[ ] 整理上周的数据`n`n备注："
Start-Sleep -Milliseconds 900
$w = DumpOf '便签'
$r = CRect $w
$p = CPt $w 200 93
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900
[G]::MoveTo($r[0] + $r[2] + 80, $r[1] + $r[3] + 80)
for ($i = 0; $i -lt 2 -and [OIme]::Layout() -notmatch '804$'; $i++) {
  [G]::KeyDown(0xA4); [G]::KeyDown(0xA0); Start-Sleep -Milliseconds 60; [G]::KeyUp(0xA0); [G]::KeyUp(0xA4); Start-Sleep -Milliseconds 800
}
Check 'the note editor is open with the Chinese keyboard' ((EditorOpen) -and [OIme]::Layout() -match '804$') "editing=$(EditorOpen) layout=$([OIme]::Layout()) added=$AddedZh"
$before = [OT]::Snap($r[0], $r[1] + 70, $r[2], 40)
foreach ($c in 'BAOGAO'.ToCharArray()) { [G]::Press([uint16][int]$c); Start-Sleep -Milliseconds 150 }
Start-Sleep -Milliseconds 700
$composing = [OT]::Changed($before, $r[0], $r[1] + 70, $r[2], 40, 40)
Crop 'i1-composing' ($w.l - 10) ($w.t - 10) 520 420 | Out-Null
Check 'the letters being composed show in the note''s line' ($composing -ge 60) "changed px in the caret's line while composing: $composing"
[G]::Press(0x20); Start-Sleep -Milliseconds 700
Crop 'i1-committed' ($w.l - 10) ($w.t - 10) 520 420 | Out-Null
[G]::Press(0x1B); Start-Sleep -Milliseconds 900
$l = @((FenceGet $nid).noteText -split "`n")
Check 'the chosen word is in the note after Esc' ($l[3] -match '^备注：\p{IsCJKUnifiedIdeographs}{2}$') "last line: '$($l[3])'"
if ($AddedZh) {
  $langs = Get-WinUserLanguageList
  foreach ($x in @($langs | Where-Object { $_.LanguageTag -like 'zh-Hans*' })) { [void]$langs.Remove($x) }
  Set-WinUserLanguageList $langs -Force
}
Check 'pf''s languages are as before' (-not $AddedZh -or -not (Get-WinUserLanguageList | Where-Object { $_.LanguageTag -like 'zh-Hans*' })) ''
}

if ($Part -eq 'R') {
# The recordings of the before/after animations: the same steps on any build.
Case 'R recordings'
[Anim]::Set($true)
$nid = NewTool '便签' '60,60,260,240' 'note' "周会准备`n[ ] 整理上周的数据`n[x] 订会议室`n[ ] 给客户回邮件：确认报价单和交付日期，附上合同扫描件`n`n备注：周五前完成"
$cid = NewTool '时钟' '350,60,240,140' 'clock' ''
$mid = NewTool '很多行' '60,340,300,230' 'script' "$sd\很多行.ps1"
WaitRun $mid | Out-Null
Start-Sleep -Seconds 2
$wn = DumpOf '便签'; $wc = DumpOf '时钟'; $wm = DumpOf '很多行'
DeskClick 900 700 | Out-Null; Start-Sleep -Milliseconds 500
# The note editor.
$p = CPt $wn 150 190
[G]::MoveTo($p[0] + 60, $p[1] + 30); Start-Sleep -Milliseconds 300
RecStart 'note-edit' -X 40 -Y 40 -W 300 -H 280 -Fps 20
Start-Sleep -Milliseconds 800
RecMark '单击便签里的文字'
[G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 300
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 1500
RecMark '打字'
foreach ($c in ' 加一句'.ToCharArray()) { [G]::Type("$c"); Start-Sleep -Milliseconds 220 }
Start-Sleep -Milliseconds 900
RecMark '按 Esc 保存'
[G]::Press(0x1B); Start-Sleep -Milliseconds 1500
RecStop
DeskClick 900 700 | Out-Null; Start-Sleep -Milliseconds 500
# The wheel over a script fence with more rows than it holds.
$p = CPt $wm 150 80
[G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 300
RecStart 'scroll' -X 40 -Y 320 -W 340 -H 270 -Fps 20
Start-Sleep -Milliseconds 800
RecMark '滚轮向下滚三格'
foreach ($i in 1..3) { Wheel $p[0] $p[1] -1; Start-Sleep -Milliseconds 600 }
Start-Sleep -Milliseconds 800
RecMark '滚轮向上滚回去'
foreach ($i in 1..3) { Wheel $p[0] $p[1] 1; Start-Sleep -Milliseconds 600 }
Start-Sleep -Milliseconds 800
RecStop
DeskClick 900 700 | Out-Null; Start-Sleep -Milliseconds 500
# A click on the clock while the note is edited.
$p = CPt $wn 150 190
RecStart 'click-clock' -X 40 -Y 40 -W 570 -H 280 -Fps 20
Start-Sleep -Milliseconds 600
RecMark '单击便签，打几个字'
[G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 300
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900
foreach ($c in ' 再加'.ToCharArray()) { [G]::Type("$c"); Start-Sleep -Milliseconds 220 }
Start-Sleep -Milliseconds 600
RecMark '单击旁边的时钟'
$q = CPt $wc 120 50
[G]::MoveTo($q[0], $q[1]); Start-Sleep -Milliseconds 400
[G]::Click($q[0], $q[1]); Start-Sleep -Milliseconds 1200
RecMark '接着打字'
foreach ($c in 'abc'.ToCharArray()) { [G]::Type("$c"); Start-Sleep -Milliseconds 250 }
Start-Sleep -Milliseconds 1200
RecStop
$open = EditorOpen
Log "after the clock click: editor open=$open fg=$(Fg) note=$(((FenceGet $nid).noteText -split "`n")[-1])"
Check 'the three recordings were made' ((Test-Path "$PfShots\rec-note-edit\frames.tsv") -and (Test-Path "$PfShots\rec-scroll\frames.tsv") -and (Test-Path "$PfShots\rec-click-clock\frames.tsv")) "editor open after the clock click=$open"
if (EditorOpen) { [G]::Press(0x1B); Start-Sleep -Milliseconds 600 }
}

Case 'cleanup'
ClearFences
[Anim]::Set($Anim0)
