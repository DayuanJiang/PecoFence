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
  [StructLayout(LayoutKind.Sequential)] struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
  // The heights of two windows about every 5 ms: "ms hA hB" per change.
  static System.Threading.Thread th; static volatile bool on; static System.Collections.Generic.List<string> rows;
  public static void Track(IntPtr a, IntPtr b) {
    rows = new System.Collections.Generic.List<string>(); on = true; var sw = System.Diagnostics.Stopwatch.StartNew();
    th = new System.Threading.Thread(() => { string last = null; while (on) { RECT ra, rb; GetWindowRect(a, out ra); GetWindowRect(b, out rb); string v = (ra.B - ra.T) + " " + (rb.B - rb.T); if (v != last) { last = v; lock (rows) rows.Add(sw.ElapsedMilliseconds + " " + v); } System.Threading.Thread.Sleep(5); } });
    th.IsBackground = true; th.Start();
  }
  public static string[] Stop() { on = false; th.Join(500); lock (rows) return rows.ToArray(); }
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
function Faded($t, $c) { return (LogCount "fading title=$t covered=$c`$") }
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
Pf fence create --title D0 --rect '190,280,280,200' | Out-Null
Pf item move (Join-Path $PfDesk $names[0]) --to D0 | Out-Null
Pf fence roll D0 | Out-Null; Start-Sleep -Milliseconds 500
Start-Sleep -Seconds 1; Park
Log "work bottom $WorkB, column B has $n fences; A: $(Tops $a); B: $(Tops $b); C: $(Tops $c)"
Shot 'stacks'

# Opened by the chevron: how tall, what is on top under the title row, and the saved height.
function OpenByClick($what, $col, [int]$want, [switch]$Covers) {
  $top = $col[0]; $second = $col[1]
  $before = (Win $second).T
  $f0 = Faded $second 'true'; $b0 = Faded $second 'false'
  Chevron $top
  $r = Win $top
  $over = [SeTop]::TitleAt($r.L + 150, [math]::Min($r.B - 4, $r.T + $TitleH + 30))
  Shot "$what"
  Check "$what`: opens to $want px" ([math]::Abs((WinH $top) - $want) -le 2) "window $(WinH $top) px of 300 (title $TitleH); $second top $before -> $((Win $second).T); on top just under its title row: '$over'"
  Check "$what`: nothing covers the opened fence" ($over -eq $top) "on top just under its title row: '$over'"
  Check "$what`: the saved height stays 300 px" ((SavedH $top) -eq 300) "saved rect h $(SavedH $top)"
  $faded = (Faded $second 'true') - $f0
  Check "$what`: $second $(if ($Covers) { 'under it fades out' } else { 'is not covered and stays' })" ($faded -eq [int][bool]$Covers) "faded out $faded times"
  Chevron $top
  Check "$what`: rolled up again, still 300 px saved" ((WinH $top) -le $TitleH + 2 -and (SavedH $top) -eq 300) "window $(WinH $top) px, saved rect h $(SavedH $top)"
  Check "$what`: $second back as before" (((Faded $second 'false') - $b0) -eq [int][bool]$Covers) "faded back $((Faded $second 'false') - $b0) times"
}

function Hover($what, $col, [switch]$Covers) {
  $top = $col[0]; $second = $col[1]
  $f0 = Faded $second 'true'; $b0 = Faded $second 'false'
  HoverTitle $top; Start-Sleep -Milliseconds 1400
  $r = Win $top
  $over = [SeTop]::TitleAt($r.L + 150, $r.T + $TitleH + 30)
  Shot "$what"
  Check "$what`: the hover peek opens in full, on top" ((WinH $top) -ge 298 -and $over -eq $top) "window $(WinH $top) px; on top under its title row: '$over'"
  $faded = (Faded $second 'true') - $f0
  if ($Covers) { Check "$what`: $second under the peek fades out" ($faded -eq 1) "faded out $faded times" }
  Park; Start-Sleep -Milliseconds 1200
  if ($Covers) { Check "$what`: $second back once the peek rolled up" (((Faded $second 'false') - $b0) -eq 1) "faded back $((Faded $second 'false') - $b0) times" }
}

foreach ($push in 'true', 'false') {
  Pf settings set rollUp.pushNeighbors $push | Out-Null; Start-Sleep -Milliseconds 800
  Case "push $push"
  OpenByClick "A push=$push click" $a 300 -Covers:($push -eq 'false')
  # The push on stops the top of a full column where the fences below reach the taskbar.
  if ($push -eq 'true') {
    Chevron $b[0]; $h = WinH $b[0]; Shot "B push=true click"
    Check 'B push=true click: opens as far as the fences below can slide, saved 300 px' ($h -gt $TitleH + 80 -and $h -le 300 -and (SavedH $b[0]) -eq 300) "window $h px, saved rect h $(SavedH $b[0])"
    Chevron $b[0]
  } else {
    OpenByClick "B push=$push click" $b 300 -Covers
  }
  Hover "B push=$push hover" $b -Covers:($push -eq 'false')
  # C: with the push off it stops 8 px short of C1, only for this opening.
  $cWant = if ($push -eq 'true') { 300 } else { 240 - 8 - 40 }
  OpenByClick "C push=$push click" $c $cWant
  Hover "C push=$push hover" $c
}

Case 'push off: from a hover peek onto a fence it covers partly'
# A0's peek covers A1 to A4 and the left half of D0. The pointer goes down inside A0, then out
# onto D0's right half and rests there: D0 opens only once A0's peek has rolled up.
$f0 = Faded 'D0' 'true'
HoverTitle A0; Start-Sleep -Milliseconds 1400
Check 'the peek covers D0 partly: D0 fades out' ((WinH A0) -ge 298 -and ((Faded 'D0' 'true') - $f0) -eq 1) "A0 $(WinH A0) px, D0 faded out $((Faded 'D0' 'true') - $f0) times"
[SeTop]::Track((Hw A0), (Hw D0))
Glide 142 58 300 300 8 25; Start-Sleep -Milliseconds 200
Glide 300 300 420 298 6 25; [G]::MoveTo(421, 298); Start-Sleep -Milliseconds 2500
$rows = @([SeTop]::Stop() | ForEach-Object { $s = $_ -split ' '; [pscustomobject]@{ ms = [int]$s[0]; a = [int]$s[1]; d = [int]$s[2] } })
$aShut = @($rows | Where-Object { $_.a -le $TitleH + 2 } | Select-Object -First 1)
$dOpen = @($rows | Where-Object { $_.d -gt $TitleH + 4 } | Select-Object -First 1)
$line = ($rows | ForEach-Object { "+$($_.ms):$($_.a)/$($_.d)" }) -join ' '
Shot 'D-after-peek'
Check 'D0 opens only after A0 rolled up, and it does open' ($aShut.Count -eq 1 -and $dOpen.Count -eq 1 -and $dOpen[0].ms -ge $aShut[0].ms -and (WinH D0) -gt 100) "heights A0/D0: $line"
Park; Start-Sleep -Milliseconds 1500

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
