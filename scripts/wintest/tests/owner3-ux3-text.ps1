# Owner round 2026-11-12, third batch: text on a fence and the tool fences, zh-CN.
# Parts (run.sh --var Part=...; all of them without it):
#   S  secondary text and a smart fence's count on a plate of middle brightness (dark mode over
#      a plain blue-grey wallpaper): contrast read from the screen
#   T  a script fence's first row stands on the line of a note's first line
#   U  a note's undo history: Ctrl+Z step by step, Ctrl+Y and Ctrl+Shift+Z forward, a paste, a
#      check box ticked while the note is edited
#   L  a web address in a note is a link: accent ink, hand cursor, underline under the pointer,
#      one click opens it; plain text while the note is edited; other addresses are no links
trap { PfTrap $_; continue }
if (-not $Part) { $Part = 'STUL' }
Case 'setup'
CloseWindows
Set-Culture zh-CN
$Anim0 = [Anim]::Get()
[Anim]::Set($true)
Add-Type -ReferencedAssemblies System.Drawing @'
using System; using System.Drawing; using System.Drawing.Imaging; using System.Runtime.InteropServices; using System.Text;
public static class T3 {
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
  static int[] Grab(int x, int y, int w, int h) {
    var b = new Bitmap(w, h, PixelFormat.Format32bppArgb);
    using (var g = Graphics.FromImage(b)) g.CopyFromScreen(x, y, 0, 0, new Size(w, h));
    var d = b.LockBits(new Rectangle(0, 0, w, h), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
    var px = new int[w * h]; Marshal.Copy(d.Scan0, px, 0, px.Length); b.UnlockBits(d); b.Dispose();
    return px;
  }
  static double Lin(int v) { double c = v / 255.0; return c <= 0.04045 ? c / 12.92 : Math.Pow((c + 0.055) / 1.055, 2.4); }
  static double Lum(int c) { return 0.2126 * Lin((c >> 16) & 255) + 0.7152 * Lin((c >> 8) & 255) + 0.0722 * Lin(c & 255); }
  static string Rgb(int c) { return ((c >> 16) & 255) + "," + ((c >> 8) & 255) + "," + (c & 255); }
  // A part of the screen that holds a plate and one kind of text: the plate is the median
  // pixel by luminance, the ink the pixel farthest from it; "contrast plate ink" (WCAG).
  public static string Read(int x, int y, int w, int h) {
    var px = Grab(x, y, w, h); var lum = new double[px.Length];
    for (int i = 0; i < px.Length; i++) lum[i] = Lum(px[i]);
    var s = (double[])lum.Clone(); Array.Sort(s); double med = s[s.Length / 2];
    int plate = 0, ink = 0; double near = 9, far = -1;
    for (int i = 0; i < px.Length; i++) { double d = Math.Abs(lum[i] - med); if (d < near) { near = d; plate = i; } if (d > far) { far = d; ink = i; } }
    double hi = Math.Max(lum[ink], lum[plate]), lo = Math.Min(lum[ink], lum[plate]);
    return string.Format(System.Globalization.CultureInfo.InvariantCulture, "{0:F2} {1} {2}", (hi + 0.05) / (lo + 0.05), Rgb(px[plate]), Rgb(px[ink]));
  }
  static int Grey(int c) { return (((c >> 16) & 255) * 299 + ((c >> 8) & 255) * 587 + (c & 255) * 114) / 1000; }
  // The first row and the first column of a part of the screen with a pixel `thr` away from
  // the part's median brightness (text on a plate): "top left", -1 without any.
  public static string InkStart(int x, int y, int w, int h, int thr) {
    var px = Grab(x, y, w, h); var l = new int[px.Length];
    for (int i = 0; i < px.Length; i++) l[i] = Grey(px[i]);
    var s = (int[])l.Clone(); Array.Sort(s); int med = s[s.Length / 2]; int top = -1, left = -1;
    for (int i = 0; i < l.Length; i++) if (Math.Abs(l[i] - med) >= thr) { if (top < 0) top = i / w; if (left < 0 || i % w < left) left = i % w; }
    return top + " " + left;
  }
  // Pixels of a part of the screen that are clearly blue (the accent ink of a link).
  public static int Blue(int x, int y, int w, int h) {
    var px = Grab(x, y, w, h); int n = 0;
    foreach (var c in px) { int r = (c >> 16) & 255, b = c & 255; if (b - r > 70) n++; }
    return n;
  }
}
'@
function SolidWall($r, $g, $b) {
  $bm = New-Object Drawing.Bitmap 32, 32; $gr = [Drawing.Graphics]::FromImage($bm); $gr.Clear([Drawing.Color]::FromArgb($r, $g, $b)); $gr.Dispose()
  $p = "$PfRun\wall-$r-$g-$b.bmp"; $bm.Save($p, [Drawing.Imaging.ImageFormat]::Bmp); $bm.Dispose()
  [G]::SystemParametersInfoW(0x14, 0, $p, 3) | Out-Null
}
GreyWallpaper
$sd = "$PfRun\scripts"
New-Item -ItemType Directory -Force $sd, "$PfRun\空文件夹" | Out-Null
$utf8 = New-Object Text.UTF8Encoding($true)
[IO.File]::WriteAllText("$sd\状态.ps1", @'
Write-Output "第一行 Ag 状态"
Write-Output "打开项目主页 | href=https://example.com"
Write-Output "---"
Write-Output "[x] 续费域名"
'@, $utf8)
$T3Files = '第一行 Ag.txt', '周报.docx', '课程.txt', '攻略.txt'
foreach ($n in $T3Files) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
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
function Ratio($read) { return [double](("$read" -split ' ')[0]) }
function CtrlShift($vk) { [G]::KeyDown(0x11); [G]::KeyDown(0x10); Start-Sleep -Milliseconds 30; [G]::Press($vk); Start-Sleep -Milliseconds 30; [G]::KeyUp(0x10); [G]::KeyUp(0x11) }

if ($Part -match 'S') {
Case 'S secondary text on a plate of middle brightness'
SolidWall 120 150 170
Pf settings set theme dark | Out-Null
# PecoFence looks at the wallpaper every 5 seconds.
Start-Sleep -Seconds 8
foreach ($n in '周报.docx', '课程.txt', '攻略.txt') { Set-Content -LiteralPath (Join-Path $PfDesk $n) -Value 'x' -Encoding UTF8 }
$i = 0
foreach ($t in '工作', '学习', '娱乐') {
  Pf fence create --title $t --rect '372,16,340,150' | Out-Null
  Pf item move (Join-Path $PfDesk @('周报.docx', '课程.txt', '攻略.txt')[$i]) --to $t | Out-Null
  $i++
}
Pf fence merge 学习 --into 工作 | Out-Null
Pf fence merge 娱乐 --into 工作 | Out-Null
Pf fence create --tool clock --title 时钟 --rect '1024,16,260,130' | Out-Null
Pf fence create --title 最近下载 --smart-folder "$PfRun\空文件夹" --rect '728,202,280,150' | Out-Null
Pf fence create --tool note --title 便签 --text "今天要做的事`n[x] 给安娜打电话" --rect '728,16,280,170' | Out-Null
Pf fence create --script "$sd\状态.ps1" --title 脚本 --rect '1300,16,284,200' | Out-Null
Start-Sleep -Seconds 5
ParkMouse 1500 800
Start-Sleep -Milliseconds 600
$c = (FenceGet '时钟').windowRect; $t = (FenceGet '工作').windowRect; $e = (FenceGet '最近下载').windowRect
$n = (FenceGet '便签').windowRect; $s = (FenceGet '脚本').windowRect
$time = [T3]::Read($c.x + 14, $c.y + 44, 130, 40)
$date = [T3]::Read($c.x + 14, $c.y + 86, 180, 18)
$tab = [T3]::Read($t.x + 16, $t.y + 9, 32, 16)
$done = [T3]::Read($n.x + 38, $n.y + 62, 110, 18)
$foot = [T3]::Read($s.x + 14, $s.y + 176, 150, 16)
$count = [T3]::Read($e.x + 222, $e.y + 11, 16, 16)
Crop 's-fences' 360 0 1240 380 | Out-Null
Log "S time=$time date=$date tab=$tab done=$done footer=$foot count=$count"
Check 'the plate is one of middle brightness with light text (the time under 4.5 : 1)' ((Ratio $time) -lt 4.5 -and (Ratio $time) -gt 2.6) "clock time: $time"
# Read from the screen on the reference build: date 2.29, resting tab 2.14, done to-do and
# footer 2.68 (the time 3.38); the thin strokes of small text read lower than their ink.
Check 'the clock''s date reaches 2.7 : 1 there and stays under the time' ((Ratio $date) -ge 2.7 -and (Ratio $date) -lt (Ratio $time)) "date: $date; time: $time"
Check 'the name of a tab that is not shown reaches 2.6 : 1' ((Ratio $tab) -ge 2.6 -and (Ratio $tab) -lt (Ratio $time)) "resting tab: $tab"
Check 'a done to-do reaches 2.95 : 1 and stays under the time' ((Ratio $done) -ge 2.95 -and (Ratio $done) -lt (Ratio $time)) "done to-do: $done"
Check 'the footer of a script fence reaches 2.95 : 1' ((Ratio $foot) -ge 2.95 -and (Ratio $foot) -lt (Ratio $time)) "footer: $foot"
Check 'the count of a smart fence reaches 3.9 : 1 on its capsule' ((Ratio $count) -ge 3.9) "smart count: $count"
# A dark plate keeps the style's own secondary text: weaker than the primary text by a clear step.
SolidWall 20 22 26
Start-Sleep -Seconds 8
$time2 = [T3]::Read($c.x + 14, $c.y + 44, 130, 40)
$date2 = [T3]::Read($c.x + 14, $c.y + 86, 180, 18)
Crop 's-fences-dark' 360 0 1240 380 | Out-Null
Check 'on a dark plate the date keeps its distance from the time' ((Ratio $date2) -ge 4.5 -and (Ratio $date2) -lt 0.8 * (Ratio $time2)) "date: $date2; time: $time2"
ClearFences
Pf settings set theme light | Out-Null
GreyWallpaper
Start-Sleep -Seconds 7
}

if ($Part -match 'T') {
Case 'T the first line of a note, a script fence, a list and a clock'
Set-Content -LiteralPath (Join-Path $PfDesk '第一行 Ag.txt') -Value 'x' -Encoding UTF8
Pf fence create --tool note --title 便签 --text "第一行 Ag 便签`n[ ] 待办" --rect '60,420,280,170' | Out-Null
Pf fence create --script "$sd\状态.ps1" --title 脚本 --rect '360,420,284,200' | Out-Null
Pf fence create --title 列表 --rect '664,420,300,170' | Out-Null
Pf item move (Join-Path $PfDesk '第一行 Ag.txt') --to 列表 | Out-Null
Pf fence set 列表 layout list | Out-Null
Pf fence create --tool clock --title 时钟 --rect '984,420,260,130' | Out-Null
Start-Sleep -Seconds 5
ParkMouse 1500 800
Start-Sleep -Milliseconds 600
$tops = @{}
foreach ($name in '便签', '脚本', '列表', '时钟') {
  $w = DumpOf $name
  $r = CRect $w
  # The text's own columns: past a list's icon, short of the right edge.
  $x0 = $r[0] + $(if ($name -eq '列表') { 44 } else { 10 })
  $tops[$name] = [T3]::InkStart($x0, $r[1], 120, 60, 60)
  Log "T $name content=$($r -join ',') ink top/left=$($tops[$name]) (from x=$($x0 - $r[0]))"
}
Crop 't-four' 48 408 1210 230 | Out-Null
$noteTop = [int](($tops['便签'] -split ' ')[0]); $scriptTop = [int](($tops['脚本'] -split ' ')[0])
$noteLeft = [int](($tops['便签'] -split ' ')[1]); $scriptLeft = [int](($tops['脚本'] -split ' ')[1]); $clockLeft = [int](($tops['时钟'] -split ' ')[1])
Check 'a script fence''s first row starts on the line of a note''s first line' ($noteTop -ge 0 -and $scriptTop -eq $noteTop) "first ink row under the title row: note $noteTop px, script $scriptTop px"
Check 'note, script fence and clock start their text at one left edge' ($noteLeft -eq $scriptLeft -and [math]::Abs($clockLeft - $noteLeft) -le 2) "first ink column: note $noteLeft, script $scriptLeft, clock $clockLeft (from 10 px inside the content)"
ClearFences
}

if ($Part -match 'U') {
Case 'U1 Ctrl+Z goes back step by step, Ctrl+Y and Ctrl+Shift+Z forward'
$nid = "$((PfJson fence create --tool note --title 便签 --text '买菜' --rect '100,100,300,220').fence.id)"
Start-Sleep -Seconds 1
$w = DumpOf '便签'
$c = CRect $w
[G]::Click($c[0] + 30, $c[1] + 12); Start-Sleep -Milliseconds 700
[G]::Press(0x23); Start-Sleep -Milliseconds 150
[G]::Type(' 西红柿'); Start-Sleep -Milliseconds 300
[G]::Type(' 鸡蛋'); Start-Sleep -Milliseconds 400
$typed = [T3]::EdText()
Chord 0x11 0x5A; Start-Sleep -Milliseconds 400
$z1 = [T3]::EdText()
Chord 0x11 0x5A; Start-Sleep -Milliseconds 400
$z2 = [T3]::EdText()
Chord 0x11 0x5A; Start-Sleep -Milliseconds 400
$z3 = [T3]::EdText()
Chord 0x11 0x59; Start-Sleep -Milliseconds 400
$y1 = [T3]::EdText()
CtrlShift 0x5A; Start-Sleep -Milliseconds 400
$y2 = [T3]::EdText()
Chord 0x11 0x59; Start-Sleep -Milliseconds 400
$y3 = [T3]::EdText()
FCrop 'u1-after-redo' $w
Check 'two words were typed after the note''s text' ($typed -eq '买菜 西红柿 鸡蛋') "editor: $typed"
Check 'the first Ctrl+Z takes back the last word' ($z1 -eq '买菜 西红柿') "editor: $z1"
Check 'the second Ctrl+Z takes back the word before it' ($z2 -eq '买菜') "editor: $z2"
Check 'a third Ctrl+Z with nothing left changes nothing' ($z3 -eq '买菜') "editor: $z3"
Check 'Ctrl+Y brings the first word back' ($y1 -eq '买菜 西红柿') "editor: $y1"
Check 'Ctrl+Shift+Z brings the second word back, a further Ctrl+Y changes nothing' ($y2 -eq '买菜 西红柿 鸡蛋' -and $y3 -eq $y2) "editor: $y2 / $y3"
# Typing after an undo: what was undone is gone, the caret is where the undo left it.
Chord 0x11 0x5A; Start-Sleep -Milliseconds 300
[G]::Type(' 牛奶'); Start-Sleep -Milliseconds 400
$t1 = [T3]::EdText()
Chord 0x11 0x59; Start-Sleep -Milliseconds 300
$t2 = [T3]::EdText()
Check 'typing after an undo goes in where the undo left the caret, and Ctrl+Y has nothing to bring back' ($t1 -eq '买菜 西红柿 牛奶' -and $t2 -eq $t1) "editor: $t1 / $t2"
[G]::Press(0x1B); Start-Sleep -Milliseconds 700
$saved = NoteText $nid
Check 'Esc saves what the editor showed' ($saved -eq '买菜 西红柿 牛奶') "note: $(OneLine $saved)"

Case 'U2 a paste and a run of Backspace are one step each'
Pf fence set $nid noteText '买菜' --string | Out-Null
Start-Sleep -Milliseconds 700
Set-Clipboard -Value '周末去超市 '
[G]::Click($c[0] + 30, $c[1] + 12); Start-Sleep -Milliseconds 700
[G]::Press(0x24); Start-Sleep -Milliseconds 150
Chord 0x11 0x56; Start-Sleep -Milliseconds 400
$p1 = [T3]::EdText()
[G]::Press(0x23); Start-Sleep -Milliseconds 150
[G]::Press(0x08); Start-Sleep -Milliseconds 80; [G]::Press(0x08); Start-Sleep -Milliseconds 300
$p2 = [T3]::EdText()
Chord 0x11 0x5A; Start-Sleep -Milliseconds 400
$p3 = [T3]::EdText()
Chord 0x11 0x5A; Start-Sleep -Milliseconds 400
$p4 = [T3]::EdText()
Check 'the paste went in and two Backspaces took the note''s two characters' ($p1 -eq '周末去超市 买菜' -and $p2 -eq '周末去超市 ') "editor: $p1 / $p2"
Check 'Ctrl+Z brings both deleted characters back at once' ($p3 -eq '周末去超市 买菜') "editor: $p3"
Check 'the next Ctrl+Z takes the paste back' ($p4 -eq '买菜') "editor: $p4"
[G]::Press(0x1B); Start-Sleep -Milliseconds 700

Case 'U3 a check box ticked while the note is edited'
Pf fence set $nid noteText "[ ] 西红柿`n[ ] 鸡蛋" --string | Out-Null
Start-Sleep -Milliseconds 900
$w = DumpOf '便签'
$boxes = Rects $w 'note_boxes'
$b0 = $boxes[0]; $b1 = $boxes[1]
[G]::Click($c[0] + [int]$b0[2] + 60, $c[1] + [int](($b0[1] + $b0[3]) / 2)); Start-Sleep -Milliseconds 700
$open0 = EditorOpen
[G]::Type('大'); Start-Sleep -Milliseconds 300
[G]::Click($c[0] + [int](($b1[0] + $b1[2]) / 2), $c[1] + [int](($b1[1] + $b1[3]) / 2)); Start-Sleep -Milliseconds 700
$open1 = EditorOpen
$k1 = [T3]::EdText()
FCrop 'u3-ticked-while-editing' $w
[G]::Type('的'); Start-Sleep -Milliseconds 300
$k2 = [T3]::EdText()
Chord 0x11 0x5A; Start-Sleep -Milliseconds 300
Chord 0x11 0x5A; Start-Sleep -Milliseconds 400
$k3 = [T3]::EdText()
Chord 0x11 0x59; Start-Sleep -Milliseconds 400
$k4 = [T3]::EdText()
[G]::Press(0x1B); Start-Sleep -Milliseconds 700
$saved = NoteText $nid
Check 'a click on a check box while the note is edited ticks it and keeps the editor open' ($open0 -and $open1 -and $k1 -match '\[x\] 鸡蛋') "editor open before / after the click: $open0 / $open1; editor: $(OneLine $k1)"
Check 'the caret stays where it was: the next character goes after the one typed before' ($k2 -match '大的') "editor: $(OneLine $k2)"
Check 'Ctrl+Z twice takes back the character and then the tick' ($k3 -match '\[ \] 鸡蛋' -and $k3 -notmatch '的') "editor: $(OneLine $k3)"
Check 'Ctrl+Y ticks it again, Esc saves the tick' ($k4 -match '\[x\] 鸡蛋' -and $saved -match '\[x\] 鸡蛋') "editor: $(OneLine $k4); note: $(OneLine $saved)"
Pf fence delete $nid | Out-Null
}

if ($Part -match 'L') {
Case 'L1 a web address in a note is a link'
$lid = "$((PfJson fence create --tool note --title 链接 --text "周报 https://example.com/wiki 详见`n[ ] 看 http://example.org/a。`n文件 file:///C:/Windows/notepad.exe" --rect '100,100,380,240').fence.id)"
ParkMouse 900 700
Start-Sleep -Seconds 1
$w = DumpOf '链接'
$c = CRect $w
$raw = DumpVal $w 'note_links'
$links = Rects $w 'note_links'
$nlinks = if ($raw -and $raw -ne '-') { @($raw -split ';').Count } else { 0 }
Log "L links=$raw"
Check 'the two web addresses are links, the file address is none' ($nlinks -eq 2 -and $links[0] -and $links[1] -and -not $links[2]) "links drawn (line:l,t,r,b in DIPs): $raw"
$l0 = $links[0]
$lx = $c[0] + [int](($l0[0] + $l0[2]) / 2); $ly = $c[1] + [int](($l0[1] + $l0[3]) / 2)
$rx = $c[0] + [int]$l0[0]; $ry = $c[1] + [int]$l0[1]; $rw = [int]($l0[2] - $l0[0]); $rh = [int]($l0[3] - $l0[1])
$blueRest = [T3]::Blue($rx, $ry, $rw, $rh)
$bluePlain = [T3]::Blue($c[0] + 14, $ry, [int]$l0[0] - 16, $rh)
FCrop 'l1-link-rest' $w
Check 'the address is drawn in the accent ink, the words before it are not' ($blueRest -ge 60 -and $bluePlain -eq 0) "blue pixels: address $blueRest, the words before it $bluePlain"
[G]::MoveTo($lx, $ly); Start-Sleep -Milliseconds 500
$cursorLink = [T3]::Cursor()
$blueHover = [T3]::Blue($rx, $ry, $rw, $rh + 2)
FCrop 'l1-link-hover' $w
[G]::MoveTo($c[0] + 24, $ly); Start-Sleep -Milliseconds 500
$cursorText = [T3]::Cursor()
$blueAway = [T3]::Blue($rx, $ry, $rw, $rh + 2)
Check 'over the address the pointer is the hand, over the words beside it the text cursor' ($cursorLink -eq 'hand' -and $cursorText -eq 'ibeam') "address: $cursorLink, words: $cursorText"
Check 'the address is underlined while the pointer is on it' ($blueHover -ge $blueAway + [int]($rw * 0.6)) "blue pixels: under the pointer $blueHover, pointer away $blueAway (address $rw px wide)"

Case 'L2 one click opens the address in the browser'
$n0 = @(PfLogAll | Where-Object { $_ -match 'note link opened in the browser' }).Count
[G]::Click($lx, $ly); Start-Sleep -Seconds 3
$lines = @(PfLogAll | Where-Object { $_ -match 'note link opened in the browser' })
$ed = EditorOpen
$browser = @(Get-Process msedge, chrome, firefox -ErrorAction SilentlyContinue).Count
Shot 'l2-after-click'
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1
CloseWindows
Check 'a click on the address opens it and does not start the editor' ($lines.Count -gt $n0 -and $lines[-1] -match 'ok=true' -and -not $ed) "log lines: $n0 -> $($lines.Count) $(if ($lines.Count) { $lines[-1] -replace '^\S+\s+', '' }); editor=$ed; browser processes=$browser"
# A press taken back by sliding off the address opens nothing.
[G]::MoveTo($lx, $ly); Start-Sleep -Milliseconds 300
MDown; Start-Sleep -Milliseconds 200
Glide $lx $ly ($lx + 10) ($ly + 120) 8 20
MUp; Start-Sleep -Milliseconds 700
$lines2 = @(PfLogAll | Where-Object { $_ -match 'note link opened in the browser' })
Check 'a press that slides off the address opens nothing and starts no editor' ($lines2.Count -eq $lines.Count -and -not (EditorOpen)) "log lines: $($lines.Count) -> $($lines2.Count); editor=$(EditorOpen)"

Case 'L3 while the note is edited the address is plain text'
[G]::Click($c[0] + 24, $ly); Start-Sleep -Milliseconds 800
$ed1 = EditorOpen
[G]::MoveTo($lx, $ly); Start-Sleep -Milliseconds 500
$cursorEdit = [T3]::Cursor()
ParkMouse 900 700
Start-Sleep -Milliseconds 400
$blueEdit = [T3]::Blue($rx, $ry, $rw, $rh)
FCrop 'l3-editing' $w
[G]::Click($lx, $ly); Start-Sleep -Milliseconds 800
$lines3 = @(PfLogAll | Where-Object { $_ -match 'note link opened in the browser' })
$ed2 = EditorOpen
[G]::Press(0x1B); Start-Sleep -Milliseconds 700
ParkMouse 900 700
Start-Sleep -Milliseconds 500
$blueBack = [T3]::Blue($rx, $ry, $rw, $rh)
Check 'the editor opens by a click beside the address' $ed1 "editor=$ed1"
Check 'while editing the address is plain text under the text cursor' ($cursorEdit -eq 'ibeam' -and $blueEdit -eq 0) "cursor: $cursorEdit; blue pixels: $blueEdit"
Check 'a click on it while editing places the caret and opens nothing' ($lines3.Count -eq $lines2.Count -and $ed2) "log lines: $($lines2.Count) -> $($lines3.Count); editor=$ed2"
Check 'after Esc the address is a link again' ($blueBack -ge 60) "blue pixels: $blueBack"

Case 'L4 an address that is no web address starts nothing'
$w = DumpOf '链接'
# The third line: the file address. A click there starts the editor, as on any text.
$np0 = @(Get-Process notepad -ErrorAction SilentlyContinue).Count
[G]::Click($c[0] + 150, $c[1] + [int]($l0[1] + 59)); Start-Sleep -Milliseconds 900
$ed3 = EditorOpen
$np1 = @(Get-Process notepad -ErrorAction SilentlyContinue).Count
[G]::Press(0x1B); Start-Sleep -Milliseconds 700
Check 'a click on the file address starts the editor and no program' ($ed3 -and $np1 -eq $np0) "editor=$ed3; notepad processes: $np0 -> $np1"
Pf fence delete $lid | Out-Null
}

Case 'cleanup'
if (EditorOpen) { [G]::Press(0x1B); Start-Sleep -Milliseconds 500 }
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force
foreach ($n in $T3Files) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
[Anim]::Set($Anim0)
