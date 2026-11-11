# Fence polish round 2026-11-12, whole-fence motion. Parts (run.sh --var Part=...):
#   A  a fence rolled up by a click stays rolled under the resting pointer (the hover peek waits
#      until the pointer has left), by the chevron, a double-click on the title, the fence menu
#      and with 单击标题时展开
#   B  an open fence stops where the fences it pushes can go no lower (the push overlap open
#      since 0.3.1): by click and CLI, with one and two fences below, auto height, a restart;
#      a hover peek still opens in full above them (#197)
#   C  a fence let go a little over another one glides to its landing place
# Default: all parts. Runs on a 1600x900 box at 100 %.
if (-not $Part) { $Part = 'ABC' }
trap { PfTrap $_; continue }
. C:\pf\kit\pixels.ps1

# Every size of one window, about every 2 ms: "ms left top right bottom" per change.
Add-Type -TypeDefinition @'
using System; using System.Collections.Generic; using System.Diagnostics; using System.Runtime.InteropServices; using System.Threading;
public static class PmTrack {
  [StructLayout(LayoutKind.Sequential)] struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] struct POINT { public int X, Y; }
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] static extern bool GetCursorPos(out POINT p);
  [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(POINT p);
  [DllImport("user32.dll")] static extern IntPtr GetAncestor(IntPtr h, uint f);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowTextW(IntPtr h, System.Text.StringBuilder s, int n);
  // The pointer's place, and the title of the top-level window at a screen point.
  public static int[] Cursor() { POINT p; GetCursorPos(out p); return new[] { p.X, p.Y }; }
  public static string TitleAt(int x, int y) { POINT p; p.X = x; p.Y = y; var h = GetAncestor(WindowFromPoint(p), 2); var s = new System.Text.StringBuilder(256); GetWindowTextW(h, s, 256); return s.ToString(); }
  static Thread t; static volatile bool on; static List<string> rows; static Stopwatch sw;
  public static void Start(IntPtr h) {
    if (on) Stop();
    rows = new List<string>(); sw = Stopwatch.StartNew(); on = true;
    t = new Thread(() => {
      string last = null;
      while (on) {
        RECT r;
        if (GetWindowRect(h, out r)) {
          string v = r.L + " " + r.T + " " + r.R + " " + r.B;
          if (v != last) { last = v; lock (rows) rows.Add(sw.ElapsedMilliseconds + " " + v); }
        }
        Thread.Sleep(2);
      }
    });
    t.IsBackground = true; t.Start();
  }
  public static long Now() { return sw == null ? 0 : sw.ElapsedMilliseconds; }
  public static string[] Stop() { on = false; if (t != null) t.Join(500); lock (rows) return rows.ToArray(); }
}
'@

$PmFiles = @('季度报告 2026 第三季度.docx', '预算表.xlsx', '会议纪要（修订版）.txt', '读书笔记.md', '发票.pdf', '旅行计划.txt',
  'a.txt', '很长很长的文件名用来测试标签折行的效果.txt', '合同.docx', '照片说明.txt', '清单.txt', '日程.txt')
function PmClean() { foreach ($n in $PmFiles) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue } }
function Park() { ParkMouse 1450 300; Start-Sleep -Milliseconds 400 }
function Hw($t) { return [G]::FindWindowW('PecoFence.Fence', $t) }
function Win($t) { return (Rect (Hw $t)) }
function WinH($t) { $r = Win $t; return $r.B - $r.T }
function TitlePt($t) { $r = Win $t; return @(($r.L + 100), ($r.T + 18)) }
function ChevronPt($t) { $r = Win $t; return @(($r.R - 14), ($r.T + 18)) }
# The pointer comes to rest on the title (two moves: the second one is what a hand does).
function HoverTitle($t) { $p = TitlePt $t; [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 300; [G]::MoveTo($p[0] + 2, $p[1]) }
function Rows($rows) { return @($rows | ForEach-Object { $p = $_ -split ' '; [pscustomobject]@{ ms = [int]$p[0]; L = [int]$p[1]; T = [int]$p[2]; R = [int]$p[3]; B = [int]$p[4] } }) }

Case 'setup'
CloseWindows
PmClean
foreach ($n in $PmFiles) { Set-Content -LiteralPath (Join-Path $PfDesk $n) 'polish' }
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
# An RDP session shows only an outline while a window is dragged: show the window itself.
[G]::SystemParametersInfoW(0x25, 1, [NullString]::Value, 3) | Out-Null
PfStart -Script @('sleep 3600000')
ClearFences | Out-Null
$WorkB = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea.Bottom
$TitleH = 36

if ($Part -match 'A') {
  # ---- a click that rolls the fence up is not undone by the hover peek under the resting pointer
  Pf fence create --title Roll1 --rect '300,200,300,260' | Out-Null
  Pf item move "$PfDesk\预算表.xlsx" "$PfDesk\发票.pdf" "$PfDesk\a.txt" --to Roll1 | Out-Null
  Start-Sleep -Seconds 1; Park
  # What the pointer does after the click decides: it stays where it clicked.
  function StaysRolled($what) {
    Start-Sleep -Milliseconds 1700
    $h = WinH Roll1
    Check "$what`: the fence stays rolled up under the resting pointer" ($h -le $TitleH + 2 -and (FenceGet Roll1).rolledUp) "window height $h 1.7 s after the click, rolledUp=$((FenceGet Roll1).rolledUp)"
  }
  function PeekWorksAgain($what) {
    Park; Start-Sleep -Milliseconds 500
    HoverTitle Roll1; Start-Sleep -Milliseconds 1300
    $h = WinH Roll1
    Check "$what`: after the pointer left and came back the hover peek opens" ($h -gt 200) "window height $h"
    Park; Start-Sleep -Milliseconds 1100
    Check "$what`: the hover peek closes when the pointer leaves" ((WinH Roll1) -le $TitleH + 2) "window height $(WinH Roll1)"
  }
  Case 'A chevron click'
  $p = ChevronPt Roll1; [G]::Click($p[0], $p[1])
  StaysRolled 'chevron click'
  Shot 'A-chevron-1700ms'
  PeekWorksAgain 'chevron click'
  # A hover peek that closed by itself blocks nothing: back on the title it opens again.
  HoverTitle Roll1; Start-Sleep -Milliseconds 1300
  Check 'a hover peek that closed by itself opens again on the next hover' ((WinH Roll1) -gt 200) "window height $(WinH Roll1)"
  Park; Start-Sleep -Milliseconds 1100

  Case 'A double-click on the title'
  Pf fence unroll Roll1 | Out-Null; Start-Sleep -Milliseconds 900
  $p = TitlePt Roll1; DblClick $p[0] $p[1]
  StaysRolled 'title double-click'
  PeekWorksAgain 'title double-click'

  Case 'A fence menu'
  Pf fence unroll Roll1 | Out-Null; Start-Sleep -Milliseconds 900
  $p = TitlePt Roll1; [G]::RClick($p[0], $p[1]); Start-Sleep -Milliseconds 600
  $picked = MenuPick '^Roll up'
  Start-Sleep -Milliseconds 400
  $c = [PmTrack]::Cursor()
  $r = Win Roll1
  $onRow = ($c[0] -ge $r.L -and $c[0] -lt $r.R -and $c[1] -ge $r.T -and $c[1] -lt $r.B)
  Log "menu pick '$picked', pointer $($c[0]),$($c[1]), rolled window $($r.L),$($r.T),$($r.R),$($r.B), on the row: $onRow"
  StaysRolled 'Roll up from the fence menu'
  PeekWorksAgain 'Roll up from the fence menu'

  Case 'A rolled by the CLI with the pointer away'
  Pf fence unroll Roll1 | Out-Null; Start-Sleep -Milliseconds 900; Park
  Pf fence roll Roll1 | Out-Null; Start-Sleep -Milliseconds 100
  HoverTitle Roll1; Start-Sleep -Milliseconds 1400
  Check 'rolled by the CLI, then the pointer arrives during the roll: the hover peek opens' ((WinH Roll1) -gt 200) "window height $(WinH Roll1)"
  Park; Start-Sleep -Milliseconds 1100

  Case 'A hover peek off'
  Pf settings set rollUp.hoverPeek false | Out-Null
  Pf fence unroll Roll1 | Out-Null; Start-Sleep -Milliseconds 900
  $p = ChevronPt Roll1; [G]::Click($p[0], $p[1])
  StaysRolled 'chevron click, hover peek off'
  Park
  Pf settings set rollUp.hoverPeek true | Out-Null

  Case 'A click on the title expands'
  Pf settings set rollUp.clickToExpand true | Out-Null
  Pf fence unroll Roll1 | Out-Null; Start-Sleep -Milliseconds 900; Park
  $p = TitlePt Roll1; [G]::Click($p[0], $p[1])
  StaysRolled '单击标题时展开, title click'
  $p = TitlePt Roll1; [G]::MoveTo($p[0] + 30, $p[1]); Start-Sleep -Milliseconds 200; [G]::Click($p[0] + 30, $p[1]); Start-Sleep -Milliseconds 900
  Check '单击标题时展开: the next click opens it' ((WinH Roll1) -gt 200 -and -not (FenceGet Roll1).rolledUp) "window height $(WinH Roll1)"
  Pf settings set rollUp.clickToExpand false | Out-Null
  Park
  Pf fence delete Roll1 | Out-Null
}

if ($Part -match 'B') {
  # ---- an open fence stops where the fences it pushes can go no lower
  function Ov($p, $q) { return [math]::Max(0, [math]::Min($p.B, $q.B) - [math]::Max($p.T, $q.T)) }
  function State($names) { return (($names | ForEach-Object { $r = Win $_; "$_ $($r.T)..$($r.B)" }) -join ', ') }
  # The stack is clean: no two of the fences overlap and none ends under the work area.
  function Clean($what, $names) {
    $bad = @()
    for ($i = 0; $i -lt $names.Count; $i++) {
      $p = Win $names[$i]
      if ($p.B -gt $WorkB) { $bad += "$($names[$i]) ends at $($p.B), under the work area ($WorkB)" }
      for ($j = $i + 1; $j -lt $names.Count; $j++) {
        $o = Ov $p (Win $names[$j])
        if ($o -gt 0) { $bad += "$($names[$i]) and $($names[$j]) overlap by $o px" }
      }
    }
    Check "$what`: no fence covers another, all above the taskbar" ($bad.Count -eq 0) "$(State $names); $($bad -join '; ')"
  }
  # Pusher's title row at 440 opens to 300 px; Pushed (280 px) rests 12 px under the row, so it
  # can go down 84 px to the taskbar: 124 px of room for Pusher.
  Pf fence create --title Pusher --rect '1000,440,300,300' | Out-Null
  Pf item move "$PfDesk\预算表.xlsx" "$PfDesk\发票.pdf" "$PfDesk\a.txt" "$PfDesk\合同.docx" "$PfDesk\清单.txt" "$PfDesk\日程.txt" --to Pusher | Out-Null
  Pf fence roll Pusher | Out-Null; Start-Sleep -Milliseconds 900
  Pf fence create --title Pushed --rect '1000,488,300,280' | Out-Null
  Pf item move "$PfDesk\读书笔记.md" "$PfDesk\旅行计划.txt" --to Pushed | Out-Null
  Start-Sleep -Seconds 1; Park
  $room = $WorkB - 280 - 8 - 440

  Case 'B opened by a click'
  $p = ChevronPt Pusher; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 1300; Park
  Shot 'B-click-open'
  Clean 'opened by a click' @('Pusher', 'Pushed')
  $f = FenceGet Pusher
  Check 'opened by a click: Pusher is as tall as the room, its saved height stays' ((WinH Pusher) -eq $room -and $f.rect.h -eq 300 -and -not $f.rolledUp) "window $(WinH Pusher) px (room $room), saved rect h $($f.rect.h), rolledUp=$($f.rolledUp)"
  # The rest of the icons scroll: the wheel moves the content.
  $w = Win Pusher
  $before = [Px]::Grab($w.L + 8, $w.T + 40, 280, 70)
  Wheel ($w.L + 150) ($w.T + 80) -2; Start-Sleep -Milliseconds 700
  $after = [Px]::Grab($w.L + 8, $w.T + 40, 280, 70)
  $moved = [Px]::Diff($before, $after, 0, 279, 0, 69)
  Check 'opened by a click: the icons that do not fit scroll' ($moved -gt 2) "mean brightness change of the content after two wheel notches: $([math]::Round($moved, 1))"
  $p = ChevronPt Pusher; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900; Park
  $q = Win Pushed
  Check 'rolled up again: Pushed is back at its place' ($q.T -eq 488) (State @('Pusher', 'Pushed'))

  Case 'B opened by the CLI'
  Pf fence unroll Pusher | Out-Null; Start-Sleep -Milliseconds 1300
  Clean 'opened by the CLI' @('Pusher', 'Pushed')
  Check 'opened by the CLI: as tall as the room, saved height kept' ((WinH Pusher) -eq $room -and (FenceGet Pusher).rect.h -eq 300) "window $(WinH Pusher) px, saved rect h $((FenceGet Pusher).rect.h)"

  Case 'B a move keeps the full height'
  # Dragged by its title to a place with nothing under it: it opens in full there.
  $p = TitlePt Pusher
  DragTo $p[0] $p[1] ($p[0] - 500) ($p[1] - 300) 16
  Start-Sleep -Milliseconds 900; Park
  $f = FenceGet Pusher
  Check 'moved to a place with room: the fence is its full height again' ((WinH Pusher) -eq 300 -and $f.rect.h -eq 300) "window $(WinH Pusher) px, saved rect h $($f.rect.h), at $($f.windowRect.x),$($f.windowRect.y)"
  Pf fence roll Pusher | Out-Null; Start-Sleep -Milliseconds 700
  Pf fence move Pusher --rect '1000,440,300,300' | Out-Null; Start-Sleep -Milliseconds 900

  Case 'B the fence below makes room'
  Pf fence unroll Pusher | Out-Null; Start-Sleep -Milliseconds 1300
  $held = WinH Pusher
  # The fence below in the user's hands (dragged a little and put back): the open fence above
  # waits, it neither grows over it nor shrinks back afterwards.
  [PmTrack]::Start((Hw Pusher))
  $p = TitlePt Pushed
  [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 150; MDown; Start-Sleep -Milliseconds 200
  Glide $p[0] $p[1] ($p[0] + 30) $p[1] 6 40; Start-Sleep -Milliseconds 500
  Glide ($p[0] + 30) $p[1] $p[0] $p[1] 6 40; Start-Sleep -Milliseconds 300
  MUp; Start-Sleep -Milliseconds 900
  $rows = @(Rows ([PmTrack]::Stop()))
  Park
  Check 'the fence below dragged a little and put back: the open fence above waits' ($rows.Count -eq 1 -and (WinH Pusher) -eq $room) "Pusher's heights meanwhile: $(($rows | ForEach-Object { "+$($_.ms):$($_.B - $_.T)" }) -join ' ')"
  Clean 'after the fence below was dragged and put back' @('Pusher', 'Pushed')
  Pf fence move Pushed --rect '600,488,300,280' | Out-Null; Start-Sleep -Milliseconds 1200
  Check 'the fence below is moved away: the open fence grows to its full height' ($held -eq $room -and (WinH Pusher) -eq 300) "held at $held px, then $(WinH Pusher) px"
  Pf fence roll Pusher | Out-Null; Start-Sleep -Milliseconds 800
  Pf fence move Pushed --rect '1000,488,300,280' | Out-Null; Start-Sleep -Milliseconds 900

  Case 'B a restart'
  Pf fence unroll Pusher | Out-Null; Start-Sleep -Milliseconds 1500
  PfQuit; PfStart -Script @('sleep 3600000'); Start-Sleep -Seconds 2
  Clean 'saved open, after a restart' @('Pusher', 'Pushed')
  $f = FenceGet Pusher
  Check 'after a restart: as tall as the room, saved height kept' ((WinH Pusher) -eq $room -and $f.rect.h -eq 300) "window $(WinH Pusher) px, saved rect h $($f.rect.h)"
  Pf fence roll Pusher | Out-Null; Start-Sleep -Milliseconds 800

  Case 'B a hover peek opens in full, on top'
  Park
  HoverTitle Pusher; Start-Sleep -Milliseconds 1300
  $w = Win Pusher; $q = Win Pushed
  $top = [PmTrack]::TitleAt($q.L + 150, $q.T + 10)
  Shot 'B-hover-peek'
  Check 'a hover peek still opens in full, above the fence it cannot push' ((WinH Pusher) -eq 300 -and $top -eq 'Pusher') "Pusher $($w.T)..$($w.B), Pushed $($q.T)..$($q.B), on top at Pushed's title: '$top'"
  Park; Start-Sleep -Milliseconds 1200
  Check 'the hover peek closed: Pushed is back at its place' ((Win Pushed).T -eq 488 -and (WinH Pusher) -le $TitleH + 2) (State @('Pusher', 'Pushed'))

  Case 'B a hover peek kept open by a click'
  # The chevron of a fence that is hover-peeking keeps it open: from then on it rests among the
  # others, so it gets as short as the room.
  HoverTitle Pusher; Start-Sleep -Milliseconds 1300
  $p = ChevronPt Pusher; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 1300; Park
  Clean 'a hover peek kept open by a click' @('Pusher', 'Pushed')
  $f = FenceGet Pusher
  Check 'a hover peek kept open by a click: as tall as the room, saved height kept' ((WinH Pusher) -eq $room -and $f.rect.h -eq 300 -and -not $f.rolledUp) "window $(WinH Pusher) px, saved rect h $($f.rect.h), rolledUp=$($f.rolledUp)"

  Case 'B a new height by hand'
  # The bottom edge dragged: the height the drag ends with is the user's own and is saved (here
  # the smallest a fence can be made by hand, one row of icons); the window still stops at the
  # room.
  $w = Win Pusher
  DragTo ($w.L + 150) ($w.B - 2) ($w.L + 150) ($w.B - 42) 10
  Start-Sleep -Milliseconds 900; Park
  $f = FenceGet Pusher
  Check 'the bottom edge dragged: the height it ended with is saved' ($f.rect.h -lt 300 -and $f.rect.h -ge (WinH Pusher) -and (WinH Pusher) -le $room) "window $(WinH Pusher) px, saved rect h $($f.rect.h)"
  Clean 'after the bottom edge was dragged' @('Pusher', 'Pushed')
  Pf fence roll Pusher | Out-Null; Start-Sleep -Milliseconds 800
  Pf fence move Pusher --rect '1000,440,300,300' | Out-Null; Start-Sleep -Milliseconds 900
  $f = FenceGet Pusher
  Log "after the resize by hand and a move back: saved rect $($f.rect.x),$($f.rect.y),$($f.rect.w),$($f.rect.h) rolledUp=$($f.rolledUp)"

  Case 'B whole rows and a view switch'
  # 「调整大小时保持整行整列」 on, the view switched while the fence is held short: the rows are
  # counted for its full height, which stays saved and comes back when there is room.
  # Pushed 14 px taller: the room (110 px) is no whole number of list rows, so rows counted from
  # the window would change its height.
  Pf fence move Pushed --rect '1000,488,300,294' | Out-Null; Start-Sleep -Milliseconds 900
  Pf fence unroll Pusher | Out-Null; Start-Sleep -Milliseconds 1300
  $room2 = $WorkB - 294 - 8 - 440
  $h0 = (FenceGet Pusher).rect.h; $w0 = WinH Pusher
  Pf settings set snapping.sizeToCells true | Out-Null
  Pf fence set Pusher layout list | Out-Null; Start-Sleep -Milliseconds 900
  $f = FenceGet Pusher
  Check 'whole rows on, the view switched while held short: the full height is still the saved one' ($w0 -eq $room2 -and $f.rect.h -ge 250 -and (WinH Pusher) -le $room2) "window $w0 px (room $room2), after the switch $(WinH Pusher) px; saved rect h $h0, after the switch $($f.rect.h)"
  Clean 'after the view switch' @('Pusher', 'Pushed')
  Pf fence move Pushed --rect '600,488,300,280' | Out-Null; Start-Sleep -Milliseconds 1200
  $f = FenceGet Pusher
  Check 'and the fence has that height again when the fence below makes room' ((WinH Pusher) -ge 250 -and (WinH Pusher) -eq $f.rect.h) "window $(WinH Pusher) px, saved rect h $($f.rect.h)"
  Pf settings set snapping.sizeToCells false | Out-Null
  Pf fence set Pusher layout icons | Out-Null
  Pf fence roll Pusher | Out-Null; Start-Sleep -Milliseconds 800
  Pf fence move Pusher --rect '1000,440,300,300' | Out-Null

  Case 'B auto height'
  Pf fence set Pusher autoHeight true | Out-Null; Start-Sleep -Milliseconds 500
  Pf fence unroll Pusher | Out-Null; Start-Sleep -Milliseconds 1400
  Clean 'auto height, opened by the CLI' @('Pusher', 'Pushed')
  Check 'auto height: open, at least its title row and 40 px' ((WinH Pusher) -ge $TitleH + 40 -and -not (FenceGet Pusher).rolledUp) "window $(WinH Pusher) px"
  Pf fence roll Pusher | Out-Null; Start-Sleep -Milliseconds 800
  Pf fence set Pusher autoHeight false | Out-Null
  Pf fence delete Pusher | Out-Null; Pf fence delete Pushed | Out-Null; Start-Sleep -Milliseconds 600

  Case 'B two fences below'
  # Top's row at 300; Mid (150 px) 12 px under it, Low (280 px) 12 px under Mid. Low can go down
  # to 572, Mid to 414: 106 px of room for Top.
  Pf fence create --title Top --rect '1000,300,300,300' | Out-Null
  Pf item move "$PfDesk\预算表.xlsx" "$PfDesk\发票.pdf" "$PfDesk\a.txt" "$PfDesk\合同.docx" --to Top | Out-Null
  Pf fence roll Top | Out-Null; Start-Sleep -Milliseconds 900
  Pf fence create --title Mid --rect '1000,348,300,150' | Out-Null
  Pf fence create --title Low --rect '1000,510,300,280' | Out-Null
  Start-Sleep -Seconds 1; Park
  $p = ChevronPt Top; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 1300; Park
  Shot 'B-two-below'
  Clean 'two fences below, opened by a click' @('Top', 'Mid', 'Low')
  Check 'two fences below: Top is as tall as the room they leave' ((WinH Top) -eq ($WorkB - 280 - 8 - 150 - 8 - 300) -and (FenceGet Top).rect.h -eq 300) "window $(WinH Top) px, saved rect h $((FenceGet Top).rect.h)"
  $p = ChevronPt Top; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900; Park
  Check 'two fences below: rolled up again, both are back' ((Win Mid).T -eq 348 -and (Win Low).T -eq 510) (State @('Top', 'Mid', 'Low'))
  Pf fence delete Top | Out-Null; Pf fence delete Mid | Out-Null; Pf fence delete Low | Out-Null; Start-Sleep -Milliseconds 600
}

if ($Part -match 'C') {
  # ---- a fence let go a little over another one glides to its landing place
  Pf fence create --title Alpha --rect '300,200,300,250' | Out-Null
  Pf fence create --title Beta --rect '700,200,300,250' | Out-Null
  Start-Sleep -Seconds 1; Park
  # Twice in the Fluent style (the system's move loop), once in Liquid Glass, which moves the
  # window itself on a title drag.
  foreach ($k in 'Fluent 1', 'Fluent 2', 'Liquid Glass') {
    Case "C dropped 10 px over the other fence ($k)"
    if ($k -eq 'Liquid Glass') { Pf settings set themeStyle liquidGlass | Out-Null; Start-Sleep -Milliseconds 900 }
    Pf fence move Alpha --rect '300,200,300,250' | Out-Null; Start-Sleep -Milliseconds 700
    $p = TitlePt Alpha
    [PmTrack]::Start((Hw Alpha))
    DragTo $p[0] $p[1] ($p[0] + 110) $p[1] 16
    Start-Sleep -Milliseconds 300
    $rows = Rows ([PmTrack]::Stop())
    $end = $rows[-1]
    # The moves of the last 300 ms before the window came to rest: the trip from where it was
    # let go (the drag's last move is 400 ms older) to where it landed.
    $trip = @($rows | Where-Object { $_.ms -ge $end.ms - 300 })
    $from = @($rows | Where-Object { $_.ms -lt $end.ms - 300 })[-1]
    $f = FenceGet Alpha
    Log "drop $k`: let go at x $($from.L), landed at x $($end.L); steps: $(($trip | ForEach-Object { "+$($_.ms - $trip[0].ms):$($_.L)" }) -join ' ')"
    Check "dropped over Beta ($k): it lands clear of Beta and is saved there" ($end.R -le 700 -and $f.rect.x -eq $end.L -and $f.windowRect.x -eq $end.L) "landed at $($end.L)..$($end.R), saved x $($f.rect.x)"
    Check "dropped over Beta ($k): the window travels there" ($from.L -ne $end.L -and $trip.Count -ge 3 -and ($trip[-1].ms - $trip[0].ms) -ge 50 -and ($trip[-1].ms - $trip[0].ms) -le 300) "from x $($from.L) to $($end.L) in $($trip.Count) steps over $($trip[-1].ms - $trip[0].ms) ms"
  }
  Pf settings set themeStyle fluent | Out-Null; Start-Sleep -Milliseconds 900
  Case 'C animations off'
  [Anim]::Set($false); Start-Sleep -Milliseconds 800
  Pf fence move Alpha --rect '300,200,300,250' | Out-Null; Start-Sleep -Milliseconds 700
  $p = TitlePt Alpha
  [PmTrack]::Start((Hw Alpha))
  DragTo $p[0] $p[1] ($p[0] + 110) $p[1] 16
  Start-Sleep -Milliseconds 300
  $rows = Rows ([PmTrack]::Stop())
  $end = $rows[-1]
  $trip = @($rows | Where-Object { $_.ms -ge $end.ms - 300 })
  Check 'animations off: the fence is placed at once' ($end.R -le 700 -and $trip.Count -le 2) "landed at $($end.L)..$($end.R) in $($trip.Count) step(s)"
  [Anim]::Set($true); Start-Sleep -Milliseconds 800
  Pf fence delete Alpha | Out-Null; Pf fence delete Beta | Out-Null; Start-Sleep -Milliseconds 600
}

Case 'cleanup'
[Anim]::Set($AnimWas)
PfQuit
PmClean
