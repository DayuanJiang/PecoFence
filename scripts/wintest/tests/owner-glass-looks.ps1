# Owner round 2026-11-12, package glass-looks: looks found on a PC with a GPU.
# Parts (run.sh --var Part=...):
#   R  a name box takes the colour of the text it replaces and a field that fits it: an item's
#      box and the fence's title box, both colour modes over a light and a dark wallpaper, both
#      styles, 文字颜色 白色 / 黑色. In dark mode over a light wallpaper the names are dark on a
#      light plate and the box was a dark block. The note editor is only read and logged.
# Default: all.
if (-not $Part) { $Part = 'R' }
trap { PfTrap $_; continue }

Add-Type -ReferencedAssemblies System.Drawing @'
using System; using System.Collections.Generic; using System.Drawing; using System.Drawing.Imaging; using System.Runtime.InteropServices;
public static class GLook {
  static int[] Px(Bitmap b) {
    var d = b.LockBits(new Rectangle(0, 0, b.Width, b.Height), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
    var px = new int[b.Width * b.Height]; Marshal.Copy(d.Scan0, px, 0, px.Length); b.UnlockBits(d);
    return px;
  }
  static int Lum(int c) { return (((c >> 16) & 255) * 299 + ((c >> 8) & 255) * 587 + (c & 255) * 114) / 1000; }
  static string Rgb(int c) { return ((c >> 16) & 255) + "," + ((c >> 8) & 255) + "," + (c & 255); }
  // A crop of text on a plate: the plate is the pixel of median brightness, the text the pixel
  // farthest from it. "plate rgb|text rgb|plate brightness|text brightness".
  public static string Text(Bitmap b) {
    var px = Px(b); var lum = new int[px.Length];
    for (int i = 0; i < px.Length; i++) lum[i] = Lum(px[i]);
    var sorted = (int[])lum.Clone(); Array.Sort(sorted);
    int med = sorted[sorted.Length / 2], plate = 0, ink = 0, near = 999, far = -1;
    for (int i = 0; i < px.Length; i++) {
      int diff = Math.Abs(lum[i] - med);
      if (diff < near) { near = diff; plate = i; }
      if (diff > far) { far = diff; ink = i; }
    }
    return Rgb(px[plate]) + "|" + Rgb(px[ink]) + "|" + lum[plate] + "|" + lum[ink];
  }
  // A text box: its field is the most frequent colour, its text the pixel farthest from the
  // field. The caret is the field's colour inverted and counts as neither. Same format.
  public static string Box(Bitmap b) {
    var px = Px(b); var count = new Dictionary<int, int>();
    int field = 0, most = 0;
    foreach (int p in px) { int c = p & 0xFFFFFF, n; count.TryGetValue(c, out n); count[c] = ++n; if (n > most) { most = n; field = c; } }
    int fl = Lum(field), far = -1, text = field, caret = ~field & 0xFFFFFF;
    foreach (int p in px) { int c = p & 0xFFFFFF; if (c == caret) continue; int d = Math.Abs(Lum(c) - fl); if (d > far) { far = d; text = c; } }
    return Rgb(field) + "|" + Rgb(text) + "|" + fl + "|" + Lum(text);
  }
}
'@

$GlWall = ''
function GlWallpaper($name) {
  if ($script:GlWall -eq $name) { return }
  $rgb = if ($name -eq 'light') { 236, 239, 243 } else { 18, 20, 26 }
  $bm = New-Object Drawing.Bitmap 32, 32; $gr = [Drawing.Graphics]::FromImage($bm); $gr.Clear([Drawing.Color]::FromArgb($rgb[0], $rgb[1], $rgb[2])); $gr.Dispose()
  $p = "$PfRun\wall-$name.bmp"; $bm.Save($p, [Drawing.Imaging.ImageFormat]::Bmp); $bm.Dispose()
  [G]::SystemParametersInfoW(0x14, 0, $p, 3) | Out-Null
  $script:GlWall = $name
  # PecoFence looks at the wallpaper every 5 seconds.
  Start-Sleep -Seconds 8
}
function GlRead($kind, $x, $y, $w, $h) {
  $b = [Px]::Grab([int]$x, [int]$y, [int]$w, [int]$h)
  $r = if ($kind -eq 'box') { [GLook]::Box($b) } else { [GLook]::Text($b) }
  $b.Dispose()
  $p = $r -split '\|'
  return [pscustomobject]@{ Back = $p[0]; Ink = $p[1]; BackLum = [int]$p[2]; InkLum = [int]$p[3]; DarkText = ([int]$p[3] -lt [int]$p[2]); Text = "text $($p[1]) on $($p[0])" }
}
# The largest difference between two "r,g,b" colours in one channel.
function GlApart($a, $b) {
  $x = @($a -split ',' | ForEach-Object { [int]$_ }); $y = @($b -split ',' | ForEach-Object { [int]$_ })
  return [math]::Max([math]::Max([math]::Abs($x[0] - $y[0]), [math]::Abs($x[1] - $y[1])), [math]::Abs($x[2] - $y[2]))
}
function GlPark() { ParkMouse 1540 780; Start-Sleep -Milliseconds 500 }
function GlWin($t) { return (FenceGet $t).windowRect }
$GlDocs = '季度报告 2026.docx', '笔记.txt', 'a1 新年活动安排与分工.txt', 'README.md'
$GlScroll = @(1..14 | ForEach-Object { '滚动 {0:D2} 文件.txt' -f $_ })
$GlAll = $GlDocs + $GlScroll
function GlClean() { foreach ($n in $GlAll) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue } }
# The name box that is open now, read with nothing selected in it (End).
function GlOpenBox($tag, $inset, $bottom) {
  $box = WaitWin 'PecoFence.Rename' 4
  if ($box -eq [IntPtr]::Zero) { return $null }
  Start-Sleep -Milliseconds 300
  [G]::Press(0x23); Start-Sleep -Milliseconds 400
  $r = Rect $box
  $q = GlRead 'box' ($r.L + $inset) ($r.T + $inset) ($r.R - $r.L - 2 * $inset) ($r.B - $r.T - $inset - $bottom)
  "$tag`t$($q.Text)`tbox $($r.L),$($r.T) $($r.R - $r.L)x$($r.B - $r.T)" | Add-Content "$PfShots\boxes.tsv" -Encoding UTF8
  return $q
}

Case 'setup'
CloseWindows
GlClean
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
foreach ($n in $GlAll) { Set-Content -LiteralPath (Join-Path $PfDesk $n) -Value "glass looks $n" -Encoding UTF8 }
GlWallpaper 'light'
PfStart -Script @('sleep 3600000') -Lang 'zh-CN'
Start-Sleep -Seconds 2
ClearFences | Out-Null
Pf settings set featureTips false | Out-Null
Pf settings set themeStyle fluent | Out-Null
Pf fence create --title 文档 --rect '200,100,360,240' | Out-Null
Pf item move @($GlDocs | ForEach-Object { Join-Path $PfDesk $_ }) --to 文档 | Out-Null
Pf fence set 文档 sort name | Out-Null
Pf fence create --tool note --title 便签 --text "今天要做的事`n给安娜打电话" --rect '640,100,300,170' | Out-Null
Pf fence create --title 滚动 --rect '200,420,360,236' | Out-Null
Pf item move @($GlScroll | ForEach-Object { Join-Path $PfDesk $_ }) --to 滚动 | Out-Null
Pf fence set 滚动 sort name | Out-Null
Start-Sleep -Seconds 3
GlPark
Check 'the fences are there' (@(PfJson fence list | Where-Object { $_ -and $_.id }).Count -ge 4) "fences=$(@(PfJson fence list).Count)"

if ($Part -match 'R') {
  # One look: the item's name box, then the fence's title box, each against the text it covers.
  function GlBoxes($style, $mode, $wall, $label, [switch]$Note) {
    trap { PfTrap $_; continue }
    $tag = "$style-$mode-$wall-$label"
    Case "name boxes: $style style, $mode mode, $wall wallpaper, text colour $label"
    Pf settings set themeStyle $style | Out-Null
    Pf settings set theme $mode | Out-Null
    Pf settings set labelColor $label | Out-Null
    GlWallpaper $wall
    Start-Sleep -Milliseconds 1500
    $w = GlWin '文档'; $c = CellPt '文档' 1
    # Nothing selected: a click on the fence's empty lower part.
    [G]::Click(($w.x + $w.w - 40), ($w.y + $w.h - 30)); GlPark
    $name = GlRead 'text' ($c[0] - 36) ($c[1] + 28) 72 30
    $title = GlRead 'text' ($w.x + 12) ($w.y + 8) 120 22
    "$tag/name`t$($name.Text)" | Add-Content "$PfShots\boxes.tsv" -Encoding UTF8
    "$tag/title`t$($title.Text)" | Add-Content "$PfShots\boxes.tsv" -Encoding UTF8
    # The item's box (F2 on the selected item).
    [G]::Click($c[0], $c[1]); Start-Sleep -Milliseconds 400; GlPark
    [G]::Press(0x71)
    $q = GlOpenBox "$tag/item box" 3 6
    Crop "R-$tag-item" $w.x $w.y $w.w $w.h | Out-Null
    $side = if ($name.DarkText) { 'dark' } else { 'light' }
    Check "the item's name box has $side text, like the name it covers" ($q -and $q.DarkText -eq $name.DarkText) "name: $($name.Text); box: $(if ($q) { $q.Text } else { 'did not open' })"
    # The same colour as the name: a text colour with some transparency (Fluent's dark text) is
    # blended over the box's field there and over the plate here, hence the 32 levels of room.
    Check "the item's name box has the name's own text colour" ($q -and (GlApart $q.Ink $name.Ink) -le 32) "name text $($name.Ink), box text $(if ($q) { $q.Ink } else { 'did not open' })"
    Check "the item's name box has a field that $side text reads on" ($q -and (($q.BackLum -ge 128) -eq $name.DarkText) -and [math]::Abs($q.BackLum - $q.InkLum) -ge 120) "box: $(if ($q) { "$($q.Text), brightness $($q.InkLum) on $($q.BackLum)" } else { 'did not open' })"
    [G]::Press(0x1B); Start-Sleep -Milliseconds 500
    # The title's box (the fence menu's 重命名).
    [G]::RClick([int]($w.x + $w.w / 2), ($w.y + 14)); Start-Sleep -Milliseconds 700
    MenuPick '^重命名' (MenuWin 5) | Out-Null
    $q = GlOpenBox "$tag/title box" 2 2
    Crop "R-$tag-title" $w.x $w.y $w.w $w.h | Out-Null
    $side = if ($title.DarkText) { 'dark' } else { 'light' }
    Check "the title's name box has $side text, like the title it covers" ($q -and $q.DarkText -eq $title.DarkText) "title: $($title.Text); box: $(if ($q) { $q.Text } else { 'did not open' })"
    Check "the title's name box has the title's own text colour" ($q -and (GlApart $q.Ink $title.Ink) -le 32) "title text $($title.Ink), box text $(if ($q) { $q.Ink } else { 'did not open' })"
    Check "the title's name box has a field that $side text reads on" ($q -and (($q.BackLum -ge 128) -eq $title.DarkText) -and [math]::Abs($q.BackLum - $q.InkLum) -ge 120) "box: $(if ($q) { "$($q.Text), brightness $($q.InkLum) on $($q.BackLum)" } else { 'did not open' })"
    [G]::Press(0x1B); Start-Sleep -Milliseconds 500
    if ($Note) {
      # The note editor: read only (its look belongs to another change).
      $n = GlWin '便签'
      $text = GlRead 'text' ($n.x + 14) ($n.y + 42) 220 44
      [G]::Click(($n.x + 150), ($n.y + 120)); Start-Sleep -Milliseconds 300
      $ed = WaitWin 'PecoFence.NoteEdit' 4
      Start-Sleep -Milliseconds 500
      if ($ed -ne [IntPtr]::Zero) {
        $r = Rect $ed
        $q = GlRead 'box' ($r.L + 2) ($r.T + 2) ($r.R - $r.L - 24) ($r.B - $r.T - 4)
        Crop "R-$tag-note" $n.x $n.y $n.w $n.h | Out-Null
        "$tag/note editor`t$($q.Text)`tnote text before: $($text.Text)" | Add-Content "$PfShots\boxes.tsv" -Encoding UTF8
        Log "note editor, ${tag}: $($q.Text); the note's text before: $($text.Text)"
        [G]::Press(0x1B); Start-Sleep -Milliseconds 500
      }
    }
    GlPark
  }
  GlBoxes fluent dark light auto -Note
  GlBoxes fluent light light auto
  GlBoxes fluent light light white
  GlBoxes liquidGlass dark light auto
  GlBoxes fluent light dark auto -Note
  GlBoxes fluent dark dark auto
  GlBoxes fluent dark dark black
  GlBoxes liquidGlass light dark auto
  Pf settings set labelColor auto | Out-Null
}

Case 'cleanup'
PfQuit
GlClean
GreyWallpaper
[Anim]::Set($AnimWas)
