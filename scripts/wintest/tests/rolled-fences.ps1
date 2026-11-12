# Rolled fences: where they land, what covers them, and clicks during a roll (fence review
# 2026-11-07, S3 S4 S5 U11 U19 U21 C13b C3). Parts (run.sh --var Part=...): A drops near the
# bottom (both styles) and restarts, B push and hover peek, C clicks (both styles), D clicks and
# saves during a roll. Default: all parts.
if (-not $Part) { $Part = 'ABCD' }
# An error inside a part's block becomes a failed check and the next line runs.
trap { PfTrap $_; continue }
Case 'setup'
CloseWindows
foreach ($i in 1..6) { Set-Content "$PfDesk\rg-$i.txt" "rg$i" }
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
PfStart -Script @('sleep 3600000')
$WorkB = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea.Bottom
function Park() { ParkMouse 1450 300; Start-Sleep -Milliseconds 400 }
function Win($t) { return (Rect ([G]::FindWindowW('PecoFence.Fence', $t))) }
function Saved($t) { $f = FenceGet $t; return "rect=$($f.rect.x),$($f.rect.y),$($f.rect.w),$($f.rect.h) window=$($f.windowRect.x),$($f.windowRect.y),$($f.windowRect.w),$($f.windowRect.h) rolled=$($f.rolledUp)" }
function Overlap($p, $q) { return ($p.x -lt $q.x + $q.w -and $q.x -lt $p.x + $p.w -and $p.y -lt $q.y + $q.h -and $q.y -lt $p.y + $p.h) }

if ($Part -match 'A') {
  # ---- S3: a rolled fence let go low lands where it opens above the taskbar, and its saved rect
  # is where its window is (a restart leaves it there).
  Pf fence create --title Low --rect '100,100,300,250' | Out-Null
  Pf fence create --title Big --rect '588,300,300,338' | Out-Null
  Pf item move "$PfDesk\rg-1.txt" "$PfDesk\rg-2.txt" --to Low | Out-Null
  Pf item move "$PfDesk\rg-3.txt" --to Big | Out-Null
  Start-Sleep -Seconds 1
  $drops = @(@(600, 780, 'dropped low'), @(1400, 753, 'dropped low past the right edge'), @(300, ($WorkB - 96), 'title 60 px above the taskbar'), @(588, 534, 'dropped onto a fence near the bottom'))
  foreach ($style in 'fluent', 'liquidGlass') {
    Pf settings set themeStyle $style | Out-Null; Start-Sleep -Milliseconds 800
    foreach ($d in $drops) {
      Case "S3 $style, $($d[2])"
      Pf fence unroll Low | Out-Null; Pf fence move Low --rect '100,100,300,250' | Out-Null; Start-Sleep -Milliseconds 700
      Pf fence roll Low | Out-Null; Park
      $w = (FenceGet Low).windowRect
      DragTo ($w.x + 100) ($w.y + 18) ($d[0] + 100) ($d[1] + 18) 16
      Park
      $f = FenceGet Low; $t = $f.windowRect
      Check "S3 $style, $($d[2]): saved rect and window agree" ([math]::Abs($f.rect.y - $t.y) -le 2 -and [math]::Abs($f.rect.x - $t.x) -le 2) (Saved Low)
      Check "S3 $style, $($d[2]): clear of Big" (-not (Overlap $t (FenceGet Big).windowRect)) (Saved Low)
      [G]::MoveTo($t.x + 100, $t.y + 18); Start-Sleep -Milliseconds 1300
      $e = Win Low
      Shot ("S3-$style-" + ($d[2] -replace ' ', '-'))
      Check "S3 $style, $($d[2]): hover-expanded inside the work area" ($e.B -le $WorkB -and $e.B - $e.T -gt 200) "expanded window $($e.L),$($e.T),$($e.R),$($e.B), work area bottom $WorkB"
      Park; Start-Sleep -Milliseconds 600
      $before = Saved Low
      PfQuit; PfStart -Script @('sleep 3600000'); Start-Sleep -Seconds 1
      $after = Saved Low
      Check "S3 $style, $($d[2]): same place after a restart" ($before -eq $after) "before: $before; after: $after"
    }
  }
  # The bottom edge dragged under the taskbar stops at the work area: open, and while a rolled
  # fence is hover-expanded (the monkey's seed 11 left one 48 px under it, moved by the restart).
  foreach ($style in 'fluent', 'liquidGlass') {
    Pf settings set themeStyle $style | Out-Null; Start-Sleep -Milliseconds 800
    foreach ($how in 'open', 'hover-expanded') {
      Case "S3 $style, bottom edge dragged under the taskbar ($how)"
      Pf fence unroll Low | Out-Null; Pf fence move Low --rect "900,$($WorkB - 300),300,250" | Out-Null; Start-Sleep -Milliseconds 700
      if ($how -eq 'hover-expanded') { Pf fence roll Low | Out-Null; Park; $t = (FenceGet Low).windowRect; [G]::MoveTo($t.x + 120, $t.y + 18); Start-Sleep -Milliseconds 1300 }
      $hw = [G]::FindWindowW('PecoFence.Fence', 'Low'); $e = Rect $hw
      [G]::MoveTo($e.L + 150, $e.B - 2); Start-Sleep -Milliseconds 300
      MDown; Start-Sleep -Milliseconds 150; Glide ($e.L + 150) ($e.B - 2) ($e.L + 150) ($e.B + 90) 10 25; Start-Sleep -Milliseconds 300; MUp
      Start-Sleep -Milliseconds 600; Park; Start-Sleep -Milliseconds 900
      $f = FenceGet Low
      Check "S3 $style ($how): the edge stopped at the work area" ($f.rect.y + $f.rect.h -le $WorkB -and [math]::Abs($f.rect.y - $f.windowRect.y) -le 2) "before $($e.L),$($e.T),$($e.R),$($e.B); now $(Saved Low)"
      $before = Saved Low
      PfQuit; PfStart -Script @('sleep 3600000'); Start-Sleep -Seconds 1
      Check "S3 $style ($how): same place after a restart" ($before -eq (Saved Low)) "before: $before; after: $(Saved Low)"
    }
  }
  # While a rolled fence is dragged low, the white outline already shows where it will land.
  foreach ($style in 'fluent', 'liquidGlass') {
    Case "S3 $style, landing outline during a low drag"
    Pf settings set themeStyle $style | Out-Null; Start-Sleep -Milliseconds 800
    Pf fence unroll Low | Out-Null; Pf fence move Low --rect '100,100,300,250' | Out-Null; Start-Sleep -Milliseconds 600
    Pf fence roll Low | Out-Null; Park
    [G]::MoveTo(200, 118); Start-Sleep -Milliseconds 150; MDown; Start-Sleep -Milliseconds 200
    Glide 200 118 1200 798 16 40
    Start-Sleep -Milliseconds 400
    $pv = @([W]::Tops('PecoFence.DropPreview'))
    $r = if ($pv.Count) { Rect $pv[0] } else { $null }
    Shot "S3-outline-$style"
    MUp; Start-Sleep -Milliseconds 900; Park
    $t = (FenceGet Low).windowRect
    $txt = if ($r) { "$($r.L),$($r.T),$($r.R),$($r.B)" } else { 'none' }
    Check "S3 $style the outline shows where the low drop lands" ($r -and [math]::Abs($r.L - $t.x) -le 2 -and [math]::Abs($r.T - $t.y) -le 2) "outline $txt; landed $($t.x),$($t.y),$($t.w),$($t.h)"
  }
  # A column of rolled fences stays a column: a title row let go between two others keeps its spot.
  Case 'S3 column of rolled fences'
  Pf settings set themeStyle fluent | Out-Null
  Pf fence create --title ColA --rect '1000,100,300,250' | Out-Null
  Pf fence create --title ColC --rect '1000,188,300,250' | Out-Null
  Pf fence roll ColA | Out-Null; Pf fence roll ColC | Out-Null
  Pf fence unroll Low | Out-Null; Pf fence move Low --rect '100,100,300,250' | Out-Null; Start-Sleep -Milliseconds 600
  Pf fence roll Low | Out-Null; Park
  $w = (FenceGet Low).windowRect
  DragTo ($w.x + 100) ($w.y + 18) 1100 162 16
  Park
  $t = (FenceGet Low).windowRect
  Check 'S3 a rolled title between two rolled titles stays there' ($t.x -eq 1000 -and $t.y -eq 144) (Saved Low)
  Pf fence delete ColA | Out-Null; Pf fence delete ColC | Out-Null
  Pf fence delete Low | Out-Null; Pf fence delete Big | Out-Null
}

if ($Part -match 'B') {
  # ---- S4 (#197): a hover-expanded fence stays above a neighbour the push cannot clear, and goes
  # back under it after it rolls up.
  Add-Type @"
using System; using System.Text; using System.Runtime.InteropServices;
public static class RgTop {
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
  [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(POINT p);
  [DllImport("user32.dll")] static extern IntPtr GetAncestor(IntPtr h, uint f);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowTextW(IntPtr h, StringBuilder s, int n);
  public static string TitleAt(int x, int y) { var p = new POINT(); p.X = x; p.Y = y; var h = GetAncestor(WindowFromPoint(p), 2); var s = new StringBuilder(256); GetWindowTextW(h, s, 256); return s.ToString(); }
}
"@
  Pf fence create --title Roller --rect "459,$($WorkB - 250),300,250" | Out-Null
  Pf item move "$PfDesk\rg-1.txt" "$PfDesk\rg-2.txt" --to Roller | Out-Null
  Pf fence roll Roller | Out-Null; Start-Sleep -Milliseconds 800
  Pf fence create --title Small --rect "632,$($WorkB - 150),95,150" | Out-Null
  Start-Sleep -Seconds 1; Park
  # Place in the z-order (0 = topmost fence).
  function ZAt($t) { $all = @([W]::Tops('PecoFence.Fence')); for ($i = 0; $i -lt $all.Count; $i++) { if ([W]::Title($all[$i]) -eq $t) { return $i } }; return -1 }
  $smallAbove = (ZAt Small) -lt (ZAt Roller)
  Check 'S4 setup: Small starts above Roller in the z-order' $smallAbove "Small z $(ZAt Small), Roller z $(ZAt Roller)"
  foreach ($k in 1..3) {
    Case "S4 hover peek above the fence it cannot push ($k)"
    $r = (FenceGet Roller).windowRect
    [G]::MoveTo($r.x + 100, $r.y + 18); Start-Sleep -Milliseconds 1300
    $e = Win Roller; $s = (FenceGet Small).windowRect
    $px = [int]($s.x + $s.w / 2); $py = [int]($s.y + $s.h / 2)
    Shot "S4-peek-$k"
    Check "S4 the hover-expanded Roller is on top ($k)" ($e.B - $e.T -gt 200 -and [RgTop]::TitleAt($px, $py) -eq 'Roller') "Roller $($e.L),$($e.T),$($e.R),$($e.B); at Small's centre: '$([RgTop]::TitleAt($px, $py))'"
    Park; Start-Sleep -Milliseconds 900
    Check "S4 rolled up again, Roller goes back under Small ($k)" ((ZAt Small) -lt (ZAt Roller)) "Small z $(ZAt Small), Roller z $(ZAt Roller)"
  }
  Pf fence delete Roller | Out-Null; Pf fence delete Small | Out-Null

  # ---- S5: a fence let go into the spot a push only lends is moved out of the way.
  foreach ($mode in 'hover', 'click') {
    Case "S5 drop into a pushed fence's spot ($mode)"
    Pf settings set rollUp.clickToExpand $(if ($mode -eq 'click') { 'true' } else { 'false' }) | Out-Null
    Pf fence create --title Top --rect '400,60,300,250' | Out-Null
    Pf fence create --title Below --rect '550,200,300,200' | Out-Null
    Pf fence create --title Mover --rect '1000,600,160,110' | Out-Null
    Pf item move "$PfDesk\rg-3.txt" --to Top | Out-Null
    Pf fence roll Top | Out-Null; Start-Sleep -Milliseconds 800; Park
    $t = (FenceGet Top).windowRect
    if ($mode -eq 'click') { [G]::Click($t.x + 120, $t.y + 18); Start-Sleep -Milliseconds 900; Park } else { [G]::MoveTo($t.x + 120, $t.y + 18); Start-Sleep -Milliseconds 1300 }
    $m = (FenceGet Mover).windowRect
    DragTo ($m.x + 80) ($m.y + 18) 785 223 18
    Park; Start-Sleep -Milliseconds 900
    if ($mode -eq 'click') { Pf fence roll Top | Out-Null; Start-Sleep -Milliseconds 900 }
    $b = (FenceGet Below).windowRect; $mv = (FenceGet Mover).windowRect
    Shot "S5-$mode"
    Check "S5 ($mode): after the push ends Mover and Below do not overlap" (-not (Overlap $mv $b)) "Below $(Saved Below); Mover $(Saved Mover)"
    $before = (Saved Top) + ' | ' + (Saved Below) + ' | ' + (Saved Mover)
    PfQuit; PfStart -Script @('sleep 3600000'); Start-Sleep -Seconds 1
    $after = (Saved Top) + ' | ' + (Saved Below) + ' | ' + (Saved Mover)
    Check "S5 ($mode): same layout after a restart" ($before -eq $after) "before: $before; after: $after"
    foreach ($n in 'Top', 'Below', 'Mover') { Pf fence delete $n | Out-Null }
  }
  Pf settings set rollUp.clickToExpand false | Out-Null
}

if ($Part -match 'C') {
  # ---- U11 / U19: 「单击标题时展开」 in both styles: a click expands, the next click rolls it up,
  # a quick drag from the rolled title moves the fence at once, a drag of an open fence moves it.
  Pf settings set rollUp.clickToExpand true | Out-Null
  Pf fence create --title Clk --rect '200,150,300,250' | Out-Null
  Pf item move "$PfDesk\rg-4.txt" --to Clk | Out-Null
  Start-Sleep -Seconds 1
  foreach ($style in 'fluent', 'liquidGlass') {
    Pf settings set themeStyle $style | Out-Null; Start-Sleep -Milliseconds 800
    Case "U11 $style click toggles"
    Pf fence move Clk --rect '200,150,300,250' | Out-Null; Pf fence roll Clk | Out-Null; Park
    [G]::Click(300, 168); Start-Sleep -Milliseconds 900; Park
    $f = FenceGet Clk; $e = Win Clk
    Check "U11 $style a title click expands" ((-not $f.rolledUp) -and $e.B - $e.T -gt 200) (Saved Clk)
    [G]::Click(300, 168); Start-Sleep -Milliseconds 900; Park
    $f = FenceGet Clk; $e = Win Clk
    Shot "U11-$style-second-click"
    Check "U11 $style the next title click rolls it up" ($f.rolledUp -and $e.B - $e.T -lt 60) (Saved Clk)
    Pf fence unroll Clk | Out-Null; Start-Sleep -Milliseconds 900; Park
    [G]::Click(300, 168); Start-Sleep -Milliseconds 900; Park
    Check "U11 $style a title click rolls up a fence opened another way" ((FenceGet Clk).rolledUp) (Saved Clk)
    DoubleClick 300 168; Start-Sleep -Milliseconds 900; Park
    Check "U11 $style a double-click toggles once" (-not (FenceGet Clk).rolledUp) (Saved Clk)
    Case "U19 $style drags in click mode"
    foreach ($k in 1..3) {
      Pf fence unroll Clk | Out-Null; Pf fence move Clk --rect '200,150,300,250' | Out-Null; Start-Sleep -Milliseconds 600
      Pf fence roll Clk | Out-Null; Park
      [G]::MoveTo(300, 168); Start-Sleep -Milliseconds 150; MDown; Start-Sleep -Milliseconds 60
      Glide 300 168 700 468 6 15
      Start-Sleep -Milliseconds 300; MUp; Start-Sleep -Milliseconds 900; Park
      $t = (FenceGet Clk).windowRect
      Check "U19 $style a quick drag from the rolled title moves it ($k)" ([math]::Abs($t.x - 600) -le 12 -and [math]::Abs($t.y - 450) -le 12) (Saved Clk)
    }
    Pf fence unroll Clk | Out-Null; Pf fence move Clk --rect '200,150,300,250' | Out-Null; Start-Sleep -Milliseconds 700; Park
    [G]::MoveTo(300, 168); Start-Sleep -Milliseconds 150; MDown; Start-Sleep -Milliseconds 200
    Glide 300 168 700 468 16 40
    Start-Sleep -Milliseconds 300; MUp; Start-Sleep -Milliseconds 900; Park
    $f = FenceGet Clk
    Check "U19 $style an open fence dragged by the title moves and stays open" ((-not $f.rolledUp) -and [math]::Abs($f.windowRect.x - 600) -le 12) (Saved Clk)
  }
  Pf settings set themeStyle fluent | Out-Null
  Pf settings set rollUp.clickToExpand false | Out-Null
  Pf fence delete Clk | Out-Null
}

if ($Part -match 'D') {
  # ---- U21 / C13b / C3: clicks and saves while a roll is in flight.
  Pf fence create --title Anim --rect '400,200,300,250' | Out-Null
  Pf item move "$PfDesk\rg-5.txt" "$PfDesk\rg-6.txt" --to Anim | Out-Null
  Start-Sleep -Seconds 1
  $hw = [G]::FindWindowW('PecoFence.Fence', 'Anim')
  function AnimH() { $r = Rect $hw; return $r.B - $r.T }
  foreach ($k in 1..3) {
    Case "U21 chevron during the expand ($k)"
    Pf fence roll Anim | Out-Null; Park; Start-Sleep -Milliseconds 300
    [G]::Click(687, 218); Start-Sleep -Milliseconds 60
    $hAt = AnimH
    [G]::Click(673, 218); Start-Sleep -Milliseconds 200; Park; Start-Sleep -Milliseconds 900
    Check "U21 a second chevron click while expanding rolls it up again ($k)" ((FenceGet Anim).rolledUp) "height at the second click $hAt, at the end $(AnimH)"
    Case "U21 chevron during the roll-up ($k)"
    Pf fence unroll Anim | Out-Null; Park; Start-Sleep -Milliseconds 300
    # Two points apart: the same spot twice would be a double-click (one toggle by design).
    [G]::Click(687, 218); Start-Sleep -Milliseconds 40
    $hAt = AnimH
    [G]::Click(673, 218); Start-Sleep -Milliseconds 200; Park; Start-Sleep -Milliseconds 900
    Check "U21 a chevron click while rolling up expands it again ($k)" (-not (FenceGet Anim).rolledUp) "height at the second click $hAt, at the end $(AnimH)"
  }
  # The 167 ms roll-up is too short for PowerShell timing: this waits for it to start and
  # double-clicks the title at once (both presses within about 45 ms), all in compiled code.
  Add-Type @"
using System; using System.Threading; using System.Diagnostics; using System.Runtime.InteropServices;
public static class RgRoll {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll", EntryPoint = "GetWindowRect")] static extern bool WindowRectRaw(IntPtr h, out RECT r);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr GetPropW(IntPtr h, string name);
  // What the user sees of the window: during a roll the window is taller than the fence, and the app notes the part that shows in the window property PecoFence.Shown (height << 16 | width).
  static bool GetWindowRect(IntPtr h, out RECT r) { bool ok = WindowRectRaw(h, out r); long s = GetPropW(h, "PecoFence.Shown").ToInt64(); if (s != 0) { r.R = r.L + (int)(s & 0xFFFF); r.B = r.T + (int)((s >> 16) & 0xFFFF); } return ok; }
  [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] static extern void mouse_event(uint f, int x, int y, uint d, UIntPtr e);
  public static int Height(IntPtr h) { RECT r; GetWindowRect(h, out r); return r.B - r.T; }
  // Waits until the window is shorter than h0 - 3 (the roll-up started), then double-clicks
  // (x, y). Returns the heights at the first and the second press.
  public static string DblClickWhenShrinking(IntPtr h, int h0, int x, int y) {
    var sw = Stopwatch.StartNew();
    while (Height(h) >= h0 - 3 && sw.ElapsedMilliseconds < 3000) { Thread.Sleep(1); }
    int first = Height(h);
    SetCursorPos(x, y);
    mouse_event(2, 0, 0, 0, UIntPtr.Zero); Thread.Sleep(10); mouse_event(4, 0, 0, 0, UIntPtr.Zero);
    Thread.Sleep(20);
    int second = Height(h);
    mouse_event(2, 0, 0, 0, UIntPtr.Zero); Thread.Sleep(10); mouse_event(4, 0, 0, 0, UIntPtr.Zero);
    return first + " " + second + " after " + sw.ElapsedMilliseconds + " ms";
  }
}
"@
  foreach ($k in 1..4) {
    Case "C13b double-click as the hover peek rolls up ($k)"
    Pf fence roll Anim | Out-Null; Park; Start-Sleep -Milliseconds 300
    [G]::MoveTo(500, 218); Start-Sleep -Milliseconds 1100
    $hOpen = AnimH
    [G]::MoveTo(1450, 300)
    $at = [RgRoll]::DblClickWhenShrinking($hw, $hOpen, 500, 218)
    Start-Sleep -Milliseconds 300; Park; Start-Sleep -Milliseconds 1200
    $f = FenceGet Anim
    $mid = [int]($at.Split(' ')[1])
    Check "C13b the second press landed during the roll-up ($k)" ($mid -gt 36 -and $mid -lt $hOpen - 3) "heights at the presses $at"
    Check "C13b the double-click leaves it expanded ($k)" ((-not $f.rolledUp) -and (AnimH) -gt 200) "open $hOpen, heights at the presses $at, end $(AnimH), rolledUp=$($f.rolledUp)"
  }
  # C3: a cell snap or a dock to the top that lands during the roll-up keeps the expanded height.
  Pf settings set snapping.sizeToCells true | Out-Null
  Pf fence unroll Anim | Out-Null; Pf fence move Anim --rect '400,200,300,250' | Out-Null; Start-Sleep -Milliseconds 600
  Pf fence set Anim layout icons | Out-Null; Start-Sleep -Milliseconds 600
  $h0 = AnimH
  foreach ($k in 1..3) {
    Case "C3 cell snap during the roll-up ($k)"
    Pf fence unroll Anim | Out-Null; Pf fence set Anim layout icons | Out-Null; Start-Sleep -Milliseconds 700; Park
    $h0 = AnimH
    Pf fence roll Anim | Out-Null; Pf fence set Anim layout list | Out-Null
    $hAt = AnimH
    Start-Sleep -Milliseconds 800
    Pf fence unroll Anim | Out-Null; Start-Sleep -Milliseconds 900
    $f = FenceGet Anim
    Check "C3 the expanded height survives a cell snap mid roll-up ($k)" ([math]::Abs((AnimH) - $h0) -le 2 -and [math]::Abs($f.rect.h - $h0) -le 2) "height before $h0, when the snap landed $hAt, after expanding $(AnimH), saved h $($f.rect.h)"
    Case "C3 dock to the top during the roll-up ($k)"
    Pf fence set Anim layout icons | Out-Null; Pf fence move Anim --rect '400,200,300,250' | Out-Null; Start-Sleep -Milliseconds 700
    $h0 = AnimH
    Pf fence roll Anim | Out-Null; Pf fence dock-top Anim | Out-Null
    $hAt = AnimH
    Start-Sleep -Milliseconds 800
    Pf fence unroll Anim | Out-Null; Start-Sleep -Milliseconds 900
    $f = FenceGet Anim
    Check "C3 the expanded height survives a dock to the top mid roll-up ($k)" ([math]::Abs((AnimH) - $h0) -le 2 -and [math]::Abs($f.rect.h - $h0) -le 2) "height before $h0, when the dock landed $hAt, after expanding $(AnimH), saved h $($f.rect.h)"
  }
  Pf settings set snapping.sizeToCells false | Out-Null
  Pf fence delete Anim | Out-Null
}

Case 'cleanup'
[Anim]::Set($AnimWas)
PfQuit
foreach ($i in 1..6) { Remove-Item "$PfDesk\rg-$i.txt" -ErrorAction SilentlyContinue }
