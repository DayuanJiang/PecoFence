# Owner round 2026-11-12, fourth batch, package ux4-open: findings the third batch left open.
# Parts (run.sh --var Part=...):
#   K  the keyboard after Esc in the Peek search while a card is on screen; the focus mark of a
#      fence whose tabs stand at the left
#   M  motion (Windows' animations on, recordings): 按时间分组 with many and with few icons, the
#      scroll bar that new files bring; a look: the hover fill after a press the pointer
#      jumped to
#   R  a look (a recording): a new name confirmed by a click on the bare part of the fence
#   W  the settings window behind another window or minimised: the command line and the tray's
#      设置… bring it to the front
#   T  (a debug build) the tips of the settings page's icon buttons follow the app's colour
#      mode, whatever Windows' own mode is
#   S  a look: a first start while the disk keeps writes waiting (about a minute): how long
#      until the first-start card shows
#   A  (--extra ui/themes) the accent of a theme file reaches the bar under the shown tab, a
#      ticked check box and a link also where the text took the other colour mode's ink
# Runs on a 1600x900 box at 100 %.
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'KMW' }
trap { PfTrap $_; continue }
. C:\pf\kit\rec.ps1
. C:\pf\kit\user.ps1

Add-Type @'
using System; using System.Runtime.InteropServices;
public static class OoWin {
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  // The pointer is put on a point and the button goes down at once, with no move on the way
  // (what a touch, a pen or a remote session does).
  public static void JumpClick(int x, int y) {
    SetCursorPos(x, y); mouse_event(2, 0, 0, 0, UIntPtr.Zero); System.Threading.Thread.Sleep(40); mouse_event(4, 0, 0, 0, UIntPtr.Zero);
  }
}
'@

$OoDocs = @('季度销售报告.pdf', '合同 2026.pdf', '发票.pdf', '说明.txt', '会议记录 11-12.txt', '全家福.png', '截图 2026-11-12.png')
$OoStems = @('季度销售报告', '合同', '发票 2026', '说明', '会议记录 11-12', '全家福', '截图', '一个名字特别长的客户往来文件最终版', '预算', '清单 A', '备忘', '照片 01', '草稿', '方案 v2', '总结', '日程')
function OoClean() {
  foreach ($n in $OoDocs) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
  Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue | Where-Object { $_.Name -match '\.fq[abc]$' } | Remove-Item -Force -ErrorAction SilentlyContinue
}
# The card on screen (a card that was answered fades out and stays as a hidden window).
function OoCard() { foreach ($h in @([W]::Tops('PecoFence.Notice'))) { if ([G]::IsWindowVisible($h)) { return $h } }; return $null }
function OoCardTitle() { $n = OoCard; if ($n) { return [L]::Text($n) }; return 'none' }
# The most dark pixels in one of the screen columns $x0..$x1 between $y0 and $y1: the left edge
# of the focus mark (a dark 2 px outline in light mode).
function OoDarkCol($x0, $x1, $y0, $y1) {
  $b = [Px]::Grab($x0, $y0, $x1 - $x0 + 1, $y1 - $y0); $best = 0
  foreach ($c in 0..($x1 - $x0)) {
    $n = 0
    foreach ($r in 0..($y1 - $y0 - 1)) { $p = $b.GetPixel($c, $r); if ($p.R + $p.G + $p.B -lt 200) { $n++ } }
    if ($n -gt $best) { $best = $n }
  }
  $b.Dispose(); return $best
}
function OoSettingsWin() { return [G]::FindWindowW('PecoFence.Settings', [NullString]::Value) }
function OoCloseSettings() {
  $h = OoSettingsWin; $r = [IntPtr]::Zero
  if ($h -ne [IntPtr]::Zero) { [G]::SendMessageTimeoutW($h, 0x10, [IntPtr]0, [IntPtr]0, 2, 5000, [ref]$r) | Out-Null }
  for ($i = 0; $i -lt 50 -and (OoSettingsWin) -ne [IntPtr]::Zero; $i++) { Start-Sleep -Milliseconds 100 }
}

Case 'setup'
CloseWindows
OoClean

# ---- K: where the keyboard is
function OoKeys() {
  trap { PfTrap $_; continue }
  Case 'K setup'
  foreach ($n in $OoDocs) { DeskFile $n | Out-Null }
  PfStart -Lang zh-CN -Script (DumpScript 250 8)
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  ClearFences | Out-Null
  Pf fence create --title '文档' --rect '200,120,340,240' | Out-Null
  Pf item move --glob '*.pdf' --to '文档' | Out-Null
  Pf fence create --title '文字' --rect '600,120,340,240' | Out-Null
  Pf item move --glob '*.txt' --to '文字' | Out-Null
  Pf fence create --title '图片' --rect '1000,120,340,240' | Out-Null
  Pf item move --glob '*.png' --to '图片' | Out-Null
  Pf fence create --title '空的' --rect '1000,500,340,200' | Out-Null
  Pf fence create --title '多余的' --rect '200,500,300,200' | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse 1300 800

  # A fence deleted from its menu leaves the undo card on screen. The desktop has the keyboard,
  # a Peek opens, a letter goes into its search box, Esc: the keyboard is on the desktop again
  # (it was on the card, which showed no mark and took no key).
  Case 'K1 Esc in the Peek search while a card is on screen'
  $w = DumpOf '多余的' 4
  [G]::RClick($w.l + 150, $w.t + 14); Start-Sleep -Milliseconds 900
  $picked = MenuPick '^删除栅栏'
  Start-Sleep -Milliseconds 1500
  $card = OoCardTitle
  DeskClick 1300 700 | Out-Null; Start-Sleep -Milliseconds 600
  $fg0 = Fg
  PeekKey; Start-Sleep -Seconds 1
  [G]::Type('a'); Start-Sleep -Milliseconds 700
  $box = [M]::BoxText()
  Shot 'k1-peek-search'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 1500
  if (@([W]::Tops('PecoFence.PeekDimmer') | Where-Object { [G]::IsWindowVisible($_) }).Count -ge 1) { [G]::Press(0x1B); Start-Sleep -Milliseconds 1500 }
  $fg1 = Fg
  Shot 'k1-after-esc'
  Check 'K1 setup: the undo card is on screen and the search box took the letter' ($card -ne 'none' -and (OoCardTitle) -ne 'none' -and $box -eq 'a') "menu=[$picked] card=[$card] box=[$box]"
  Check 'K1 after Esc the keyboard is back where it was before the Peek' ($fg1 -eq $fg0 -and $fg1 -notmatch 'PecoFence.Notice') "before the Peek: $fg0; after Esc: $fg1"
  CloseNotice | Out-Null

  # Tabs at the left: the tab with nothing in it has the keyboard, so the mark is on the title
  # row, which is the column there. It goes around the shown tab's row (it went around the
  # whole column).
  Case 'K2 tabs at the left: the focus mark is around the row of the shown tab'
  Pf fence merge '图片' --into '文字' | Out-Null
  Pf fence merge '空的' --into '文字' | Out-Null
  Pf settings set tabBar left | Out-Null
  Start-Sleep -Seconds 2
  $w = DumpOf '文字' 4
  [G]::Click($w.r - 30, $w.b - 30); Start-Sleep -Milliseconds 500
  ParkMouse 1300 800
  foreach ($i in 1..3) {
    Chord 0x11 0x09; Start-Sleep -Milliseconds 1200
    $w = DumpOf '文字' 4
    if ((DumpVal $w 'items') -eq '0') { break }
  }
  $x0 = $w.l + [int]$w.ox + 1
  $col = OoDarkCol $x0 ($x0 + 11) ($w.t + 2) ($w.b - 2)
  $h = $w.b - $w.t
  Shot 'k2-rail-focus'
  Crop 'k2-rail' ($w.l - 12) ($w.t - 12) 220 ($h + 24) | Out-Null
  Check 'K2 setup: the tab with nothing in it has the keyboard' ((DumpVal $w 'items') -eq '0' -and (DumpVal $w 'focus_ring') -eq 'true') "items=$(DumpVal $w 'items') focus_ring=$(DumpVal $w 'focus_ring') tabs=$($w.tabs -join ',')"
  Check 'K2 the mark is as tall as one row of the column' ($col -ge 24 -and $col -le 44) "the mark's left edge: $col px; the column: $h px"
  Pf settings set tabBar top | Out-Null
  PfQuit
}
if ($Part -match 'K') { OoKeys }

# ---- M: motion
function OoMotion() {
  trap { PfTrap $_; continue }
  Case 'M setup'
  $animWas = [Anim]::Get(); [Anim]::Set($true)
  # Files of today, yesterday, eight days and forty days ago: four groups by time.
  $k = 0
  foreach ($s in $OoStems) { $p = DeskFile "$s.fqa"; (Get-Item -LiteralPath $p).LastWriteTime = (Get-Date).AddDays(-@(0, 1, 8, 40)[$k % 4]); $k++ }
  $k = 0
  foreach ($s in $OoStems[0..7]) { $p = DeskFile "$s.fqb"; (Get-Item -LiteralPath $p).LastWriteTime = (Get-Date).AddDays(-@(0, 1, 8, 40)[$k % 4]); $k++ }
  # Sixteen files of one day: 按时间分组 only moves them down by its one caption.
  foreach ($s in $OoStems) { DeskFile "$s.fqc" | Out-Null }
  PfStart -Lang zh-CN
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  ClearFences | Out-Null
  Pf fence create --title '多的' --rect '60,80,420,440' | Out-Null
  Pf fence create --title '少的' --rect '520,80,420,300' | Out-Null
  Pf item move --glob '*.fqa' --to '多的' | Out-Null
  Pf item move --glob '*.fqb' --to '少的' | Out-Null
  Pf fence create --title '同一天' --rect '1000,300,420,440' | Out-Null
  Pf item move --glob '*.fqc' --to '同一天' | Out-Null
  # Sorted by date: 按时间分组 keeps the order then, so its own motion is what shows (where
  # the order changes too, the list that follows cross-fades by the sort's rule).
  Pf fence set '多的' sort date | Out-Null
  Pf fence set '少的' sort date | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse 1300 800
  $n16 = @(@(PfJson item list --fence '多的') | Where-Object { $_ -and $_.name }).Count
  $n8 = @(@(PfJson item list --fence '少的') | Where-Object { $_ -and $_.name }).Count
  Check 'M setup: sixteen icons in one fence, eight in the other' ($n16 -eq 16 -and $n8 -eq 8) "$n16 and $n8"

  # 按时间分组 on and off: sixteen icons cross-fade (they all glided through each other),
  # eight glide as before.
  Case 'M3 按时间分组: many icons cross-fade, a few glide'
  RecStart 'm3-group' -X 60 -Y 80 -W 880 -H 440 -Fps 30
  Start-Sleep -Milliseconds 300
  RecMark '打开「按时间分组」'
  Pf fence set '多的' groupByDate true | Out-Null
  Pf fence set '少的' groupByDate true | Out-Null
  Start-Sleep -Milliseconds 1500
  RecMark '关闭「按时间分组」'
  Pf fence set '多的' groupByDate false | Out-Null
  Pf fence set '少的' groupByDate false | Out-Null
  Start-Sleep -Milliseconds 1500
  RecStop
  Pf fence set '同一天' groupByDate true | Out-Null
  Start-Sleep -Milliseconds 1200
  Pf fence set '同一天' groupByDate false | Out-Null
  Start-Sleep -Milliseconds 1200
  Pf fence delete '同一天' | Out-Null
  Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue | Where-Object { $_.Name -match '\.fqc$' } | Remove-Item -Force -ErrorAction SilentlyContinue
  $lines = @(PfLogAll | Where-Object { $_ -match 'group by date changed' } | ForEach-Object { $_ -replace '^.*(group by date changed)', '$1' } | Select-Object -Unique)
  $fade16 = @($lines | Where-Object { $_ -match 'cross-fades' -and $_ -match 'items=16' }).Count
  $glide8 = @($lines | Where-Object { $_ -match 'icons glide' -and $_ -match 'items=8' }).Count
  Check 'M3 sixteen icons cross-fade when 按时间分组 goes on and off' ($fade16 -ge 2) "$($lines -join ' | ')"
  Check 'M3 eight icons glide' ($glide8 -ge 2) "glide lines for eight icons: $glide8"
  $glide16 = @($lines | Where-Object { $_ -match 'icons glide' -and $_ -match 'items=16' }).Count
  Check 'M3 sixteen icons of one day, which only move down by the caption, glide' ($glide16 -ge 2) "glide lines for sixteen icons: $glide16"

  # Nine files arrive in a fence 150 px high: they need a scroll bar. It comes with the icons
  # (it stood there about three frames before them).
  Case 'M4 the scroll bar that new files bring comes with them'
  Pf fence create --title '丙' --rect '1000,80,340,150' | Out-Null
  Pf rule add --name fqc --ext fqc --to '丙' | Out-Null
  Start-Sleep -Milliseconds 800
  ParkMouse 1300 800
  RecStart 'm4-arrive' -X 1000 -Y 80 -W 340 -H 150 -Fps 30
  Start-Sleep -Milliseconds 300
  RecMark '九个文件同时到来'
  foreach ($k in 1..9) { DeskFile "来的文件$k.fqc" | Out-Null }
  Start-Sleep -Milliseconds 3000
  RecStop
  Shot 'm4-after-arrive'
  $with = @(PfLogAll | Where-Object { $_ -match 'new icons bring a scroll bar' }).Count
  Check 'M4 the scroll bar fades in with the new icons' ($with -ge 1) "log lines: $with"

  # A look (the same on the round's base): the pointer rests on the fifth icon, then jumps
  # onto the first one and presses: the fifth is plain again. The third batch saw it keep its
  # hover fill until the next key in one recording; neither way of clicking shows that.
  Case 'M7 look: the hover fill after a press the pointer jumped to'
  $p5 = CellPt '少的' 4; $p1 = CellPt '少的' 0
  ParkMouse 1300 800; Start-Sleep -Milliseconds 600
  $b0 = [Px]::Grab($p5[0] - 40, $p5[1] - 30, 80, 86)
  [G]::MoveTo($p5[0], $p5[1]); Start-Sleep -Milliseconds 800
  $b1 = [Px]::Grab($p5[0] - 40, $p5[1] - 30, 80, 86)
  [OoWin]::JumpClick($p1[0], $p1[1]); Start-Sleep -Milliseconds 1000
  $b2 = [Px]::Grab($p5[0] - 40, $p5[1] - 30, 80, 86)
  Crop 'm7-after-press' ($p1[0] - 50) ($p1[1] - 40) 520 110 | Out-Null
  $d1 = [Px]::Diff($b0, $b1, 0, 80, 0, 86); $d2 = [Px]::Diff($b0, $b2, 0, 80, 0, 86)
  $b0.Dispose(); $b1.Dispose(); $b2.Dispose()
  Check 'M7 setup: the fifth icon showed its hover fill' ($d1 -gt 0.5) "difference from the plain icon: $d1"
  Check 'M7 after the press on the first icon the fifth is plain again' ($d2 -lt $d1 * 0.15) "difference from the plain icon: $d2 (with the hover fill: $d1)"
  # The steps the finding was seen with: a click on the first icon, Ctrl on the third, Shift
  # on the fifth, then a plain click on the first.
  $p3 = CellPt '少的' 2
  ParkMouse 1300 800; Start-Sleep -Milliseconds 600
  [G]::Click($p1[0], $p1[1]); Start-Sleep -Milliseconds 500
  [G]::KeyDown(0x11); [G]::Click($p3[0], $p3[1]); [G]::KeyUp(0x11); Start-Sleep -Milliseconds 600
  [G]::KeyDown(0x10); [G]::Click($p5[0], $p5[1]); [G]::KeyUp(0x10); Start-Sleep -Milliseconds 600
  [G]::Click($p1[0], $p1[1]); Start-Sleep -Milliseconds 700
  $b3 = [Px]::Grab($p5[0] - 40, $p5[1] - 30, 80, 86)
  Crop 'm7-after-clicks' ($p1[0] - 50) ($p1[1] - 40) 520 110 | Out-Null
  # The plain fifth icon of a fence that has the keyboard: the pointer away, the first selected.
  ParkMouse 1300 800; Start-Sleep -Milliseconds 700
  $b4 = [Px]::Grab($p5[0] - 40, $p5[1] - 30, 80, 86)
  $d3 = [Px]::Diff($b4, $b3, 0, 80, 0, 86)
  $b3.Dispose(); $b4.Dispose()
  Check 'M7 after Ctrl and Shift clicks and a plain click on the first icon the fifth is plain' ($d3 -lt $d1 * 0.15) "difference from the plain icon: $d3 (with the hover fill: $d1)"

  [Anim]::Set($animWas)
  PfQuit
}
if ($Part -match 'M') { OoMotion }

# ---- R: a look at a new name confirmed by a click on the bare part of the fence
function OoRename() {
  trap { PfTrap $_; continue }
  Case 'R look: a new name confirmed by a click on the bare part of the fence'
  $animWas = [Anim]::Get(); [Anim]::Set($true)
  foreach ($s in $OoStems[0..5]) { DeskFile "$s.fqb" | Out-Null }
  PfStart -Lang zh-CN
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  ClearFences | Out-Null
  Pf fence create --title '改名' --rect '520,80,420,300' | Out-Null
  Pf item move --glob '*.fqb' --to '改名' | Out-Null
  Log ("sort: " + (Pf fence set '改名' sort name))
  Start-Sleep -Seconds 2
  ParkMouse 1300 800
  # The last icon gets a name that sorts first: it glides to the first cell.
  $p6 = CellPt '改名' 5
  [G]::Click($p6[0], $p6[1]); Start-Sleep -Milliseconds 500
  [G]::Press(0x71); Start-Sleep -Milliseconds 1000
  [G]::Type('aa'); Start-Sleep -Milliseconds 500
  $g = FenceGet '改名'
  RecStart 'r-rename' -X 520 -Y 80 -W 420 -H 300 -Fps 30
  Start-Sleep -Milliseconds 300
  RecMark '点栅栏空白处确认新名字'
  [G]::Click($g.windowRect.x + $g.windowRect.w - 30, $g.windowRect.y + $g.windowRect.h - 20)
  Start-Sleep -Milliseconds 1500
  RecStop
  $names = (@(PfJson item list --fence '改名') | Where-Object { $_ -and $_.name } | ForEach-Object { $_.fileName }) -join ' | '
  Check 'R the renamed file stands first in the fence sorted by name' ($names -like 'aa*') "$names (sort=$($g.sort))"
  [Anim]::Set($animWas)
  PfQuit
}
if ($Part -match 'R') { OoRename }

# ---- W: the settings window comes to the front
function OoNotepadFront($n) {
  # Its right part stands out beside the settings window: a click on the title bar there.
  $r = Rect $n
  [G]::Click($r.R - 220, $r.T + 12); Start-Sleep -Milliseconds 500
  return ([G]::GetForegroundWindow() -eq $n)
}
function OoFront() {
  trap { PfTrap $_; continue }
  Case 'W setup'
  PfStart -Lang zh-CN
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  Pf settings open-ui | Out-Null
  $h = WaitWin 'PecoFence.Settings' 30
  Start-Sleep -Seconds 5
  $r = Rect $h
  Start-Process notepad | Out-Null
  $n = WaitWin 'Notepad' 15
  [L]::MoveWindow($n, $r.R - 380, $r.T + 60, 760, 520, $true) | Out-Null
  Start-Sleep -Milliseconds 500
  Check 'W setup: the settings window and Notepad over it' ($h -ne [IntPtr]::Zero -and $n -ne [IntPtr]::Zero) "settings=$h notepad=$n"

  Case 'W the command line brings the settings window to the front'
  $np = OoNotepadFront $n
  Pf settings open-ui | Out-Null; Start-Sleep -Milliseconds 1200
  Shot 'w-cli-behind'
  Check 'W behind Notepad: pecofence-cli settings open-ui brings it to the front' ($np -and [G]::GetForegroundWindow() -eq $h) "Notepad was in front: $np; now $(Fg)"
  [OoWin]::ShowWindow($h, 6) | Out-Null; Start-Sleep -Milliseconds 600
  $np = OoNotepadFront $n
  Pf settings open-ui | Out-Null; Start-Sleep -Milliseconds 1200
  Check 'W minimised, Notepad in front: the command line restores it in front' ($np -and -not [OoWin]::IsIconic($h) -and [G]::GetForegroundWindow() -eq $h) "Notepad was in front: $np; iconic $([OoWin]::IsIconic($h)); now $(Fg)"

  Case 'W the tray''s 设置… brings the settings window to the front'
  $np = OoNotepadFront $n
  $t = TrayPick '^设置'
  Start-Sleep -Milliseconds 1200
  Shot 'w-tray-behind'
  Check 'W behind Notepad: 设置… of the tray brings it to the front' ($np -and [G]::GetForegroundWindow() -eq $h) "menu=[$t] Notepad was in front: $np; now $(Fg)"
  [OoWin]::ShowWindow($h, 6) | Out-Null; Start-Sleep -Milliseconds 600
  $np = OoNotepadFront $n
  $t = TrayPick '^设置'
  Start-Sleep -Milliseconds 1200
  Check 'W minimised, Notepad in front: 设置… of the tray restores it in front' ($np -and -not [OoWin]::IsIconic($h) -and [G]::GetForegroundWindow() -eq $h) "menu=[$t] iconic $([OoWin]::IsIconic($h)); now $(Fg)"

  Case 'W a new settings window asked for by the command line opens in front'
  OoCloseSettings
  Start-Sleep -Seconds 2
  $np = OoNotepadFront $n
  Pf settings open-ui | Out-Null
  $h2 = WaitWin 'PecoFence.Settings' 30
  Start-Sleep -Milliseconds 1500
  Shot 'w-cli-new'
  Check 'W Notepad in front: a new settings window opens in front of it' ($np -and $h2 -ne [IntPtr]::Zero -and [G]::GetForegroundWindow() -eq $h2) "Notepad was in front: $np; now $(Fg)"
  OoCloseSettings
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  PfQuit
}
if ($Part -match 'W') { OoFront }

# ---- T: the tips the browser draws (the settings page's icon buttons) in the app's mode
$OoThemeKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize'
function OoWinMode($light) {
  if ($null -eq $light) { Remove-ItemProperty -Path $OoThemeKey -Name AppsUseLightTheme -ErrorAction SilentlyContinue }
  else {
    # New-Item -Force on a key that is there empties it: SystemUsesLightTheme went with it, and
    # every later job on the box had PecoFence on dark (2026-11-13).
    if (-not (Test-Path -LiteralPath $OoThemeKey)) { New-Item -Path $OoThemeKey -Force -ErrorAction SilentlyContinue | Out-Null }
    Set-ItemProperty -Path $OoThemeKey -Name AppsUseLightTheme -Value $light -Type DWord
  }
}
# The tone of the tip of the first 编辑 button on 整理规则: how many of the pixels that changed
# under the pointer are light and how many are dark.
function OoTipTone($name) {
  PageClick 'nav a[data-page=rules]'
  Start-Sleep -Milliseconds 900
  $b = PageRect '.iconbtn[data-act=edit]'
  if (-not $b) { return 'no button' }
  [G]::MoveTo($b.cx - 260, $b.cy + 160); Start-Sleep -Milliseconds 700
  # Above and under the button (the button itself changes under the pointer and is left out).
  $x0 = $b.cx - 95; $y0 = $b.cy - 64
  $g0 = [Px]::Grab($x0, $y0, 190, 146)
  [G]::MoveTo($b.cx - 2, $b.cy); Start-Sleep -Milliseconds 300
  [G]::MoveTo($b.cx, $b.cy); Start-Sleep -Milliseconds 2200
  $g1 = [Px]::Grab($x0, $y0, 190, 146)
  Crop $name ($b.cx - 200) ($b.cy - 70) 400 160 | Out-Null
  $light = 0; $dark = 0; $changed = 0
  foreach ($y in @(0..45) + @(82..145)) {
    foreach ($x in 0..189) {
      $p = $g0.GetPixel($x, $y); $q = $g1.GetPixel($x, $y)
      if ([math]::Abs($p.R - $q.R) + [math]::Abs($p.G - $q.G) + [math]::Abs($p.B - $q.B) -gt 12) {
        $changed++; $s = $q.R + $q.G + $q.B
        if ($s -gt 540) { $light++ } elseif ($s -lt 300) { $dark++ }
      }
    }
  }
  $g0.Dispose(); $g1.Dispose()
  return "changed=$changed light=$light dark=$dark"
}
function OoTips() {
  trap { PfTrap $_; continue }
  $script:OoWas = (Get-ItemProperty -Path $OoThemeKey -ErrorAction SilentlyContinue).AppsUseLightTheme
  $script:OoTouched = $true
  foreach ($c in @(@('dark', 1), @('light', 0))) {
    Case "T tips of the icon buttons: PecoFence $($c[0]), Windows' apps light=$($c[1])"
    OoWinMode $c[1]
    $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
    PfStart -Lang zh-CN
    Start-Sleep -Seconds 4
    CloseNotice | Out-Null
    Pf settings set theme $c[0] | Out-Null
    $f = @(@(PfJson fence list) | Where-Object { $_ -and $_.id }) | Select-Object -First 1
    Pf rule add --name '文档' --ext pdf --to $f.id | Out-Null
    $ok = PfSettings
    Start-Sleep -Seconds 1
    $tone = OoTipTone "t-tip-$($c[0])"
    $l = 0; $d = 0
    if ($tone -match 'light=(\d+) dark=(\d+)') { $l = [int]$Matches[1]; $d = [int]$Matches[2] }
    if ($c[0] -eq 'dark') { Check 'T the tip on the dark page is dark although Windows is light' ($ok -and $d -gt 200 -and $d -gt $l * 3) "settings open: $ok; $tone" }
    else { Check 'T the tip on the light page is light although Windows is dark' ($ok -and $l -gt 200 -and $l -gt $d * 3) "settings open: $ok; $tone" }
    OoCloseSettings
    PfQuit
    Remove-Item Env:\PECOFENCE_WEBVIEW_ARGS -ErrorAction SilentlyContinue
  }
  OoWinMode $script:OoWas
}
if ($Part -match 'T') { OoTips }

# ---- S: a look at a first start while C: keeps every write waiting (the stall of part S of
# owner3-ux3-firstrun.ps1: a 6 GB file copied by the shell from a helper process): how long
# until the first-start card shows (39 to 41 s on these boxes; the app's log has where it
# waits: between `z-order after show` and `watchdog started`). The job's own log would wait
# for the disk too: nothing is written while the start is watched.
function OoSlowStart() {
  trap { PfTrap $_; continue }
  Case 'S look: a first start while the disk keeps writes waiting'
  if (PfAlive) { PfQuit }
  Remove-Item -Recurse -Force "$PfApp\config", "$PfApp\notices.json" -ErrorAction SilentlyContinue
  if (-not (Test-Path "$PfApp\pecofence.exe")) { New-Item -ItemType Directory -Force $PfApp | Out-Null; Copy-Item "$PfBin\*" $PfApp -Force }
  New-Item -ItemType Directory -Force "$PfRun\big" | Out-Null
  $f = [IO.File]::Create("$PfRun\big\oo-big.bin"); $f.SetLength([int64]6 * 1GB); $f.Close()
  ParkMouse 800 850
  $cmd = "Add-Type -AssemblyName Microsoft.VisualBasic; [Microsoft.VisualBasic.FileIO.FileSystem]::CopyFile('$PfRun\big\oo-big.bin', '$PfRun\big\oo-copy.bin', 'AllDialogs')"
  $sw = [Diagnostics.Stopwatch]::StartNew()
  $copy = Start-Process powershell -PassThru -WindowStyle Hidden -ArgumentList '-NoProfile', '-Command', $cmd
  Start-Sleep -Seconds 6
  $env:PECOFENCE_INSTANCE = $PfInst
  $env:RUST_LOG = 'pecofence=debug'
  $t0 = $sw.ElapsedMilliseconds
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $PfApp
  $tCard = -1
  while ($sw.ElapsedMilliseconds - $t0 -lt 60000) {
    $t = $sw.ElapsedMilliseconds - $t0
    $nw = [G]::FindWindowW('PecoFence.Notice', [NullString]::Value)
    if ($nw -ne [IntPtr]::Zero -and [G]::IsWindowVisible($nw)) { $tCard = $t; break }
    Start-Sleep -Milliseconds 100
  }
  $copyDone = $copy.WaitForExit(180000)
  Start-Sleep -Seconds 2
  $wait = @(PfLogAll | Where-Object { $_ -match 'z-order after show|watchdog started' } | ForEach-Object { ($_ -split ' ')[0] } | Select-Object -First 2) -join ' to '
  Check 'S the first-start card came' ($tCard -ge 0) "the card came $tCard ms after the start; the app's log between the fences being shown and the watchdog started: $wait; the copy ended=$copyDone after $($sw.ElapsedMilliseconds) ms"
  if (PfAlive) { PfQuit }
  Remove-Item -Recurse -Force "$PfRun\big" -ErrorAction SilentlyContinue
}
if ($Part -match 'S') { OoSlowStart }

# ---- A: a theme file's accent beside text of the other colour mode
function OoWall($r, $g, $b) {
  $bm = New-Object Drawing.Bitmap 32, 32; $gr = [Drawing.Graphics]::FromImage($bm); $gr.Clear([Drawing.Color]::FromArgb($r, $g, $b)); $gr.Dispose()
  $p = "$PfRun\wall-$r.bmp"; $bm.Save($p, [Drawing.Imaging.ImageFormat]::Bmp); $bm.Dispose()
  [G]::SystemParametersInfoW(0x14, 0, $p, 3) | Out-Null
}
# The most saturated pixel of the title row of fence $title: the bar under the shown tab.
function OoMark($title) {
  $w = DumpOf $title 4
  $b = [Px]::Grab($w.l + 4, $w.t + 2, 180, 34); $best = $null; $sat = -1
  foreach ($y in 0..33) {
    foreach ($x in 0..179) {
      $c = $b.GetPixel($x, $y); $s = [Math]::Max($c.R, [Math]::Max($c.G, $c.B)) - [Math]::Min($c.R, [Math]::Min($c.G, $c.B))
      if ($s -gt $sat) { $sat = $s; $best = $c }
    }
  }
  $b.Dispose(); return $best
}
function OoAccent() {
  trap { PfTrap $_; continue }
  Case 'A setup'
  $script:OoWallWas = (Get-ItemProperty 'HKCU:\Control Panel\Desktop').WallPaper
  foreach ($n in $OoDocs) { DeskFile $n | Out-Null }
  PfStart -Lang zh-CN -Script (DumpScript 500 8 3000)
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  ClearFences | Out-Null
  Pf settings set featureTips false | Out-Null
  Pf settings set tools.enabled true | Out-Null
  New-Item -ItemType Directory -Force "$PfApp\config\themes" | Out-Null
  Set-Content -LiteralPath "$PfApp\config\themes\oo-orange.json" -Value '{ "base": "fluent", "accent": "#E8710A" }' -Encoding ASCII
  if ($PfExtra -and (Test-Path "$PfExtra\warm.json")) { Copy-Item "$PfExtra\warm.json" "$PfApp\config\themes\warm.json" -Force }
  Pf fence create --title '文档' --rect '200,120,340,240' | Out-Null
  Pf item move --glob '*.pdf' --to '文档' | Out-Null
  Pf fence create --title '文字' --rect '200,420,340,200' | Out-Null
  Pf item move --glob '*.txt' --to '文字' | Out-Null
  Pf fence merge '文字' --into '文档' | Out-Null
  Pf fence create --tool note --title '便签' --text "https://example.com/docs 文档`n[x] 续费域名`n[ ] 给安娜打电话" --rect '600,120,320,200' | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse 1300 800

  # The bar under the shown tab, a ticked check box and a link take the theme file's accent
  # in the shade for the text (they took Windows' own accent, blue, where the text had
  # changed to the other colour mode's because of the wallpaper).
  foreach ($c in @(@('dark', 236, 239, 243, 'oo-orange'), @('light', 28, 32, 44, 'oo-orange'), @('dark', 236, 239, 243, 'warm'))) {
    Case "A accent of the theme file $($c[4]): $($c[0]) mode, wallpaper $($c[1]),$($c[2]),$($c[3])"
    OoWall $c[1] $c[2] $c[3]
    Pf settings set theme $c[0] | Out-Null
    Pf settings set customTheme $c[4] --string | Out-Null
    # PecoFence looks at the wallpaper every 5 seconds.
    Start-Sleep -Seconds 8
    ParkMouse 1300 800
    $m = OoMark '文档'
    Crop "a-$($c[4])-$($c[0])" 190 110 740 260 | Out-Null
    Check "A the bar under the shown tab has the theme's hue ($($c[4]), $($c[0]) mode)" ($m.R -gt $m.B + 30) "mark $($m.R),$($m.G),$($m.B)"
  }
  Pf settings set customTheme null | Out-Null
  PfQuit
}
if ($Part -match 'A') { OoAccent }

Case 'cleanup'
if (PfAlive) { PfQuit }
Remove-Item -Recurse -Force "$PfRun\big" -ErrorAction SilentlyContinue
# The wallpaper the box had.
if ($null -ne $script:OoWallWas) { [G]::SystemParametersInfoW(0x14, 0, $script:OoWallWas, 3) | Out-Null }
if ($script:OoTouched) { OoWinMode $script:OoWas }
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
[G]::KeyUp(0x11); [G]::KeyUp(0x12)
OoClean
Check 'test files removed' (@(Get-ChildItem $PfDesk | Where-Object { $OoDocs -contains $_.Name -or $_.Name -match '\.fq[abc]$' }).Count -eq 0) ''
