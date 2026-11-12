# Owner round 2026-11-12, fourth batch: a to-do of a note is moved by dragging it, zh-CN.
# Parts (run.sh --var Part=...; D, C and K without it):
#   D  the drag: the third to-do above the first, by the check box, ended with Esc, released
#      outside the fence, dropped under a plain line, in a locked fence, taken back with Ctrl+Z;
#      the saved text after each
#   C  a press without a move is the click it was: on a to-do the editor opens on the release (on
#      other text on the press, as before) with the caret
#      at the press, a check box ticks, a web address opens, a double-click takes a word, a drag
#      inside the open editor selects
#   K  Alt+Shift+Down and Alt+Shift+Up move the line with the caret while the note is edited;
#      Ctrl+Z takes a move back in one step
#   R  the drag recorded (--var Rec=before|after for the clip; after: light and dark mode too)
trap { PfTrap $_; continue }
if (-not $Part) { $Part = 'DCK' }
Case 'setup'
CloseWindows
Set-Culture zh-CN
$Anim0 = [Anim]::Get()
[Anim]::Set($true)
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class T4 {
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr FindWindowW(string cls, string title);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr FindWindowExW(IntPtr p, IntPtr a, string c, string t);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr SendMessageW(IntPtr h, uint m, IntPtr w, StringBuilder l);
  [StructLayout(LayoutKind.Sequential)] struct CI { public int cb; public int flags; public IntPtr h; public int x; public int y; }
  [DllImport("user32.dll")] static extern bool GetCursorInfo(ref CI ci);
  [DllImport("user32.dll")] static extern IntPtr LoadCursorW(IntPtr inst, IntPtr id);
  // The text the open note editor holds (line breaks as \n); null when none is open.
  public static string EdText() {
    var p = FindWindowW("PecoFence.NoteEdit", null); if (p == IntPtr.Zero) return null;
    var e = FindWindowExW(p, IntPtr.Zero, "Edit", null); if (e == IntPtr.Zero) return null;
    var sb = new StringBuilder(8192); SendMessageW(e, 0x0D, (IntPtr)8192, sb);
    return sb.ToString().Replace("\r\n", "\n");
  }
  // The pointer's shape now: arrow, ibeam, hand, or its handle.
  public static string Cursor() {
    var ci = new CI(); ci.cb = Marshal.SizeOf(typeof(CI)); GetCursorInfo(ref ci);
    string[] names = { "arrow", "ibeam", "hand" }; int[] ids = { 32512, 32513, 32649 };
    for (int i = 0; i < ids.Length; i++) if (LoadCursorW(IntPtr.Zero, (IntPtr)ids[i]) == ci.h) return names[i];
    return "0x" + ci.h.ToString("x");
  }
}
'@
GreyWallpaper
PfStart -Lang 'zh-CN' -Script (DumpScript 500 25 3000)
Start-Sleep -Seconds 3
ClearPfNotices
ClearFences
Pf settings set featureTips false | Out-Null
Pf settings set theme light | Out-Null
# The content rect of a dumped window in screen px, and the rects a dump field lists
# ("index:l,t,r,b;..." in content DIPs) by index.
function CRect($w) { $cx = [int](DumpVal $w 'content_x'); $cy = [int](DumpVal $w 'content_y'); return @(($w.l + $cx), ($w.t + $cy), ($w.r - $w.l - $cx), ($w.b - $w.t - $cy)) }
function Rects($w, $name) {
  $o = @{}; $v = DumpVal $w $name
  if ($v -and $v -ne '-') { foreach ($p in ($v -split ';')) { $kv = $p -split ':'; if (-not $o.ContainsKey([int]$kv[0])) { $o[[int]$kv[0]] = @($kv[1] -split ',' | ForEach-Object { [double]$_ }) } } }
  return $o
}
function FCrop($name, $w, $m = 12) { Crop $name ([math]::Max(0, $w.l - $m)) ([math]::Max(0, $w.t - $m)) ($w.r - $w.l + 2 * $m) ($w.b - $w.t + 2 * $m) | Out-Null }
function EditorOpen() { $h = [G]::FindWindowW('PecoFence.NoteEdit', [NullString]::Value); return ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) }
function NoteText($id) { return "$((FenceGet $id).noteText)" }
function OneLine($t) { return ("$t" -replace "`n", ' / ') }
function Lifts() { return @(PfLogAll | Where-Object { $_ -match 'to-do drag starts' }).Count }
function AltShift($vk) { [G]::KeyDown(0x12); [G]::KeyDown(0x10); Start-Sleep -Milliseconds 40; [G]::Press($vk); Start-Sleep -Milliseconds 40; [G]::KeyUp(0x10); [G]::KeyUp(0x12) }
# A point on the text of the to-do on line $line (40 px after its check box) and on its box.
function TodoPt($c, $boxes, $line) { $b = $boxes[$line]; return @(($c[0] + [int]$b[2] + 40), ($c[1] + [int](($b[1] + $b[3]) / 2))) }
function BoxPt($c, $boxes, $line) { $b = $boxes[$line]; return @(($c[0] + [int](($b[0] + $b[2]) / 2)), ($c[1] + [int](($b[1] + $b[3]) / 2))) }
# The note of the drag parts: a plain line, two short to-dos, a to-do that wraps, a plain line.
$Long = '把上周的周报和三张报销单一起发给财务，抄送组长'
$T0 = "周会后要做`n[ ] 买菜`n[x] 回邮件`n[ ] $Long`n备注：周五前"
# The note gets $text again (no editor open, the pointer away); returns its dump.
function Fresh($id, $text) {
  if (EditorOpen) { [G]::Press(0x1B); Start-Sleep -Milliseconds 500 }
  Pf fence set $id noteText $text --string | Out-Null
  ParkMouse 900 700
  Start-Sleep -Milliseconds 900
  return (DumpOf '便签')
}

if ($Part -match 'D') {
Case 'D1 the third to-do dragged above the first'
$nid = "$((PfJson fence create --tool note --title 便签 --text $T0 --rect '100,100,300,240').fence.id)"
Start-Sleep -Seconds 1
$w = Fresh $nid $T0; $c = CRect $w; $boxes = Rects $w 'note_boxes'
Check 'the note shows three check boxes, on lines 1, 2 and 3' ($boxes[1] -and $boxes[2] -and $boxes[3] -and -not $boxes[0]) "note_boxes=$(DumpVal $w 'note_boxes')"
$from = TodoPt $c $boxes 3; $to = TodoPt $c $boxes 1
$lift0 = Lifts
[G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 400
$cursorRest = [T4]::Cursor()
MDown; Start-Sleep -Milliseconds 300
$edHeld = EditorOpen
Glide $from[0] $from[1] ($from[0] + 6) ($to[1] - 4) 14 30
Start-Sleep -Milliseconds 500
$edDrag = EditorOpen
$cursorDrag = [T4]::Cursor()
$lift1 = Lifts
FCrop 'd1-dragging' $w
$mid = NoteText $nid
MUp; Start-Sleep -Milliseconds 900
$after = NoteText $nid
FCrop 'd1-dropped' $w
$want = "周会后要做`n[ ] $Long`n[ ] 买菜`n[x] 回邮件`n备注：周五前"
Check 'a press held on a to-do''s text does not open the editor' (-not $edHeld) "editor open while the button is held: $edHeld"
Check 'moved past the drag threshold, the to-do is lifted: no editor, the arrow for a pointer' ($lift1 -eq $lift0 + 1 -and -not $edDrag -and $cursorDrag -eq 'arrow') "lifted (log): $lift0 -> $lift1; editor=$edDrag; pointer before the press: $cursorRest, while dragging: $cursorDrag"
Check 'the note''s text changes on the release, not while the to-do is held' ($mid -eq $T0) "while held: $(OneLine $mid)"
Check 'the third to-do, a line that wraps, lands whole above the first; nothing else changes places' ($after -eq $want) "note: $(OneLine $after)"
Check 'the drop opens no editor' (-not (EditorOpen)) "editor=$(EditorOpen)"

Case 'D2 a drag that starts on the check box moves the line and ticks nothing'
$w = Fresh $nid $T0; $c = CRect $w; $boxes = Rects $w 'note_boxes'
$from = BoxPt $c $boxes 1; $to = BoxPt $c $boxes 2
[G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 300
MDown; Start-Sleep -Milliseconds 200
Glide $from[0] $from[1] ($from[0] + 3) ($to[1] + 6) 10 30
Start-Sleep -Milliseconds 400
MUp; Start-Sleep -Milliseconds 900
$after = NoteText $nid
$want = "周会后要做`n[x] 回邮件`n[ ] 买菜`n[ ] $Long`n备注：周五前"
Check 'dragged by its check box one line down, the to-do changes places with the next and stays unticked' ($after -eq $want) "note: $(OneLine $after)"

Case 'D3 Esc during the drag puts the line back'
$w = Fresh $nid $T0; $c = CRect $w; $boxes = Rects $w 'note_boxes'
$from = TodoPt $c $boxes 3; $to = TodoPt $c $boxes 1
$lift0 = Lifts
[G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 300
MDown; Start-Sleep -Milliseconds 200
Glide $from[0] $from[1] ($from[0] + 6) ($to[1] - 4) 14 30
Start-Sleep -Milliseconds 400
[G]::Press(0x1B); Start-Sleep -Milliseconds 600
FCrop 'd3-after-esc' $w
Glide ($from[0] + 6) ($to[1] - 4) ($from[0] + 6) ($to[1] + 10) 4 30
MUp; Start-Sleep -Milliseconds 900
$after = NoteText $nid
$lift1 = Lifts
$back = @(PfLogAll | Where-Object { $_ -match 'to-do drag ends' -and $_ -match 'commit=false' }).Count
Check 'the to-do was lifted, Esc put it back, and the release after it changes nothing' ($lift1 -eq $lift0 + 1 -and $back -ge 1 -and $after -eq $T0 -and -not (EditorOpen)) "lifted (log): $lift0 -> $lift1; taken back (log): $back; note: $(OneLine $after); editor=$(EditorOpen)"

Case 'D4 released outside the fence, the line goes back'
$w = Fresh $nid $T0; $c = CRect $w; $boxes = Rects $w 'note_boxes'
$from = TodoPt $c $boxes 3
$lift0 = Lifts
[G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 300
MDown; Start-Sleep -Milliseconds 200
Glide $from[0] $from[1] ($from[0] + 6) ($from[1] - 40) 8 30
Glide ($from[0] + 6) ($from[1] - 40) ($w.r + 120) ($from[1] - 40) 12 30
Start-Sleep -Milliseconds 500
FCrop 'd4-pointer-outside' $w 140
MUp; Start-Sleep -Milliseconds 900
$after = NoteText $nid
$lift1 = Lifts
Check 'a to-do dragged out of the fence and released there stays where it was' ($lift1 -eq $lift0 + 1 -and $after -eq $T0 -and -not (EditorOpen)) "lifted (log): $lift0 -> $lift1; note: $(OneLine $after); editor=$(EditorOpen)"

Case 'D5 a to-do dropped under a plain line'
$w = Fresh $nid $T0; $c = CRect $w; $boxes = Rects $w 'note_boxes'
$from = TodoPt $c $boxes 1
[G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 300
MDown; Start-Sleep -Milliseconds 200
Glide $from[0] $from[1] ($from[0] + 6) ($w.b - 8) 16 30
Start-Sleep -Milliseconds 500
FCrop 'd5-at-the-end' $w
MUp; Start-Sleep -Milliseconds 900
$after = NoteText $nid
$want = "周会后要做`n[x] 回邮件`n[ ] $Long`n备注：周五前`n[ ] 买菜"
Check 'dragged to the bottom, the to-do lands after the last plain line; the plain lines keep their order' ($after -eq $want) "note: $(OneLine $after)"

Case 'D6 a locked fence still lets its to-dos be moved'
$w = Fresh $nid $T0; $c = CRect $w; $boxes = Rects $w 'note_boxes'
Pf fence set $nid locked true | Out-Null
Start-Sleep -Milliseconds 600
$from = TodoPt $c $boxes 2; $to = TodoPt $c $boxes 1
[G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 300
MDown; Start-Sleep -Milliseconds 200
Glide $from[0] $from[1] ($from[0] + 6) ($to[1] - 6) 10 30
Start-Sleep -Milliseconds 400
MUp; Start-Sleep -Milliseconds 900
$after = NoteText $nid
$locked = "$((FenceGet $nid).locked)"
$want = "周会后要做`n[x] 回邮件`n[ ] 买菜`n[ ] $Long`n备注：周五前"
Check 'in a fence locked for position and size the second to-do is dragged above the first' ($after -eq $want -and $locked -eq 'True') "locked=$locked; note: $(OneLine $after)"
Pf fence set $nid locked false | Out-Null

Case 'D7 Ctrl+Z after a drop takes the move back'
$w = Fresh $nid $T0; $c = CRect $w; $boxes = Rects $w 'note_boxes'
$from = TodoPt $c $boxes 3; $to = TodoPt $c $boxes 1
[G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 300
MDown; Start-Sleep -Milliseconds 200
Glide $from[0] $from[1] ($from[0] + 6) ($to[1] - 4) 14 30
Start-Sleep -Milliseconds 400
MUp; Start-Sleep -Milliseconds 900
$dropped = NoteText $nid
Chord 0x11 0x5A; Start-Sleep -Milliseconds 800
$undone = NoteText $nid
$want = "周会后要做`n[ ] $Long`n[ ] 买菜`n[x] 回邮件`n备注：周五前"
Check 'Ctrl+Z right after the drop puts the to-do back where it was' ($dropped -eq $want -and $undone -eq $T0) "after the drop: $(OneLine $dropped); after Ctrl+Z: $(OneLine $undone)"
# A drop, then a tick: the note no longer reads as the drop left it, and Ctrl+Z leaves it alone.
$w = Fresh $nid $T0; $c = CRect $w; $boxes = Rects $w 'note_boxes'
$from = TodoPt $c $boxes 2; $to = TodoPt $c $boxes 1
[G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 300
MDown; Start-Sleep -Milliseconds 200
Glide $from[0] $from[1] ($from[0] + 6) ($to[1] - 6) 10 30
Start-Sleep -Milliseconds 400
MUp; Start-Sleep -Milliseconds 900
$b = BoxPt $c $boxes 2
[G]::Click($b[0], $b[1]); Start-Sleep -Milliseconds 700
$ticked = NoteText $nid
Chord 0x11 0x5A; Start-Sleep -Milliseconds 800
$kept = NoteText $nid
$want = "周会后要做`n[x] 回邮件`n[x] 买菜`n[ ] $Long`n备注：周五前"
Check 'after a tick that followed the drop, Ctrl+Z changes nothing in the note' ($ticked -eq $want -and $kept -eq $want) "after the drop and the tick: $(OneLine $ticked); after Ctrl+Z: $(OneLine $kept)"
Pf fence delete $nid | Out-Null
}

if ($Part -match 'C') {
Case 'C1 the editor opens on the release, the caret at the press'
$T1 = "[ ] 西红柿`n[ ] 鸡蛋 牛奶 面包`n周报 https://example.com/wiki 详见"
$nid = "$((PfJson fence create --tool note --title 便签 --text $T1 --rect '100,100,380,220').fence.id)"
Start-Sleep -Seconds 1
$w = Fresh $nid $T1; $c = CRect $w; $boxes = Rects $w 'note_boxes'
$p = TodoPt $c $boxes 0
[G]::MoveTo($c[0] + 330, $p[1]); Start-Sleep -Milliseconds 300
MDown; Start-Sleep -Milliseconds 500
$edHeld = EditorOpen
MUp; Start-Sleep -Milliseconds 700
$edUp = EditorOpen
[G]::Type('大'); Start-Sleep -Milliseconds 300
$typed = [T4]::EdText()
[G]::Press(0x1B); Start-Sleep -Milliseconds 700
Check 'on a to-do the editor opens when the button is released, not while it is held' (-not $edHeld -and $edUp) "editor while held: $edHeld; after the release: $edUp"
Check 'the caret is where the press was: a click after the first to-do''s text, and the character typed ends that line' ($typed -eq "[ ] 西红柿大`n[ ] 鸡蛋 牛奶 面包`n周报 https://example.com/wiki 详见") "editor: $(OneLine $typed)"
# The third line is plain text: nothing can be dragged there, and the editor opens on the press.
$w = Fresh $nid $T1; $c = CRect $w; $boxes = Rects $w 'note_boxes'
$y2 = $c[1] + [int]((($boxes[1][1] + $boxes[1][3]) / 2) + ($boxes[1][1] - $boxes[0][1]))
[G]::MoveTo($c[0] + 24, $y2); Start-Sleep -Milliseconds 300
MDown; Start-Sleep -Milliseconds 400
$edPlain = EditorOpen
MUp; Start-Sleep -Milliseconds 400
[G]::Type('好'); Start-Sleep -Milliseconds 300
$typed2 = [T4]::EdText()
[G]::Press(0x1B); Start-Sleep -Milliseconds 700
Check 'on a line that is no to-do the editor opens on the press, as it did' ($edPlain -and (@($typed2 -split "`n")[2] -match '好')) "editor while held: $edPlain; editor: $(OneLine $typed2)"

Case 'C2 a click on a check box ticks it, a click on a web address opens it'
$w = Fresh $nid $T1; $c = CRect $w; $boxes = Rects $w 'note_boxes'
$b = BoxPt $c $boxes 1
[G]::Click($b[0], $b[1]); Start-Sleep -Milliseconds 700
$ticked = NoteText $nid
Check 'a click on the second check box ticks it and opens no editor' ($ticked -eq "[ ] 西红柿`n[x] 鸡蛋 牛奶 面包`n周报 https://example.com/wiki 详见" -and -not (EditorOpen)) "note: $(OneLine $ticked); editor=$(EditorOpen)"
$w = DumpOf '便签'
$links = Rects $w 'note_links'
$l0 = $links[2]
if (-not $l0) { $l0 = @(47.8, 51.0, 204.3, 70.0) }
$lx = $c[0] + [int](($l0[0] + $l0[2]) / 2); $ly = $c[1] + [int](($l0[1] + $l0[3]) / 2)
$n0 = @(PfLogAll | Where-Object { $_ -match 'note link opened in the browser' }).Count
[G]::Click($lx, $ly); Start-Sleep -Seconds 3
$n1 = @(PfLogAll | Where-Object { $_ -match 'note link opened in the browser' }).Count
$ed = EditorOpen
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1
CloseWindows
Check 'a click on the web address opens it and no editor' ($n1 -gt $n0 -and -not $ed) "log lines: $n0 -> $n1; editor=$ed"

Case 'C3 a double-click takes a word, a drag inside the open editor selects'
$w = Fresh $nid $T1; $c = CRect $w; $boxes = Rects $w 'note_boxes'
$p = TodoPt $c $boxes 1
DoubleClick ($p[0] + 2) $p[1]; Start-Sleep -Milliseconds 700
$edDbl = EditorOpen
[G]::Type('奶酪'); Start-Sleep -Milliseconds 300
$dbl = [T4]::EdText()
# A drag over the first to-do's text, in the editor that is open now: it selects.
$q = TodoPt $c $boxes 0
$lift0 = Lifts
[G]::MoveTo($q[0] - 30, $q[1]); Start-Sleep -Milliseconds 200
MDown; Start-Sleep -Milliseconds 150
Glide ($q[0] - 30) $q[1] ($q[0] + 60) $q[1] 8 30
MUp; Start-Sleep -Milliseconds 400
[G]::Type('土豆'); Start-Sleep -Milliseconds 300
$sel = [T4]::EdText()
[G]::Press(0x1B); Start-Sleep -Milliseconds 700
Check 'a double-click on a word of a note at rest opens the editor with the word selected: typing replaces it' ($edDbl -and $dbl -match '^\[ \] 西红柿\n\[ \] 鸡蛋 奶酪 面包\n' ) "editor=$edDbl; editor: $(OneLine $dbl)"
Check 'a drag over the text of the open editor selects it (typing replaces it) and lifts no to-do' ($sel -match '^\[ \] 土豆\n' -and $lift0 -eq (Lifts)) "editor: $(OneLine $sel)"
Pf fence delete $nid | Out-Null
}

if ($Part -match 'K') {
Case 'K Alt+Shift+Down and Alt+Shift+Up while the note is edited'
$T2 = "[ ] 一号`n[x] 二号`n说明`n[ ] 三号"
$nid = "$((PfJson fence create --tool note --title 便签 --text $T2 --rect '100,100,300,220').fence.id)"
Start-Sleep -Seconds 1
$w = Fresh $nid $T2; $c = CRect $w; $boxes = Rects $w 'note_boxes'
$p = TodoPt $c $boxes 0
[G]::Click($p[0] - 20, $p[1]); Start-Sleep -Milliseconds 700
$open = EditorOpen
AltShift 0x28; Start-Sleep -Milliseconds 400
$k1 = [T4]::EdText()
FCrop 'k-after-down' $w
[G]::Type('甲'); Start-Sleep -Milliseconds 300
$k2 = [T4]::EdText()
Chord 0x11 0x5A; Start-Sleep -Milliseconds 300
Chord 0x11 0x5A; Start-Sleep -Milliseconds 400
$k3 = [T4]::EdText()
Chord 0x11 0x59; Start-Sleep -Milliseconds 400
$k4 = [T4]::EdText()
AltShift 0x28; Start-Sleep -Milliseconds 300
AltShift 0x28; Start-Sleep -Milliseconds 300
AltShift 0x28; Start-Sleep -Milliseconds 400
$k5 = [T4]::EdText()
AltShift 0x26; Start-Sleep -Milliseconds 400
$k6 = [T4]::EdText()
[G]::Press(0x1B); Start-Sleep -Milliseconds 700
$saved = NoteText $nid
Check 'Alt+Shift+Down moves the to-do with the caret one line down' ($open -and $k1 -eq "[x] 二号`n[ ] 一号`n说明`n[ ] 三号") "editor open: $open; editor: $(OneLine $k1)"
Check 'the caret moved with its line: the next character goes into the moved to-do' ($k2 -match '^\[x\] 二号\n\[ \] 一甲?号甲?\n' -and $k2 -match '甲') "editor: $(OneLine $k2)"
Check 'Ctrl+Z twice takes back the character and then the move, in one step' ($k3 -eq $T2) "editor: $(OneLine $k3)"
Check 'Ctrl+Y moves the line down again' ($k4 -eq $k1) "editor: $(OneLine $k4)"
Check 'Alt+Shift+Down goes on to the last line and stops there' ($k5 -eq "[x] 二号`n说明`n[ ] 三号`n[ ] 一号") "editor: $(OneLine $k5)"
Check 'Alt+Shift+Up moves it one line up, and Esc saves the order shown' ($k6 -eq "[x] 二号`n说明`n[ ] 一号`n[ ] 三号" -and $saved -eq $k6) "editor: $(OneLine $k6); note: $(OneLine $saved)"
Pf fence delete $nid | Out-Null
}

if ($Part -match 'R') {
Case "R the drag recorded ($Rec)"
. C:\pf\kit\rec.ps1
$nid = "$((PfJson fence create --tool note --title 便签 --text $T0 --rect '100,100,300,240').fence.id)"
Start-Sleep -Seconds 1
$modes = @('light')
if ($Rec -eq 'after') { $modes = @('light', 'dark') }
foreach ($mode in $modes) {
  Pf settings set theme $mode | Out-Null
  Start-Sleep -Milliseconds 900
  $w = Fresh $nid $T0; $c = CRect $w; $boxes = Rects $w 'note_boxes'
  $from = TodoPt $c $boxes 3; $to = TodoPt $c $boxes 1
  [G]::MoveTo($from[0] + 30, $from[1] + 30); Start-Sleep -Milliseconds 300
  RecStart "todo-$Rec-$mode" -X ($w.l - 30) -Y ($w.t - 20) -W 360 -H 280 -Fps 30
  Start-Sleep -Milliseconds 400
  RecMark '按住第三条待办'
  Glide ($from[0] + 30) ($from[1] + 30) $from[0] $from[1] 4 30
  MDown; Start-Sleep -Milliseconds 500
  RecMark '向上拖到第一条待办上面'
  Glide $from[0] $from[1] ($from[0] + 6) ($to[1] - 4) 30 33
  Start-Sleep -Milliseconds 500
  RecMark '松开鼠标'
  MUp; Start-Sleep -Milliseconds 1200
  RecStop
  $after = NoteText $nid
  Log "R $Rec $mode note: $(OneLine $after) editor=$(EditorOpen)"
  Check "recorded in $mode mode" $true "note: $(OneLine $after); editor=$(EditorOpen)"
}
Pf settings set theme light | Out-Null
Pf fence delete $nid | Out-Null
}

Case 'cleanup'
if (EditorOpen) { [G]::Press(0x1B); Start-Sleep -Milliseconds 500 }
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force
[Anim]::Set($Anim0)
