# Owner round 2026-11-12, second batch: the tool fences (note, clock, script) under the pointer
# and the keyboard, zh-CN.
# Parts (run.sh --var Part=...; all of them without it):
#   A  the clock: the date keeps its form when the fence gets taller; a change of the region's
#      time format or of the system time shows at once
#   B  the note: Enter goes on with a to-do list; a check box answers the pointer and acts on
#      release; the pointer over a note's text; a click into a note during a Peek keeps the Peek
#   C  script rows: they act on release, the hover plate fades
#   D  a run started by the interval does not make the footer blink (waits up to 2.5 minutes)
#   E  the clock's menu says whether the seconds show; the footer of a narrow script fence;
#      the first run of a slow script
#   R  the recordings for the before/after pictures (no checks of the product)
trap { PfTrap $_; continue }
if (-not $Part) { $Part = 'ABCDE' }
Case 'setup'
CloseWindows
Set-Culture zh-CN
GreyWallpaper
$Anim0 = [Anim]::Get()
[Anim]::Set($true)
. C:\pf\kit\rec.ps1
Add-Type -ReferencedAssemblies System.Drawing @'
using System; using System.Collections.Generic; using System.Diagnostics; using System.Drawing;
using System.Drawing.Imaging; using System.Runtime.InteropServices; using System.Threading;
public static class UX {
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr SendMessageTimeoutW(IntPtr h, uint m, IntPtr w, string l, uint f, uint t, out IntPtr r);
  [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
  [StructLayout(LayoutKind.Sequential)] struct CI { public int cb; public int flags; public IntPtr h; public int x; public int y; }
  [DllImport("user32.dll")] static extern bool GetCursorInfo(ref CI ci);
  [DllImport("user32.dll")] static extern IntPtr LoadCursorW(IntPtr inst, IntPtr id);
  // What Windows broadcasts after a change on the region settings page, and after the system
  // time or the time zone was set.
  public static void Intl() { IntPtr r; SendMessageTimeoutW((IntPtr)0xffff, 0x1A, IntPtr.Zero, "intl", 2, 2000, out r); }
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr FindWindowW(string cls, string title);
  // WM_TIMECHANGE, straight to PecoFence's control window: Windows lets only itself broadcast
  // it (after the time or the time zone was set, which the test user may not do).
  public static bool TimeChange() {
    var h = FindWindowW("PecoFence.Control", null); IntPtr r;
    return h != IntPtr.Zero && SendMessageTimeoutW(h, 0x1E, IntPtr.Zero, null, 2, 2000, out r) != IntPtr.Zero;
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
  static int Lum(int c) { return (((c >> 16) & 255) * 299 + ((c >> 8) & 255) * 587 + (c & 255) * 114) / 1000; }
  public static int[] Snap(int x, int y, int w, int h) { return Grab(x, y, w, h); }
  // Pixels of a region that differ by `thr` from the same region grabbed before.
  public static int Changed(int[] before, int x, int y, int w, int h, int thr) {
    var now = Grab(x, y, w, h); int n = 0;
    for (int i = 0; i < now.Length; i++) if (Math.Abs(Lum(now[i]) - Lum(before[i])) >= thr) n++;
    return n;
  }
  static double Mean(int[] px) { long s = 0; foreach (var c in px) s += Lum(c); return (double)s / px.Length; }
  // Text in a region: how many pixels are `thr` darker or lighter than the region's median,
  // and the column of the rightmost one (-1 without any), as "count right".
  public static string Ink(int x, int y, int w, int h, int thr) {
    var px = Grab(x, y, w, h); var l = new int[px.Length];
    for (int i = 0; i < px.Length; i++) l[i] = Lum(px[i]);
    var s = (int[])l.Clone(); Array.Sort(s); int med = s[s.Length / 2]; int n = 0, right = -1;
    for (int i = 0; i < l.Length; i++) if (Math.Abs(l[i] - med) >= thr) { n++; if (i % w > right) right = i % w; }
    return n + " " + right;
  }
  // Puts the pointer at (px, py) and reads the mean brightness of a region as fast as it can
  // for `ms`: the values in the order they were seen, repeats dropped, as "t:value" pairs.
  public static string Watch(int px, int py, int x, int y, int w, int h, int ms) {
    var seen = new List<string>(); double last = -1; var sw = Stopwatch.StartNew();
    SetCursorPos(px, py);
    while (sw.ElapsedMilliseconds < ms) {
      double m = Math.Round(Mean(Grab(x, y, w, h)), 1);
      if (Math.Abs(m - last) >= 0.3) { seen.Add(sw.ElapsedMilliseconds + ":" + m.ToString("0.0")); last = m; }
      Thread.Sleep(2);
    }
    return string.Join(" ", seen.ToArray());
  }
  // A watcher on its own thread: the largest number of pixels of a region that differed (by
  // `thr`) from `before` at any moment, and the picture of that moment.
  static Thread worker; static volatile bool running; static int worst; static string worstFile;
  public static void WatchStart(int[] before, int x, int y, int w, int h, int thr, string file) {
    worst = 0; worstFile = file; running = true;
    worker = new Thread(() => {
      while (running) {
        var now = Grab(x, y, w, h); int n = 0;
        for (int i = 0; i < now.Length; i++) if (Math.Abs(Lum(now[i]) - Lum(before[i])) >= thr) n++;
        if (n > worst) {
          worst = n;
          using (var b = new Bitmap(w, h, PixelFormat.Format32bppArgb)) {
            var d = b.LockBits(new Rectangle(0, 0, w, h), ImageLockMode.WriteOnly, PixelFormat.Format32bppArgb);
            Marshal.Copy(now, 0, d.Scan0, now.Length); b.UnlockBits(d); b.Save(worstFile, ImageFormat.Png);
          }
        }
        Thread.Sleep(15);
      }
    });
    worker.IsBackground = true; worker.Start();
  }
  public static int WatchStop() { running = false; if (worker != null) worker.Join(2000); return worst; }
}
'@
$sd = "$PfRun\scripts"
New-Item -ItemType Directory -Force $sd | Out-Null
$utf8 = New-Object Text.UTF8Encoding($true)
[IO.File]::WriteAllText("$sd\状态面板.ps1", @'
Write-Output "今天的任务 | color=blue"
Write-Output "打开项目主页 | href=https://example.com"
Write-Output "复制服务器地址 | copy=10.0.12.34"
Write-Output "---"
Write-Output "磁盘剩余 12 GB | color=red"
Write-Output "一行没有动作的普通文字"
'@, $utf8)
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
# The content rect of a dumped window in screen px, and the rects a dump field lists
# ("index:l,t,r,b;..." in content DIPs) by index.
function CRect($w) { $cx = [int](DumpVal $w 'content_x'); $cy = [int](DumpVal $w 'content_y'); return @(($w.l + $cx), ($w.t + $cy), ($w.r - $w.l - $cx), ($w.b - $w.t - $cy)) }
function Rects($w, $name) {
  $o = @{}; $v = DumpVal $w $name
  if ($v -and $v -ne '-') { foreach ($p in ($v -split ';')) { $kv = $p -split ':'; $o[[int]$kv[0]] = @($kv[1] -split ',' | ForEach-Object { [double]$_ }) } }
  return $o
}
function FCrop($name, $w, $m = 12) { Crop $name ([math]::Max(0, $w.l - $m)) ([math]::Max(0, $w.t - $m)) ($w.r - $w.l + 2 * $m) ($w.b - $w.t + 2 * $m) | Out-Null }
function EditorOpen() { $h = [G]::FindWindowW('PecoFence.NoteEdit', [NullString]::Value); return ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) }
function PeekOn() { return ([W]::Tops('PecoFence.PeekDimmer')).Count -gt 0 }
function NoteText($id) { return "$((FenceGet $id).noteText)" }
function OneLine($t) { return ($t -replace "`n", ' / ') }

if ($Part -match 'A') {
Case 'A1 a taller clock keeps the long date'
$cid = NewTool '时钟' '100,100,240,140' 'clock' ''
Start-Sleep -Seconds 1
$w = DumpOf '时钟'
$d1 = DumpVal $w 'clock_date'
Pf fence move $cid --rect '100,100,240,400' | Out-Null
Start-Sleep -Milliseconds 900
$w = DumpOf '时钟'
$d2 = DumpVal $w 'clock_date'
FCrop 'a1-tall-clock' $w
Check 'a new clock shows the long date' ($d1 -eq 'long') "date form: $d1"
Check 'the same clock made taller keeps the long date' ($d2 -eq 'long') "date form: $d2, time / date sizes $(DumpVal $w 'clock_text') DIPs"
$forms = @(foreach ($h in 150, 180, 220, 260, 300, 340, 380, 420, 460) { Pf fence move $cid --rect "100,100,240,$h" | Out-Null; Start-Sleep -Milliseconds 400; DumpVal (DumpOf '时钟') 'clock_date' })
Check 'at nine heights from 150 to 460 the date stays the long one' (@($forms | Where-Object { $_ -ne 'long' }).Count -eq 0) "forms: $($forms -join ' ')"
Pf fence move $cid --rect '100,100,240,140' | Out-Null

Case 'A2 a change of the time format shows at once'
Pf settings set tools.clockSeconds false | Out-Null
Start-Sleep -Milliseconds 600
# Away from the minute's end: the clock's own tick would draw it too.
while ((Get-Date).Second -gt 48 -or (Get-Date).Second -lt 3) { Start-Sleep -Milliseconds 500 }
$w = DumpOf '时钟'
$r = CRect $w
$before = [UX]::Snap($r[0], $r[1], $r[2], $r[3])
Set-Culture en-US
[UX]::Intl()
Start-Sleep -Milliseconds 1500
$changed = [UX]::Changed($before, $r[0], $r[1], $r[2], $r[3], 40)
FCrop 'a2-format-changed' $w
Check 'the clock shows the new time format within 1.5 s' ($changed -gt 300) "pixels of the clock that changed: $changed (12 hours with AM / PM and an English date)"
Set-Culture zh-CN
[UX]::Intl()
Start-Sleep -Milliseconds 800

Case 'A3 a change of the system time'
$n0 = @(PfLogAll | Where-Object { $_ -match 'clocks drawn again' -and $_ -match 'system time' }).Count
$sent = [UX]::TimeChange()
Start-Sleep -Milliseconds 800
$lines = @(PfLogAll | Where-Object { $_ -match 'clocks drawn again' -and $_ -match 'system time' })
Check 'the clock is drawn again when Windows says the time changed' ($lines.Count -gt $n0) "sent=$sent log lines: $n0 -> $($lines.Count) $(if ($lines.Count) { $lines[-1] -replace '^\S+\s+', '' })"
Pf fence delete $cid | Out-Null
}

if ($Part -match 'B') {
Case 'B1 Enter goes on with a to-do list'
$nid = NewTool '便签' '100,100,280,260' 'note' "买菜`n[ ] 西红柿`n[ ] 鸡蛋"
Start-Sleep -Seconds 1
$w = DumpOf '便签'
$c = CRect $w
$boxes = Rects $w 'note_boxes'
$b2 = $boxes[2]
[G]::Click($c[0] + [int]$b2[2] + 40, $c[1] + [int](($b2[1] + $b2[3]) / 2)); Start-Sleep -Milliseconds 700
[G]::Press(0x23); [G]::Press(0x0D); Start-Sleep -Milliseconds 300
[G]::Type('牛奶'); Start-Sleep -Milliseconds 400
FCrop 'b1-next-todo' $w
[G]::Press(0x0D); Start-Sleep -Milliseconds 300
[G]::Press(0x0D); Start-Sleep -Milliseconds 300
[G]::Type('备注'); Start-Sleep -Milliseconds 400
FCrop 'b1-list-ended' $w
[G]::Press(0x1B); Start-Sleep -Milliseconds 700
$t = NoteText $nid
Check 'Enter after a to-do starts the next one, Enter on an empty one ends the list' ($t -eq "买菜`n[ ] 西红柿`n[ ] 鸡蛋`n[ ] 牛奶`n备注") "note: $(OneLine $t)"
# In the middle of a to-do's text: both halves are to-dos.
Pf fence set $nid noteText "[ ] 西红柿鸡蛋" --string | Out-Null
Start-Sleep -Milliseconds 700
$w = DumpOf '便签'
$b0 = (Rects $w 'note_boxes')[0]
[G]::Click($c[0] + [int]$b0[2] + 60, $c[1] + [int](($b0[1] + $b0[3]) / 2)); Start-Sleep -Milliseconds 700
[G]::Press(0x24); [G]::Press(0x27); [G]::Press(0x27); [G]::Press(0x27); [G]::Press(0x0D); Start-Sleep -Milliseconds 400
[G]::Press(0x1B); Start-Sleep -Milliseconds 700
$t = NoteText $nid
Check 'Enter inside a to-do splits it into two to-dos' ($t -eq "[ ] 西红柿`n[ ] 鸡蛋") "note: $(OneLine $t)"
# Shift+Enter stays a plain line break.
[G]::Click($c[0] + [int]$b0[2] + 60, $c[1] + [int](($b0[1] + $b0[3]) / 2)); Start-Sleep -Milliseconds 700
[G]::Press(0x23); Chord 0x10 0x0D; [G]::Type('x'); Start-Sleep -Milliseconds 300
[G]::Press(0x1B); Start-Sleep -Milliseconds 700
$t = NoteText $nid
Check 'Shift+Enter starts a plain line' ($t -eq "[ ] 西红柿`nx`n[ ] 鸡蛋") "note: $(OneLine $t)"
# The old way still works: a marker typed on the line Enter began as a to-do is not doubled.
Pf fence set $nid noteText "[ ] 茶" --string | Out-Null
Start-Sleep -Milliseconds 700
[G]::Click($c[0] + [int]$b0[2] + 60, $c[1] + [int](($b0[1] + $b0[3]) / 2)); Start-Sleep -Milliseconds 700
[G]::Press(0x23); [G]::Press(0x0D); Start-Sleep -Milliseconds 300
[G]::Type('[ ] 咖啡'); Start-Sleep -Milliseconds 400
[G]::Press(0x1B); Start-Sleep -Milliseconds 700
$t = NoteText $nid
Check 'a marker typed after Enter on a to-do line is not doubled' ($t -eq "[ ] 茶`n[ ] 咖啡") "note: $(OneLine $t)"

Case 'B2 a check box answers the pointer and acts on release'
Pf fence set $nid noteText "买菜`n[ ] 西红柿`n[x] 鸡蛋" --string | Out-Null
ParkMouse 700 700
Start-Sleep -Milliseconds 900
$w = DumpOf '便签'
$boxes = Rects $w 'note_boxes'
$b1 = $boxes[1]; $b2 = $boxes[2]
$bx = $c[0] + [int](($b1[0] + $b1[2]) / 2); $by = $c[1] + [int](($b1[1] + $b1[3]) / 2)
$rx = $c[0] + [int]$b1[0]; $ry = $c[1] + [int]$b1[1]
$rest = [UX]::Snap($rx, $ry, 16, 16)
$rest2 = [UX]::Snap($rx, $c[1] + [int]$b2[1], 16, 16)
[G]::MoveTo($bx, $by); Start-Sleep -Milliseconds 500
$hover = [UX]::Changed($rest, $rx, $ry, 16, 16, 5)
$cursorBox = [UX]::Cursor()
Crop 'b2-box-hover' ($w.l) ($c[1] + [int]$b1[1] - 8) 200 60 | Out-Null
MDown; Start-Sleep -Milliseconds 500
$held = NoteText $nid
$press = [UX]::Changed($rest, $rx, $ry, 16, 16, 5)
Crop 'b2-box-pressed' ($w.l) ($c[1] + [int]$b1[1] - 8) 200 60 | Out-Null
MUp; Start-Sleep -Milliseconds 700
$after = NoteText $nid
[G]::MoveTo($bx, $c[1] + [int](($b2[1] + $b2[3]) / 2)); Start-Sleep -Milliseconds 500
$hover2 = [UX]::Changed($rest2, $rx, $c[1] + [int]$b2[1], 16, 16, 5)
Check 'an open check box under the pointer looks different from one at rest' ($hover -ge 60) "pixels of the 16 x 16 box that changed: $hover"
Check 'a ticked check box under the pointer looks different too' ($hover2 -ge 60) "pixels that changed: $hover2"
Check 'the pointer over a check box is the arrow' ($cursorBox -eq 'arrow') "cursor: $cursorBox"
Check 'while the button is held the to-do is not ticked yet, and the box looks pressed' ($held -match '\[ \] 西红柿' -and $press -ge 60) "note while held: $(OneLine $held); pixels changed: $press"
Check 'the release ticks it' ($after -match '\[x\] 西红柿') "note: $(OneLine $after)"
[G]::MoveTo($bx, $by); Start-Sleep -Milliseconds 300
MDown; Start-Sleep -Milliseconds 250
Glide $bx $by ($bx + 120) ($by + 90) 8 20
MUp; Start-Sleep -Milliseconds 700
$t = NoteText $nid
Check 'a press taken back by sliding off the box changes nothing' ($t -eq $after -and -not (EditorOpen)) "note: $(OneLine $t) editor=$(EditorOpen)"
# A quick double click ticks and unticks.
DoubleClick $bx $by; Start-Sleep -Milliseconds 700
$t = NoteText $nid
Check 'two quick clicks on a box tick it and untick it' ($t -eq $after) "note: $(OneLine $t)"

Case 'B3 the pointer over a note says that a click writes'
[G]::MoveTo($c[0] + 60, $c[1] + 12); Start-Sleep -Milliseconds 500
$cursorText = [UX]::Cursor()
[G]::MoveTo($c[0] + 120, $c[1] + 180); Start-Sleep -Milliseconds 500
$cursorEmpty = [UX]::Cursor()
[G]::MoveTo($w.l + 120, $w.t + 14); Start-Sleep -Milliseconds 500
$cursorTitle = [UX]::Cursor()
Check 'over a note''s text and the room under it the pointer is the text cursor' ($cursorText -eq 'ibeam' -and $cursorEmpty -eq 'ibeam') "text: $cursorText, under the text: $cursorEmpty"
Check 'over the title row it stays the arrow' ($cursorTitle -eq 'arrow') "title: $cursorTitle"

Case 'B4 a click into a note during a Peek'
Start-Process notepad
Start-Sleep -Seconds 2
$np = (Get-Process notepad | Select-Object -First 1).MainWindowHandle
[L]::MoveWindow($np, 60, 60, 700, 420, $true) | Out-Null
Start-Sleep -Milliseconds 800
PeekKey
Start-Sleep -Milliseconds 1200
$on0 = PeekOn
# A check box in the Peek: a click ticks it and the Peek stays.
$peekBox = (Rects (DumpOf '便签') 'note_boxes')[1]
$tick0 = NoteText $nid
[G]::Click($c[0] + [int](($peekBox[0] + $peekBox[2]) / 2), $c[1] + [int](($peekBox[1] + $peekBox[3]) / 2)); Start-Sleep -Milliseconds 800
$tick1 = NoteText $nid
Check 'a click on a check box during the Peek ticks it and keeps the Peek' ($tick1 -ne $tick0 -and (PeekOn)) "note before: $(OneLine $tick0); after: $(OneLine $tick1); peek=$(PeekOn)"
[G]::Click($c[0] + 60, $c[1] + 12); Start-Sleep -Milliseconds 900
$on1 = PeekOn; $ed1 = EditorOpen
[G]::Type('速览'); Start-Sleep -Milliseconds 500
Shot 'b4-peek-typing'
[G]::Press(0x1B); Start-Sleep -Milliseconds 800
$on2 = PeekOn; $ed2 = EditorOpen
$t = NoteText $nid
[G]::Press(0x1B); Start-Sleep -Milliseconds 800
$on3 = PeekOn
Check 'the Peek is on over the window' $on0 "peek=$on0"
Check 'after a click in the note the Peek stays and the editor is open' ($on1 -and $ed1) "peek=$on1 editor=$ed1"
Check 'Esc closes the editor, the Peek stays, the typed text is in the note' ($on2 -and -not $ed2 -and $t -match '速览') "peek=$on2 editor=$ed2 note: $(OneLine $t)"
Check 'a second Esc ends the Peek' (-not $on3) "peek=$on3"
if (PeekOn) { PeekKey; Start-Sleep -Milliseconds 600 }
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
Pf fence delete $nid | Out-Null
}

if ($Part -match 'C') {
Case 'C1 a script row acts on release'
$sid = NewTool '状态面板' '420,100,300,230' 'script' "$sd\状态面板.ps1"
Start-Sleep -Seconds 4
$w = DumpOf '状态面板'
$c = CRect $w
$rows = Rects $w 'script_rows'
$r2 = $rows[2]
$px = $c[0] + 150; $py = $c[1] + [int](($r2[1] + $r2[3]) / 2)
Set-Clipboard -Value 'before'
[G]::MoveTo($px, $py); Start-Sleep -Milliseconds 500
$plateRest = [UX]::Snap($c[0] + [int]$r2[2] - 60, $py - 6, 40, 12)
MDown; Start-Sleep -Milliseconds 500
$held = Get-Clipboard
$platePressed = [UX]::Changed($plateRest, $c[0] + [int]$r2[2] - 60, $py - 6, 40, 12, 2)
FCrop 'c1-row-pressed' $w
MUp; Start-Sleep -Milliseconds 700
$after = Get-Clipboard
Check 'while the button is held nothing is copied yet' ($held -eq 'before') "clipboard while held: '$held'"
Check 'the held row looks different from the hovered row' ($platePressed -ge 200) "pixels of the plate that changed: $platePressed of 480"
Check 'the release copies' ($after -eq '10.0.12.34') "clipboard: '$after'"
Set-Clipboard -Value 'before'
[G]::MoveTo($px, $py); Start-Sleep -Milliseconds 300
MDown; Start-Sleep -Milliseconds 250
Glide $px $py ($px + 30) ($w.b + 60) 8 20
MUp; Start-Sleep -Milliseconds 700
Check 'a press taken back by sliding off the row copies nothing' ((Get-Clipboard) -eq 'before') "clipboard: '$(Get-Clipboard)'"

Case 'C2 the hover plate fades in'
ParkMouse 700 700
Start-Sleep -Milliseconds 800
$sx = $c[0] + [int]$r2[2] - 60; $sy = $py - 6
$seen = [UX]::Watch($px, $py, $sx, $sy, 40, 12, 450)
$vals = @($seen -split ' ' | Where-Object { $_ } | ForEach-Object { [double](($_ -split ':')[1]) })
$restV = $vals[0]; $fullV = $vals[-1]
$between = @($vals | Where-Object { [math]::Abs($_ - $restV) -ge 1 -and [math]::Abs($_ - $fullV) -ge 1 }).Count
Check 'the plate passes through at least one strength between none and full' ($between -ge 1 -and [math]::Abs($fullV - $restV) -ge 4) "brightness of the plate after the pointer arrived (ms:value): $seen"
# With Windows' animations off the plate is there at once.
[Anim]::Set($false)
ParkMouse 700 700
Start-Sleep -Milliseconds 1200
$seen = [UX]::Watch($px, $py, $sx, $sy, 40, 12, 450)
$vals = @($seen -split ' ' | Where-Object { $_ } | ForEach-Object { [double](($_ -split ':')[1]) })
$between = @($vals | Where-Object { [math]::Abs($_ - $vals[0]) -ge 1 -and [math]::Abs($_ - $vals[-1]) -ge 1 }).Count
Check 'with Windows'' animations off the plate snaps' ($between -eq 0 -and [math]::Abs($vals[-1] - $vals[0]) -ge 4) "brightness of the plate (ms:value): $seen"
[Anim]::Set($true)
Start-Sleep -Milliseconds 800
ParkMouse 700 700
Pf fence delete $sid | Out-Null
}

if ($Part -match 'D') {
Case 'D1 a run started by the interval does not make the footer blink'
$sid = NewTool '状态面板' '420,100,300,230' 'script' "$sd\状态面板.ps1"
Start-Sleep -Seconds 4
Pf fence set $sid scriptInterval 1 | Out-Null
ParkMouse 700 700
$w = DumpOf '状态面板'
# The footer line: the lowest 26 px of the fence, off the rounded corners.
$fx = $w.l + 14; $fy = $w.b - 28; $fw = 240; $fh = 22
$runs0 = @(PfLogAll | Where-Object { $_ -match 'script fence run ended' }).Count
$base = [UX]::Snap($fx, $fy, $fw, $fh)
[UX]::WatchStart($base, $fx, $fy, $fw, $fh, 40, "$PfShots\d1-footer-worst.png")
$runs = $runs0
for ($i = 0; $i -lt 150 -and $runs -le $runs0; $i++) {
  Start-Sleep -Milliseconds 1000
  $runs = @(PfLogAll | Where-Object { $_ -match 'script fence run ended' }).Count
}
Start-Sleep -Milliseconds 1200
$worst = [UX]::WatchStop()
# The footer names the minute of the last run: what differs once the run is over is that.
$final = [UX]::Changed($base, $fx, $fy, $fw, $fh, 40)
Crop 'd1-footer-after' $fx $fy $fw $fh | Out-Null
Check 'the interval started a run' ($runs -gt $runs0) "runs ended: $runs0 -> $runs after $i s"
Check 'the footer did not change to another text and back during that run' ($worst - $final -le 40) "pixels of the footer line that differed from before the run: at most $worst at one moment, $final once it was over"
# A run the user asks for says so at once.
$base = [UX]::Snap($fx, $fy, $fw, $fh)
[UX]::WatchStart($base, $fx, $fy, $fw, $fh, 40, "$PfShots\d1-footer-asked.png")
Pf fence refresh $sid | Out-Null
Start-Sleep -Milliseconds 1500
$asked = [UX]::WatchStop()
Check 'a run the user asks for shows that it runs' ($asked -gt 60) "most pixels that differed: $asked"
Pf fence delete $sid | Out-Null
}

if ($Part -match 'E') {
Case 'E1 the clock menu says whether the seconds show'
$cid = NewTool '时钟' '100,100,240,140' 'clock' ''
Pf settings set tools.clockSeconds false | Out-Null
Start-Sleep -Seconds 1
$w = DumpOf '时钟'
[G]::RClick($w.l + 120, $w.t + 14); Start-Sleep -Milliseconds 800
$off = MenuTexts (MenuWin 5)
CloseMenus
Pf settings set tools.clockSeconds true | Out-Null
Start-Sleep -Milliseconds 800
[G]::RClick($w.l + 120, $w.t + 14); Start-Sleep -Milliseconds 800
$m = MenuWin 5
$on = MenuTexts $m
Crop 'e1-menu-seconds-on' ($w.l - 10) ($w.t - 10) 420 260 | Out-Null
$picked = MenuPick '^不显示秒' $m
Start-Sleep -Milliseconds 800
# A menu left open would keep the CLI waiting.
CloseMenus
$after = (PfJson settings get).tools.clockSeconds
Check 'without the seconds the entry reads 显示秒' ($off -match '(^| )显示秒( |$)') "menu: $off"
Check 'with the seconds on the entry reads 不显示秒' ($on -match '不显示秒') "menu: $on"
Check 'picking it takes the seconds away' ($picked -and $after -eq $false) "picked '$picked', tools.clockSeconds=$after"
Case 'E4 a clock too narrow for its seconds'
# 86 px: the narrowest a title 「时钟」 lets the mouse make it.
Pf settings set tools.clockSeconds true | Out-Null
Pf fence move $cid --rect '100,100,86,150' | Out-Null
Start-Sleep -Milliseconds 1500
$w = DumpOf '时钟'
$c = CRect $w
# The time's line, from the text's left edge to 4 px before the fence's right edge (its rim).
$ink = [UX]::Ink($w.l + 16, $c[1] + 2, $w.r - $w.l - 20, 30, 50) -split ' '
$gap = ($w.r - $w.l - 20) - 1 - [int]$ink[1]
FCrop 'e4-narrow-clock' $w
Check 'in an 86 px wide clock the time ends before the edge' ([int]$ink[0] -gt 20 -and $gap -ge 8) "text pixels $($ink[0]), room right of the last one $gap px; time form: $(DumpVal $w 'clock_time')"
Pf fence move $cid --rect '100,100,240,140' | Out-Null
Start-Sleep -Milliseconds 1200
$w = DumpOf '时钟'
$wide = [UX]::Ink($w.l + 16, $w.t + 38, 200, 50, 50) -split ' '
Check 'at 240 px the seconds are back' ([int]$wide[1] -ge 100) "the time's last text pixel is $($wide[1]) px from its left edge; time form: $(DumpVal $w 'clock_time')"
# A 12 hour format without seconds: at 86 px the hours and minutes stay whole, the AM / PM goes.
Pf settings set tools.clockSeconds false | Out-Null
Set-Culture en-US
[UX]::Intl()
Pf fence move $cid --rect '100,100,86,150' | Out-Null
Start-Sleep -Milliseconds 1500
$w = DumpOf '时钟'
$ink = [UX]::Ink($w.l + 16, $c[1] + 2, $w.r - $w.l - 20, 30, 50) -split ' '
$gap = ($w.r - $w.l - 20) - 1 - [int]$ink[1]
FCrop 'e4-narrow-clock-12h' $w
Check 'in a 12 hour format the 86 px wide clock keeps its minutes whole' ([int]$ink[0] -gt 20 -and $gap -ge 8) "text pixels $($ink[0]), room right of the last one $gap px; time form: $(DumpVal $w 'clock_time')"
Set-Culture zh-CN
[UX]::Intl()
Start-Sleep -Milliseconds 600
Pf settings set tools.clockSeconds false | Out-Null
Pf fence delete $cid | Out-Null

Case 'E2 the footer of a narrow script fence'
$sid = NewTool '状态面板' '420,100,300,230' 'script' "$sd\状态面板.ps1"
Start-Sleep -Seconds 4
ParkMouse 700 700
Pf fence move $sid --rect '420,100,120,230' | Out-Null
Start-Sleep -Milliseconds 1200
$w = DumpOf '状态面板'
# The footer's line, from the text's left edge to the fence's right edge.
$ink = [UX]::Ink($w.l + 16, $w.b - 28, $w.r - $w.l - 16, 22, 40) -split ' '
$gap = ($w.r - $w.l - 16) - 1 - [int]$ink[1]
FCrop 'e2-narrow-footer' $w
$form = @(PfLogAll | Where-Object { $_ -match 'script footer: the form that fits' -and $_ -match 'form="?short' }).Count
Check 'the footer of a 120 px wide script fence ends before the edge' ([int]$ink[0] -gt 20 -and $gap -ge 8) "text pixels $($ink[0]), room right of the last one $gap px; log lines for the short form: $form"
Pf fence delete $sid | Out-Null

Case 'E3 the first run of a slow script'
[IO.File]::WriteAllText("$sd\六秒.ps1", "Start-Sleep -Seconds 6`r`nWrite-Output '完成'`r`n", $utf8)
$slow = NewTool '六秒' '420,100,300,160' 'script' "$sd\六秒.ps1"
Start-Sleep -Milliseconds 2500
$w = DumpOf '六秒'
$c = CRect $w
# The body: under the title row, above the footer's line.
$body = [UX]::Ink($c[0] + 10, $c[1] + 2, $c[2] - 20, $c[3] - 40, 40) -split ' '
FCrop 'e3-first-run' $w
Check 'while the first run goes on the body says so' ([int]$body[0] -ge 60) "text pixels in the body 2.5 s after the fence was made: $($body[0])"
Start-Sleep -Seconds 6
Pf fence delete $slow | Out-Null
}

if ($Part -match 'R') {
Case 'R recordings'
$nid = NewTool '便签' '100,100,280,200' 'note' "买菜`n[ ] 西红柿`n[ ] 鸡蛋"
$sid = NewTool '状态面板' '420,100,300,230' 'script' "$sd\状态面板.ps1"
Start-Sleep -Seconds 4
ParkMouse 700 700
$w = DumpOf '便签'
$c = CRect $w
$b1 = (Rects $w 'note_boxes')[1]; $b2 = (Rects $w 'note_boxes')[2]
$bx = $c[0] + [int](($b1[0] + $b1[2]) / 2); $by = $c[1] + [int](($b1[1] + $b1[3]) / 2)
RecStart 'box' -X 90 -Y 90 -W 300 -H 160 -Fps 30
RecMark '指针移到待办的方框上'
Glide ($bx + 120) ($by + 70) $bx $by 10 25
Start-Sleep -Milliseconds 900
RecMark '按住鼠标左键不放'
MDown; Start-Sleep -Milliseconds 1200
RecMark '松开'
MUp; Start-Sleep -Milliseconds 1200
RecStop
[G]::Click($bx, $by); Start-Sleep -Milliseconds 600
ParkMouse 700 700
RecStart 'enter' -X 90 -Y 90 -W 300 -H 220 -Fps 15
RecMark '在最后一条待办后面按回车，输入「牛奶」'
[G]::Click($c[0] + [int]$b2[2] + 40, $c[1] + [int](($b2[1] + $b2[3]) / 2)); Start-Sleep -Milliseconds 700
[G]::Press(0x23); [G]::Press(0x0D); Start-Sleep -Milliseconds 500
[G]::Type('牛奶'); Start-Sleep -Milliseconds 900
RecMark '再按两次回车，输入「备注」'
[G]::Press(0x0D); Start-Sleep -Milliseconds 600
[G]::Press(0x0D); Start-Sleep -Milliseconds 600
[G]::Type('备注'); Start-Sleep -Milliseconds 900
RecMark '按 Esc'
[G]::Press(0x1B); Start-Sleep -Milliseconds 1000
RecStop
$ws = DumpOf '状态面板'
$cs = CRect $ws
$r1 = (Rects $ws 'script_rows')[1]; $r2 = (Rects $ws 'script_rows')[2]
$py1 = $cs[1] + [int](($r1[1] + $r1[3]) / 2); $py2 = $cs[1] + [int](($r2[1] + $r2[3]) / 2)
Set-Clipboard -Value 'before'
RecStart 'row' -X 410 -Y 90 -W 320 -H 250 -Fps 30
RecMark '指针从上往下滑过各行'
Glide ($cs[0] + 150) ($cs[1] - 20) ($cs[0] + 150) ($py2) 30 30
Start-Sleep -Milliseconds 700
RecMark '在「复制服务器地址」上按住鼠标左键不放'
MDown; Start-Sleep -Milliseconds 1300
RecMark '松开'
MUp; Start-Sleep -Milliseconds 1500
RecStop
ParkMouse 700 700
Start-Process notepad
Start-Sleep -Seconds 2
$np = (Get-Process notepad | Select-Object -First 1).MainWindowHandle
[L]::MoveWindow($np, 60, 60, 760, 420, $true) | Out-Null
Start-Sleep -Milliseconds 800
RecStart 'peek' -X 40 -Y 40 -W 800 -H 460 -Fps 15
RecMark '按 Ctrl+Alt+Space 速览'
PeekKey
Start-Sleep -Milliseconds 1500
RecMark '单击便签的文字，输入「速览里记一笔」'
[G]::Click($c[0] + 60, $c[1] + 12); Start-Sleep -Milliseconds 900
[G]::Type('速览里记一笔'); Start-Sleep -Milliseconds 1500
RecStop
[G]::Press(0x1B); Start-Sleep -Milliseconds 500
if (PeekOn) { PeekKey; Start-Sleep -Milliseconds 600 }
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
Pf fence delete $nid | Out-Null
Pf fence delete $sid | Out-Null
}

Case 'cleanup'
# The boxes' own culture.
Set-Culture en-US
[UX]::Intl()
[Anim]::Set($Anim0)
