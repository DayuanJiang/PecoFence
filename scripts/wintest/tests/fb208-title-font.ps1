# Feedback #208: two options Stardock Fences has.
#   N  显示标题栏 不显示 (fence set <fence> titleOnHover never): the title row stays folded away at
#      rest and under the pointer (with 鼠标悬停时显示 it comes back under the pointer); its place
#      is still the caption (move cursor, a drag moves the fence, a right-click opens the fence
#      menu), the roll-up chevron takes no click; rolled up and in a Peek the row shows.
#   F  字体 (settings set font "<family>"): titles, tabs and item names take the family at once,
#      Malgun Gothic and Microsoft YaHei; a family that is not installed and 默认 (null) leave
#      the fences exactly as they were.
#   P  the settings page (a debug build: run.sh --bin target/wintest-bin-debug --var Part=P):
#      字体 lists 默认 and the installed families and posts the choice; 显示标题栏 offers 不显示.
# Parts: run.sh --var Part=N ... (default: N and F). Runs on a 1600x900 box at 100 %.
if (-not $Part) { $Part = 'NF' }
trap { PfTrap $_; continue }

Add-Type -ReferencedAssemblies System.Drawing @'
using System; using System.Drawing; using System.Drawing.Imaging; using System.Runtime.InteropServices;
public static class Tf {
  [StructLayout(LayoutKind.Sequential)] public struct CI { public int cbSize, flags; public IntPtr hCursor; public int x, y; }
  [DllImport("user32.dll")] static extern bool GetCursorInfo(ref CI ci);
  [DllImport("user32.dll")] static extern IntPtr LoadCursorW(IntPtr inst, IntPtr name);
  public static string Cursor() {
    var ci = new CI(); ci.cbSize = Marshal.SizeOf(typeof(CI)); GetCursorInfo(ref ci);
    int[] ids = { 32512, 32646, 32644, 32645, 32642, 32643, 32649, 32513 };
    string[] names = { "arrow", "sizeall", "sizewe", "sizens", "sizenwse", "sizenesw", "hand", "ibeam" };
    for (int i = 0; i < ids.Length; i++) if (LoadCursorW(IntPtr.Zero, (IntPtr)ids[i]) == ci.hCursor) return names[i];
    return "other";
  }
  static int[] Pixels(Bitmap b) {
    var d = b.LockBits(new Rectangle(0, 0, b.Width, b.Height), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
    var a = new int[b.Width * b.Height]; Marshal.Copy(d.Scan0, a, 0, a.Length); b.UnlockBits(d); return a;
  }
  // How many pixels of two grabs of one region differ by more than 24 (R + G + B).
  public static int Diff(Bitmap a, Bitmap b) {
    var pa = Pixels(a); var pb = Pixels(b); int n = 0;
    for (int i = 0; i < pa.Length && i < pb.Length; i++) {
      int x = pa[i], y = pb[i];
      int d = Math.Abs(((x >> 16) & 255) - ((y >> 16) & 255)) + Math.Abs(((x >> 8) & 255) - ((y >> 8) & 255)) + Math.Abs((x & 255) - (y & 255));
      if (d > 24) n++;
    }
    return n;
  }
}
'@

function TfClean() {
  Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue | Where-Object { $_.Name -like 'tf *' } | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
}
function TfFence($title, $rect, $names) {
  foreach ($n in $names) { Set-Content -LiteralPath (Join-Path $PfDesk "$n.txt") "fb208 $n" -Encoding UTF8 }
  Start-Sleep -Seconds 2
  Pf fence create --title $title --rect $rect | Out-Null
  Pf item move @($names | ForEach-Object { Join-Path $PfDesk "$_.txt" }) --to $title | Out-Null
  Pf fence set $title sort name | Out-Null
  Start-Sleep -Seconds 2
}
function FR($t) { return (FenceGet $t).windowRect }
function Strip($wr) { return [Px]::Grab($wr.x, $wr.y, $wr.w, 36) }

Case 'setup'
CloseWindows
TfClean
GreyWallpaper
if ($Part -match 'P' -and $PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
PfStart -Lang 'zh-CN'
Pf settings set featureTips false | Out-Null
Start-Sleep -Seconds 3
ClearFences | Out-Null

if ($Part -match 'N') {
  Case 'N 不显示'
  TfFence 'TfN' '300,160,420,260' @('tf 1 报告', 'tf 2 预算表', 'tf 3 会议纪要')
  $wr = FR 'TfN'
  $h = [G]::FindWindowW('PecoFence.Fence', 'TfN')
  $in = @([int]($wr.x + 210), [int]($wr.y + 36 + 8 + 96 + 50))
  $tp = @([int]($wr.x + 150), [int]($wr.y + 16))
  $cp = @([int]($wr.x + $wr.w - 20), [int]($wr.y + 18))
  # 鼠标悬停时显示 first: the row is folded at rest and comes under the pointer.
  Pf fence set TfN titleOnHover hover | Out-Null; Start-Sleep -Milliseconds 1200
  ParkMouse 1300 700; Start-Sleep -Milliseconds 900
  $restH = Strip $wr; Crop 'N-0-hover-rest' ($wr.x - 16) ($wr.y - 16) ($wr.w + 32) ($wr.h + 32) | Out-Null
  [G]::MoveTo($in[0], $in[1]); Start-Sleep -Milliseconds 900
  $hovH = Strip $wr; Crop 'N-1-hover-pointer' ($wr.x - 16) ($wr.y - 16) ($wr.w + 32) ($wr.h + 32) | Out-Null
  $dH = [Tf]::Diff($restH, $hovH)
  Check 'N 鼠标悬停时显示: the title row comes under the pointer, as before' ($dH -gt 300) "$dH of $($wr.w * 36) px of the title row differ"
  ParkMouse 1300 700
  $r = Pf fence set TfN titleOnHover never
  Start-Sleep -Milliseconds 1200
  $got = (FenceGet 'TfN').titleOnHover
  Check 'N fence set titleOnHover never is taken and reported by fence get' ($got -eq 'never') "titleOnHover=$got; cli: $("$r".Trim() -replace '\s+', ' ')"
  Start-Sleep -Milliseconds 600
  $restN = Strip $wr; Crop 'N-2-never-rest' ($wr.x - 16) ($wr.y - 16) ($wr.w + 32) ($wr.h + 32) | Out-Null
  $dRest = [Tf]::Diff($restH, $restN)
  Check 'N 不显示 at rest looks like the folded row of 鼠标悬停时显示' ($dRest -eq 0) "$dRest px of the title row differ"
  [G]::MoveTo($in[0], $in[1]); Start-Sleep -Milliseconds 900
  $hovN = Strip $wr; Crop 'N-3-never-pointer' ($wr.x - 16) ($wr.y - 16) ($wr.w + 32) ($wr.h + 32) | Out-Null
  $dN = [Tf]::Diff($restN, $hovN)
  Check 'N 不显示: the pointer in the fence does not bring the title row' ($dN -eq 0) "$dN px of the title row differ (鼠标悬停时显示: $dH)"
  # The row's place: caption, move cursor.
  [G]::MoveTo($tp[0] - 6, $tp[1]); Start-Sleep -Milliseconds 200; [G]::MoveTo($tp[0], $tp[1]); Start-Sleep -Milliseconds 800
  $cur = [Tf]::Cursor(); $ht = NcHit $h $tp[0] $tp[1]
  $onRow = Strip $wr
  $dRow = [Tf]::Diff($restN, $onRow)
  Check 'N the hidden title row stays hidden under the pointer, is the caption (hit 2) and shows the move cursor' ($dRow -eq 0 -and $ht -eq 2 -and $cur -eq 'sizeall') "$dRow px differ, hit=$ht, cursor=$cur"
  # The chevron is not drawn and takes no click.
  $hc = NcHit $h $cp[0] $cp[1]
  [G]::Click($cp[0], $cp[1]); Start-Sleep -Milliseconds 1200
  $rolled = (FenceGet 'TfN').rolledUp
  Check 'N the roll-up chevron is not drawn and takes no click' ($hc -eq 2 -and -not $rolled) "hit at its spot=$hc, rolled after a click=$rolled"
  if ($rolled) { Pf fence unroll TfN | Out-Null; Start-Sleep -Milliseconds 800 }
  # A right-click there opens the fence menu.
  ParkMouse 1300 700; Start-Sleep -Milliseconds 300
  [G]::RClick($tp[0], $tp[1])
  $m = MenuWin 4
  $items = [Menu]::Items()
  Crop 'N-4-menu' ($wr.x - 16) ($wr.y - 16) ($wr.w + 300) ($wr.h + 260) | Out-Null
  CloseMenus; Start-Sleep -Milliseconds 500
  Check 'N a right-click on the hidden title row opens the fence menu' ($m -ne [IntPtr]::Zero -and $items) "menu=$m items=$items"
  # A drag by the hidden row moves the fence.
  DragTo $tp[0] $tp[1] ($tp[0] + 120) ($tp[1] + 70)
  $w2 = FR 'TfN'
  Check 'N dragging the hidden title row moves the fence' ([math]::Abs($w2.x - $wr.x - 120) -le 40 -and [math]::Abs($w2.y - $wr.y - 70) -le 40 -and $w2.w -eq $wr.w) "from $($wr.x),$($wr.y) to $($w2.x),$($w2.y)"
  Pf fence move TfN --rect '300,160,420,260' | Out-Null; Start-Sleep -Milliseconds 1200
  # Rolled up the row shows, so the fence can be found and opened.
  ParkMouse 1300 700; Start-Sleep -Milliseconds 600
  Pf fence roll TfN | Out-Null; Start-Sleep -Milliseconds 1500
  $stripRolled = Strip $wr; $stripRolled.Save("$PfShots\N-5-rolled.png")
  $dRoll = [Tf]::Diff($restN, $stripRolled)
  Pf fence unroll TfN | Out-Null; Start-Sleep -Milliseconds 1500
  Check 'N rolled up, the fence shows its title row' ($dRoll -gt 2000) "$dRoll of $($wr.w * 36) px differ from the hidden row"
  # In a Peek the pointer brings the row, as on a hover-only row.
  ParkMouse 1300 700
  Pf peek start | Out-Null
  for ($i = 0; $i -lt 25 -and @([W]::Tops('PecoFence.PeekDimmer')).Count -eq 0; $i++) { Start-Sleep -Milliseconds 200 }
  Start-Sleep -Milliseconds 1200
  $pk0 = Strip $wr
  [G]::MoveTo($in[0] + 6, $in[1]); Start-Sleep -Milliseconds 100; [G]::MoveTo($in[0], $in[1]); Start-Sleep -Milliseconds 1000
  $pk1 = Strip $wr; Crop 'N-6-peek-pointer' ($wr.x - 16) ($wr.y - 16) ($wr.w + 32) ($wr.h + 32) | Out-Null
  $peekOn = @([W]::Tops('PecoFence.PeekDimmer')).Count -gt 0
  $dPk = [Tf]::Diff($pk0, $pk1)
  Pf peek end | Out-Null; Start-Sleep -Milliseconds 1500
  Check 'N in a Peek the pointer over the fence brings its title row' ($peekOn -and $dPk -gt 300) "Peek on=$peekOn; $dPk px of the title row differ"
  ParkMouse 1300 700; Start-Sleep -Milliseconds 800
  $after = Strip $wr
  Check 'N after the Peek the row is hidden again' ([Tf]::Diff($restN, $after) -eq 0) "$([Tf]::Diff($restN, $after)) px differ"
  # Back to 跟随常规设置: the fence follows the general switch (off: the row shows at rest).
  Pf fence set TfN titleOnHover default | Out-Null; Start-Sleep -Milliseconds 1200
  $back = (FenceGet 'TfN').titleOnHover
  $restD = Strip $wr
  Check 'N titleOnHover default ends 不显示: the row shows again' ($back -eq 'default' -and [Tf]::Diff($restN, $restD) -gt 2000) "titleOnHover=$back, $([Tf]::Diff($restN, $restD)) px differ"
  $bad = Pf fence set TfN titleOnHover sometimes
  Check 'N a wrong value names the four allowed ones' ("$bad" -match 'never' -and "$bad" -match 'always') ("$bad".Trim() -replace '\s+', ' ')
  ClearFences | Out-Null
  TfClean
}

if ($Part -match 'F') {
  Case 'F 字体'
  TfFence 'TfF 报告 보고서' '300,160,520,300' @('tf Report 2026', 'tf 预算表 Budget', 'tf 보고서 회의록')
  Pf fence set 'TfF 报告 보고서' titleOnHover always | Out-Null
  Pf settings set font null | Out-Null
  Start-Sleep -Milliseconds 1500
  $wr = FR 'TfF 报告 보고서'
  ParkMouse 1300 700; Start-Sleep -Milliseconds 900
  $g = @(($wr.x - 8), ($wr.y - 8), ($wr.w + 16), ($wr.h + 16))
  $base = Crop 'F-0-default' $g[0] $g[1] $g[2] $g[3]
  $looks = @{}
  foreach ($face in @('Malgun Gothic', 'Microsoft YaHei', 'Georgia')) {
    $since = (Get-Date).ToUniversalTime()
    Pf settings set font $face | Out-Null
    $line = PfWaitLog "fonts changed label=Some\(`"$([regex]::Escape($face))`"\) title=Some\(`"$([regex]::Escape($face))`"\)" 10
    Start-Sleep -Milliseconds 1200
    $name = 'F-1-' + ($face -replace ' ', '-')
    $shot = Crop $name $g[0] $g[1] $g[2] $g[3]
    $d = [Tf]::Diff($base, $shot)
    $looks[$face] = $shot
    $now = (PfJson settings get).font
    Check "F 字体 ${face}: settings get reports it, the log says titles and names took it, and the fence looks different" ($now -eq $face -and $line -and $d -gt 300) "font=$now; log: $line; $d px differ from 默认"
  }
  # Titles change too: the title row of the YaHei fence is not the Malgun one.
  $dTitle = [Tf]::Diff($looks['Malgun Gothic'].Clone((New-Object Drawing.Rectangle 0, 0, $g[2], 44), $base.PixelFormat), $looks['Georgia'].Clone((New-Object Drawing.Rectangle 0, 0, $g[2], 44), $base.PixelFormat))
  Check 'F the title row follows the font (Malgun Gothic and Georgia differ in the title row)' ($dTitle -gt 50) "$dTitle px of the title row differ"
  # A family that is not installed: the built-in fonts stay, the log says why.
  Pf settings set font 'PecoFence No Such Font' | Out-Null
  $warn = PfWaitLog 'the chosen font is not installed; the built-in fonts stay' 10
  Start-Sleep -Milliseconds 1500
  $none = Crop 'F-2-not-installed' $g[0] $g[1] $g[2] $g[3]
  $dNone = [Tf]::Diff($base, $none)
  Check 'F a font that is not installed leaves the fence as with 默认' ($warn -and $dNone -eq 0) "log: $warn; $dNone px differ from 默认"
  # 默认 again: exactly as at the start.
  Pf settings set font null | Out-Null; Start-Sleep -Milliseconds 1800
  $again = Crop 'F-3-default-again' $g[0] $g[1] $g[2] $g[3]
  $dAgain = [Tf]::Diff($base, $again)
  $fontNow = (PfJson settings get).font
  Check 'F 默认 (null) brings the built-in fonts back exactly' ($null -eq $fontNow -and $dAgain -eq 0) "font=$fontNow; $dAgain px differ from the start"
  Pf fence set 'TfF 报告 보고서' titleOnHover default | Out-Null
  ClearFences | Out-Null
  TfClean
}

if ($Part -match 'P') {
  Case 'P settings page'
  TfFence 'TfP' '300,160,420,260' @('tf 1 报告', 'tf 2 预算表')
  Check 'P a debug build' $PfDebugBuild
  Check 'P the settings page is reached' (PfSettings)
  $set = "(id, v) => { const e = document.getElementById(id); e.value = v; e.dispatchEvent(new Event('change', { bubbles: true })); return e.value; }"
  $opts = Js "[...document.getElementById('fontFamily').options].map(o => o.value)"
  $has = @($opts) -contains 'Arial' -and @($opts) -contains 'Malgun Gothic' -and @($opts)[0] -eq ''
  Check 'P 字体 lists 默认 first, then the installed families (Arial, Malgun Gothic)' ($has -and @($opts).Count -gt 10) "$(@($opts).Count) options, first '$(@($opts)[0])'"
  Js "document.getElementById('fontFamily').scrollIntoView({ block: 'center' })" | Out-Null
  Start-Sleep -Milliseconds 600
  $r = PageRect '#fontFamily'
  if ($r) { Crop 'P-1-font-card' ($r.x - 560) ($r.y - 30) ($r.w + 590) ($r.h + 60) | Out-Null }
  Js "($set)('fontFamily', 'Malgun Gothic')" | Out-Null
  Start-Sleep -Milliseconds 1500
  $f1 = (PfJson settings get).font
  if ($r) { Crop 'P-2-font-chosen' ($r.x - 560) ($r.y - 30) ($r.w + 590) ($r.h + 60) | Out-Null }
  Js "($set)('fontFamily', '')" | Out-Null
  Start-Sleep -Milliseconds 1500
  $f2 = (PfJson settings get).font
  Check 'P picking a family sets font, 默认 sets it back to null' ($f1 -eq 'Malgun Gothic' -and $null -eq $f2) "after the pick: $f1, after 默认: $f2"
  # The open list shows each family in its own face.
  if ($r) {
    PageClick '#fontFamily'; Start-Sleep -Milliseconds 900
    [G]::Shot("$PfShots\P-3-font-list.png")
    [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  }
  Js "document.querySelector('a[data-page=fences]').click()" | Out-Null
  Start-Sleep -Milliseconds 800
  $id = (FenceGet 'TfP').id | ConvertTo-Json
  Js "($set)('fenceSel', $id)" | Out-Null
  Start-Sleep -Milliseconds 800
  $tOpts = Js "[...document.getElementById('fenceTitleOnHover').options].map(o => o.value).join()"
  Js "($set)('fenceTitleOnHover', 'never')" | Out-Null
  Start-Sleep -Milliseconds 1500
  $t = (FenceGet 'TfP').titleOnHover
  Js "document.getElementById('fenceTitleOnHover').scrollIntoView({ block: 'center' })" | Out-Null
  Start-Sleep -Milliseconds 500
  $r2 = PageRect '#fenceTitleOnHover'
  if ($r2) { Crop 'P-4-title-never' ($r2.x - 560) ($r2.y - 30) ($r2.w + 590) ($r2.h + 60) | Out-Null }
  Check 'P 显示标题栏 offers 不显示 and it reaches the fence' ($tOpts -eq 'default,always,hover,never' -and $t -eq 'never') "options $tOpts, fence: $t"
  ClearFences | Out-Null
  TfClean
}
