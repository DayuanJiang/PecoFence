# Feedback #207 item 1: the top fence of closely stacked rolled fences opened by a click only to
# the gap before the next fence (with 「展开时把下面的栅栏推开」 off, one row over the next titles),
# and that short height was saved: the hover peek and every later open stayed short. Columns:
# A a short stack (the top fence and 4 rolled ones 8 px apart), B a stack down to the taskbar,
# C two fences with a gap that holds a row. With the push on and off, then the hover peek, the
# fence below moved away and a restart. Runs on a 1600x900 box at 100 %.
trap { PfTrap $_; continue }

Add-Type -TypeDefinition @'
using System; using System.Runtime.InteropServices;
public static class SeTop {
  [StructLayout(LayoutKind.Sequential)] struct POINT { public int X, Y; }
  [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(POINT p);
  [DllImport("user32.dll")] static extern IntPtr GetAncestor(IntPtr h, uint f);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowTextW(IntPtr h, System.Text.StringBuilder s, int n);
  public static string TitleAt(int x, int y) { POINT p; p.X = x; p.Y = y; var h = GetAncestor(WindowFromPoint(p), 2); var s = new System.Text.StringBuilder(256); GetWindowTextW(h, s, 256); return s.ToString(); }
}
'@

$WorkB = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea.Bottom
$TitleH = 36
$Step = $TitleH + 8
function Park() { ParkMouse 1500 120; Start-Sleep -Milliseconds 400 }
function Hw($t) { return [G]::FindWindowW('PecoFence.Fence', $t) }
function Win($t) { return (Rect (Hw $t)) }
function WinH($t) { $r = Win $t; return $r.B - $r.T }
function SavedH($t) { return (FenceGet $t).rect.h }
function TitlePt($t) { $r = Win $t; return @(($r.L + 100), ($r.T + 18)) }
function ChevronPt($t) { $r = Win $t; return @(($r.R - 14), ($r.T + 18)) }
function HoverTitle($t) { $p = TitlePt $t; [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 300; [G]::MoveTo($p[0] + 2, $p[1]) }
function Tops($names) { return (($names | ForEach-Object { "$_@$((Win $_).T)" }) -join ' ') }
function Chevron($t) { $p = ChevronPt $t; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 1500; Park }

$names = @()
for ($i = 1; $i -le 18; $i++) { $n = "se$i.txt"; Set-Content -LiteralPath (Join-Path $PfDesk $n) -Value $n; $names += $n }

# A column of rolled fences 8 px apart; the first one is 300 px tall when open, with 6 items.
function Column($prefix, $x, $count, $files) {
  $all = @()
  for ($k = 0; $k -lt $count; $k++) {
    $t = "$prefix$k"
    $h = if ($k -eq 0) { 300 } else { 200 }
    Pf fence create --title $t --rect "$x,$(40 + $k * $Step),300,$h" | Out-Null
    if ($k -eq 0) { Pf item move ($files | ForEach-Object { Join-Path $PfDesk $_ }) --to $t | Out-Null }
    Pf fence roll $t | Out-Null; Start-Sleep -Milliseconds 500
    $all += $t
  }
  return $all
}

Case 'setup'
CloseWindows
PfStart -Script @('sleep 3600000'); Start-Sleep -Seconds 2
ClearFences | Out-Null
$a = Column 'A' 40 5 $names[0..5]
# Column B down to the taskbar: the lower fences are made 120 px tall so that they open above it.
$n = [math]::Floor(($WorkB - 40 - 120) / $Step) + 1
$b = Column 'B' 1000 $n $names[6..11]
# Column C: C1 rests 200 px under C0's title row, a gap that holds a row of icons.
Pf fence create --title C0 --rect '500,40,300,300' | Out-Null
Pf item move ($names[12..17] | ForEach-Object { Join-Path $PfDesk $_ }) --to C0 | Out-Null
Pf fence roll C0 | Out-Null; Start-Sleep -Milliseconds 500
Pf fence create --title C1 --rect '500,240,300,200' | Out-Null
Pf fence roll C1 | Out-Null; Start-Sleep -Milliseconds 500
$c = @('C0', 'C1')
Start-Sleep -Seconds 1; Park
Log "work bottom $WorkB, column B has $n fences; A: $(Tops $a); B: $(Tops $b); C: $(Tops $c)"
Shot 'stacks'

# Opened by the chevron: how tall, what is on top under the title row, and the saved height.
function OpenByClick($what, $col, [int]$want, [switch]$Covers) {
  $top = $col[0]; $second = $col[1]
  $before = (Win $second).T
  Chevron $top
  $r = Win $top
  $over = [SeTop]::TitleAt($r.L + 150, [math]::Min($r.B - 4, $r.T + $TitleH + 30))
  Shot "$what"
  Check "$what`: opens to $want px" ([math]::Abs((WinH $top) - $want) -le 2) "window $(WinH $top) px of 300 (title $TitleH); $second top $before -> $((Win $second).T); on top just under its title row: '$over'"
  Check "$what`: nothing covers the opened fence" ($over -eq $top) "on top just under its title row: '$over'"
  Check "$what`: the saved height stays 300 px" ((SavedH $top) -eq 300) "saved rect h $(SavedH $top)"
  Chevron $top
  Check "$what`: rolled up again, still 300 px saved" ((WinH $top) -le $TitleH + 2 -and (SavedH $top) -eq 300) "window $(WinH $top) px, saved rect h $(SavedH $top)"
}

function Hover($what, $col) {
  $top = $col[0]
  HoverTitle $top; Start-Sleep -Milliseconds 1400
  $r = Win $top
  $over = [SeTop]::TitleAt($r.L + 150, $r.T + $TitleH + 30)
  Shot "$what"
  Check "$what`: the hover peek opens in full, on top" ((WinH $top) -ge 298 -and $over -eq $top) "window $(WinH $top) px; on top under its title row: '$over'"
  Park; Start-Sleep -Milliseconds 1200
}

foreach ($push in 'true', 'false') {
  Pf settings set rollUp.pushNeighbors $push | Out-Null; Start-Sleep -Milliseconds 800
  Case "push $push"
  OpenByClick "A push=$push click" $a 300
  # The push on stops the top of a full column where the fences below reach the taskbar.
  if ($push -eq 'true') {
    Chevron $b[0]; $h = WinH $b[0]; Shot "B push=true click"
    Check 'B push=true click: opens as far as the fences below can slide, saved 300 px' ($h -gt $TitleH + 80 -and $h -le 300 -and (SavedH $b[0]) -eq 300) "window $h px, saved rect h $(SavedH $b[0])"
    Chevron $b[0]
  } else {
    OpenByClick "B push=$push click" $b 300
  }
  Hover "B push=$push hover" $b
  # C: with the push off it stops 8 px short of C1, only for this opening.
  $cWant = if ($push -eq 'true') { 300 } else { 240 - 8 - 40 }
  OpenByClick "C push=$push click" $c $cWant
  Hover "C push=$push hover" $c
}

Case 'push off: the fence below goes away, and a restart'
Chevron C0
Check 'C held short' ((WinH C0) -eq 192) "window $(WinH C0) px"
Pf fence move C1 --rect '1350,600,200,200' | Out-Null; Start-Sleep -Milliseconds 1500
Check 'the fence below moved away: C0 opens to its saved 300 px' ((WinH C0) -eq 300 -and (SavedH C0) -eq 300) "window $(WinH C0) px, saved rect h $(SavedH C0)"
Chevron C0
Pf fence move C1 --rect '500,240,300,200' | Out-Null; Start-Sleep -Milliseconds 1200
Chevron C0
Check 'the fence below back under it: C0 held short again' ((WinH C0) -eq 192 -and (SavedH C0) -eq 300) "window $(WinH C0) px, saved rect h $(SavedH C0); C1 at $((Win C1).T)"
PfQuit; PfStart -Script @('sleep 3600000'); Start-Sleep -Seconds 3; Park
Shot 'after-restart'
Check 'after a restart: C0 open and held short, C1 not moved, 300 px saved' ((WinH C0) -eq 192 -and (Win C1).T -eq 240 -and (SavedH C0) -eq 300) "C0 window $(WinH C0) px, saved rect h $(SavedH C0); C1 at $((Win C1).T); A: $(Tops $a)"
Pf settings set rollUp.pushNeighbors true | Out-Null
PfQuit
foreach ($n in $names) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
