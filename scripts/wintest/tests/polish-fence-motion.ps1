# Fence polish round 2026-11-12, whole-fence motion. Parts (run.sh --var Part=...):
#   A  a fence rolled up by a click stays rolled under the resting pointer (the hover peek waits
#      until the pointer has left), by the chevron, a double-click on the title, the fence menu
#      and with 单击标题时展开
# Default: all parts. Runs on a 1600x900 box at 100 %.
if (-not $Part) { $Part = 'A' }
trap { PfTrap $_; continue }

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

Case 'cleanup'
[Anim]::Set($AnimWas)
PfQuit
PmClean
