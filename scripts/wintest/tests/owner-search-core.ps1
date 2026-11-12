# Owner round 2026-11-12, package search-core: the Peek search box.
# PK-1: with Microsoft Pinyin in Chinese mode, the first key typed during a Peek opens the search
#       box at once and the word is composed in it (it was composed in the input method's own
#       window at the screen's top-left corner, the box opened only when a candidate was taken).
# PK-4: a list with more matches than rows scrolls: Down past the last row shown, the wheel,
#       PageDown / PageUp, End / Home, Ctrl+digit on the rows shown, a thumb at the right.
# Parts (run.sh --var Part=...): A PK-1, B PK-4. Default: all. Record=1 also records
# the clips for the report. Keys are pressed with their scan codes, as a keyboard sends them.
if (-not $Part) { $Part = 'AB' }
trap { PfTrap $_; continue }
. C:\pf\kit\rec.ps1

Add-Type @"
using System; using System.Text; using System.Collections.Generic; using System.Runtime.InteropServices;
public static class Sc {
  [DllImport("user32.dll")] static extern uint MapVirtualKeyW(uint code, uint type);
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, IntPtr pid);
  [DllImport("user32.dll")] static extern IntPtr GetKeyboardLayout(uint thread);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out R r);
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, R2, B; }
  delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc f, IntPtr l);
  public static ushort Scan(ushort vk) { return (ushort)MapVirtualKeyW(vk, 0); }
  public static string Layout() { var t = GetWindowThreadProcessId(GetForegroundWindow(), IntPtr.Zero); return GetKeyboardLayout(t).ToInt64().ToString("X"); }
  // Visible top-level windows whose class reads like an input method's (candidates, composition).
  public static string ImeWindows() {
    var list = new List<string>();
    EnumWindows((h, l) => {
      if (!IsWindowVisible(h)) return true;
      var s = new StringBuilder(256); GetClassNameW(h, s, 256); var c = s.ToString();
      if (c.IndexOf("IME", StringComparison.OrdinalIgnoreCase) < 0 && c.IndexOf("Candidate", StringComparison.OrdinalIgnoreCase) < 0 && c.IndexOf("TextInput", StringComparison.OrdinalIgnoreCase) < 0) return true;
      R r; GetWindowRect(h, out r);
      if (r.R2 - r.L <= 1 || r.B - r.T <= 1) return true;
      list.Add(c + " " + r.L + "," + r.T + "," + r.R2 + "," + r.B);
      return true;
    }, IntPtr.Zero);
    return string.Join(" | ", list.ToArray());
  }
}
"@
# A key press with its scan code (the kit's Press sends none).
function ScPress($vk) { $s = [Sc]::Scan($vk); [L]::ScanKey($vk, $s, $false); Start-Sleep -Milliseconds 30; [L]::ScanKey($vk, $s, $true) }
function ScKeys($s, $ms = 140) { foreach ($c in $s.ToUpper().ToCharArray()) { ScPress ([uint16][int]$c); Start-Sleep -Milliseconds $ms } }
function BoxH() { $h = [G]::FindWindowW('PecoFence.PeekSearch', [NullString]::Value); if ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) { return $h }; return [IntPtr]::Zero }
function BoxRect() { $h = BoxH; if ($h -eq [IntPtr]::Zero) { return $null }; return (Rect $h) }
function RectText($r) { if (-not $r) { return 'none' }; return "$($r.L),$($r.T),$($r.R),$($r.B)" }
function PeekOn() { return ([W]::Tops('PecoFence.PeekDimmer')).Count -gt 0 }
function EndPeekHard() { for ($i = 0; $i -lt 4 -and (PeekOn); $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 450 } }
# Left Alt+Shift (Windows' hotkey for the next input language) until the layout is / is not Chinese.
function ToLayout([bool]$chinese) {
  for ($i = 0; $i -lt 3 -and (([Sc]::Layout() -match '804$') -ne $chinese); $i++) {
    [G]::KeyDown(0xA4); [G]::KeyDown(0xA0); Start-Sleep -Milliseconds 60; [G]::KeyUp(0xA0); [G]::KeyUp(0xA4); Start-Sleep -Milliseconds 900
  }
  return [Sc]::Layout()
}
# The last "search list laid out" line of the app log as a table of its fields.
function ListLine() {
  $l = @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match 'search list laid out' }) | Select-Object -Last 1
  $h = @{}
  if ($l) { foreach ($m in [regex]::Matches($l, '(\w+)=(Some\((\d+)\)|[^\s]+)')) { $h[$m.Groups[1].Value] = if ($m.Groups[3].Success) { $m.Groups[3].Value } else { $m.Groups[2].Value } } }
  return $h
}
function ListText() { $h = ListLine; return (($h.Keys | Sort-Object | ForEach-Object { "$_=$($h[$_])" }) -join ' ') }
# The file a row opened last ("search row opened row=Fence(n)").
function OpenedRow() {
  $l = @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match 'search row opened' }) | Select-Object -Last 1
  if ($l -match 'row=Fence\((\d+)\)') { return [int]$Matches[1] }
  return -1
}
$ScDir = Join-Path $PfDocs 'pf-owner-search'
# Files with 报 that are in no fence (the Everything case of part B).
$ScPc = Join-Path $PfDocs 'pf-owner-search-pc'
# Chinese names of mixed length, all with 报 in them, all text files (Enter opens Notepad).
$ScNames = '报告.txt', '报销单 11月.txt', '年度报告 2025 董事会审阅版（含附录和财务报表）.txt', '季度报告（终稿）.txt', '报价单-客户A.txt', '月报.txt', '周报 第45周.txt',
  '日报 1108.txt', '情报汇总.txt', '报表.txt', '简报 第3期.txt', '汇报材料（最终版）修订2.txt', '报名表.txt', '通报.txt'
function ScSeed() {
  if (Test-Path -LiteralPath $ScDir) { [IO.Directory]::Delete($ScDir, $true) }
  New-Item -ItemType Directory -Force $ScDir | Out-Null
  foreach ($n in $ScNames) { Set-Content -LiteralPath (Join-Path $ScDir $n) -Value "sample $n" -Encoding UTF8 }
}
function ScClean() {
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  foreach ($d in $ScDir, $ScPc) { if (Test-Path -LiteralPath $d) { [IO.Directory]::Delete($d, $true) } }
}
$ScAnim = [Anim]::Get()
[Anim]::Set($true)
ScClean

# ---------------------------------------------------------------- A: PK-1
function PartA() {
  trap { PfTrap $_; continue }
  Case 'PK-1 setup: Microsoft Pinyin for pf, a folder fence, Notepad in Chinese mode'
  CloseWindows
  ScSeed
  $langs = Get-WinUserLanguageList
  $script:AddedZh = -not ($langs | Where-Object { $_.LanguageTag -like 'zh-Hans*' })
  if ($AddedZh) { $langs.Add('zh-Hans-CN'); Set-WinUserLanguageList $langs -Force }
  Check 'Microsoft Pinyin is an input method of pf' (@(Get-WinUserLanguageList | Where-Object { $_.InputMethodTips -match '0804:' }).Count -ge 1) "added=$AddedZh"
  PfStart -Lang zh-CN
  ClearFences | Out-Null
  Pf fence create --portal $ScDir --title '工作文件' --rect '1180,60,400,420' | Out-Null
  Start-Sleep -Seconds 2
  Start-Process notepad; Start-Sleep -Seconds 2
  $np = (Get-Process notepad | Select-Object -First 1).MainWindowHandle
  [L]::MoveWindow($np, 240, 420, 620, 360, $true) | Out-Null
  $nr = Rect $np
  [G]::Click($nr.L + 200, $nr.T + 200); Start-Sleep -Milliseconds 500
  $lay = ToLayout $true
  ScKeys 'nihao'; ScPress 0x20; Start-Sleep -Milliseconds 700
  $ed = [M]::FindWindowExW($np, [IntPtr]::Zero, 'Edit', [NullString]::Value)
  $sb = New-Object Text.StringBuilder 256; [M]::SendMessageW($ed, 0x000D, [IntPtr]256, $sb) | Out-Null
  Check 'Notepad takes Chinese from Pinyin (nihao, Space)' ($lay -match '804$' -and "$sb" -match '你好') "layout=$lay notepad='$sb'"

  Case 'PK-1 Chinese mode: the first key opens the box and the word is composed in it'
  if ($Record) { RecStart 'PK-1' -X 0 -Y 0 -W 1600 -H 560 -Fps 15; Start-Sleep -Milliseconds 300; RecMark '按速览快捷键' }
  PeekKey; Start-Sleep -Milliseconds 1000
  $corner0 = [Px]::Grab(0, 0, 620, 110)
  $lay = [Sc]::Layout()
  if ($Record) { RecMark '按 B（中文输入法）' }
  ScPress 0x42; Start-Sleep -Milliseconds 800
  $r1 = BoxRect
  Check 'B: the search box is open, nothing committed yet' ($r1 -and [M]::BoxText() -eq '') "layout=$lay box=$(RectText $r1) text='$([M]::BoxText())' fg=$(Fg)"
  if ($Record) { RecMark '接着按 A O G A O' }
  ScKeys 'aogao'; Start-Sleep -Milliseconds 800
  $corner1 = [Px]::Grab(0, 0, 620, 110)
  $diff = [math]::Round([Px]::Diff($corner0, $corner1, 0, 620, 0, 110), 2)
  $ime = [Sc]::ImeWindows()
  Shot 'A1-composing'
  $r2 = BoxRect
  Check 'B A O G A O: still composing in the open box' ($r2 -and [M]::BoxText() -eq '') "box=$(RectText $r2) text='$([M]::BoxText())'"
  Check 'nothing of the input method shows in the screen''s top-left corner' ($diff -lt 0.5) "corner changed by $diff; input method windows: $ime"
  if ($r2) { Crop 'A1-box' ([math]::Max(0, $r2.L - 20)) ([math]::Max(0, $r2.T - 20)) ($r2.R - $r2.L + 40) 260 | Out-Null }
  Crop 'A1-corner' 0 0 620 110 | Out-Null
  if ($Record) { RecMark '按空格选第一个候选' }
  ScPress 0x20; Start-Sleep -Milliseconds 1000
  $r3 = BoxRect
  Check 'Space takes the first candidate: 报告 is in the box and the list shows' ($r3 -and [M]::BoxText() -eq '报告' -and ($r3.B - $r3.T) -gt 150) "box=$(RectText $r3) text='$([M]::BoxText())'"
  Shot 'A2-committed'
  if ($Record) { Start-Sleep -Milliseconds 600; RecStop }
  ScKeys 'dan'; Start-Sleep -Milliseconds 500; ScPress 0x20; Start-Sleep -Milliseconds 900
  Check 'a second word is composed in the box as before (报告单)' ([M]::BoxText() -eq '报告单') "text='$([M]::BoxText())'"
  EndPeekHard
  Check 'Esc ends the Peek' (-not (PeekOn)) ''
  Check 'the app logged no error' (@(PfLogErrors).Count -eq 0) "$(@(PfLogErrors) | Select-Object -First 2)"

  Case 'PK-1 Chinese mode: Enter keeps the letters, punctuation, the other keys of a Peek'
  PeekKey; Start-Sleep -Milliseconds 1000
  ScKeys 'bg'; Start-Sleep -Milliseconds 600
  $rb = BoxRect
  ScPress 0x0D; Start-Sleep -Milliseconds 900
  Check 'B G then Enter: the box was open while composing and holds bg' ($rb -and [M]::BoxText() -eq 'bg') "box while composing=$(RectText $rb) text='$([M]::BoxText())'"
  EndPeekHard
  # No fence clicked yet: Right puts the keyboard into the first fence, Enter opens its first icon.
  PeekKey; Start-Sleep -Milliseconds 1000
  $before = @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match 'peek key: the keyboard goes into a fence' }).Count
  ScPress 0x27; Start-Sleep -Milliseconds 700
  $after = @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match 'peek key: the keyboard goes into a fence' }).Count
  Check 'Right arrow: the keyboard goes into a fence, no search box' ($after -eq $before + 1 -and -not (BoxRect)) "log lines $before -> $after box=$(RectText (BoxRect))"
  # A click on an icon, then Delete: the file goes to the Recycle Bin, the Peek stays.
  $p = ItemPt '工作文件' '通报'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600
  ScPress 0x2E; Start-Sleep -Milliseconds 1500
  Check 'Delete on a clicked icon deletes it, the Peek stays' (-not (Test-Path -LiteralPath (Join-Path $ScDir '通报.txt')) -and (PeekOn) -and -not (BoxRect)) "file there=$(Test-Path -LiteralPath (Join-Path $ScDir '通报.txt')) peek=$(PeekOn) box=$(RectText (BoxRect))"
  $p = ItemPt '工作文件' '报表'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600
  ScPress 0x0D; Start-Sleep -Milliseconds 1800
  Check 'Enter on a clicked icon opens it and ends the Peek' ((NotepadTitles) -match '报表' -and -not (PeekOn)) "notepad: $(NotepadTitles) peek=$(PeekOn)"
  Get-Process notepad -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -match '报表' } | Stop-Process -Force
  Start-Sleep -Milliseconds 500
  $np = (Get-Process notepad | Select-Object -First 1).MainWindowHandle
  $nr = Rect $np
  [G]::Click($nr.L + 200, $nr.T + 200); Start-Sleep -Milliseconds 500
  PeekKey; Start-Sleep -Milliseconds 1000
  ScPress 0x1B; Start-Sleep -Milliseconds 700
  Check 'Esc on the dimmer ends the Peek' (-not (PeekOn)) ''

  Case 'PK-1 English mode: the first letter opens the box with the letter in it'
  [G]::Click($nr.L + 200, $nr.T + 200); Start-Sleep -Milliseconds 500
  $lay = ToLayout $false
  PeekKey; Start-Sleep -Milliseconds 1000
  ScPress 0x42; Start-Sleep -Milliseconds 700
  $r = BoxRect
  Check 'English layout, B: the box opens with b' ($r -and [M]::BoxText() -eq 'b') "layout=$lay box=$(RectText $r) text='$([M]::BoxText())'"
  ScKeys 'ao'; Start-Sleep -Milliseconds 600
  Check 'B A O: bao, found by pinyin' ([M]::BoxText() -eq 'bao' -and [int](ListLine).total -ge 5) "text='$([M]::BoxText())' $(ListText)"
  EndPeekHard

  Case 'PK-1 cleanup'
  PfQuit
  ScClean
  if ($AddedZh) {
    $l = Get-WinUserLanguageList
    $keep = New-WinUserLanguageList ($l[0].LanguageTag)
    foreach ($x in $l) { if ($x.LanguageTag -notlike 'zh-Hans*' -and $x.LanguageTag -ne $l[0].LanguageTag) { $keep.Add($x.LanguageTag) } }
    Set-WinUserLanguageList $keep -Force
  }
  Check 'pf''s languages are as before' (-not $AddedZh -or -not (Get-WinUserLanguageList | Where-Object { $_.LanguageTag -like 'zh-Hans*' })) "$((Get-WinUserLanguageList | ForEach-Object { $_.LanguageTag }) -join ', ')"
}
if ($Part -match 'A') { PartA }

# ---------------------------------------------------------------- B: PK-4
# Opens a Peek, types 报 (14 matches in the folder fence) and waits for the list.
function ScSearch() {
  PeekKey; Start-Sleep -Milliseconds 1000
  [G]::Type('报'); Start-Sleep -Milliseconds 1200
}
function ScDown($n) { for ($i = 0; $i -lt $n; $i++) { ScPress 0x28; Start-Sleep -Milliseconds 110 }; Start-Sleep -Milliseconds 450 }
# Enter, then which result opened (its place in the ranked matches, from 0); Notepad is closed again.
function ScEnterRow() {
  ScPress 0x0D; Start-Sleep -Milliseconds 1500
  $r = OpenedRow
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Milliseconds 400
  EndPeekHard
  return $r
}
function PartB() {
  trap { PfTrap $_; continue }
  Case 'PK-4 setup: a folder fence with 14 files that match 报'
  CloseWindows
  ScSeed
  PfStart -Lang zh-CN
  ClearFences | Out-Null
  Pf fence create --portal $ScDir --title '工作文件' --rect '1180,60,400,420' | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse | Out-Null

  Case 'PK-4 the keys reach every match'
  if ($Record) { RecStart 'PK-4' -X 470 -Y 215 -W 660 -H 590 -Fps 15; Start-Sleep -Milliseconds 300; RecMark '速览里输入「报」，有 14 个匹配' }
  ScSearch
  $l = ListLine
  $box0 = BoxRect
  Check '报 finds 14 files and the list shows nine of them' ([int]$l.total -eq 14 -and [int]$l.rows -eq 9 -and "$($l.selected)" -eq '0') (ListText)
  Shot 'B1-top'
  if ($Record) { RecMark '按 9 次向下键' }
  ScDown 9
  $l = ListLine
  Shot 'B2-down9'
  Check 'Down nine times: the list moved by one row, the highlight is on its last row' ("$($l.first)" -eq '1' -and "$($l.selected)" -eq '8') (ListText)
  if ($Record) { Start-Sleep -Milliseconds 600; RecStop }
  $r = ScEnterRow
  Check 'Enter there opens the tenth match' ($r -eq 9) "opened match $r"
  ScSearch
  ScDown 13
  $l13 = ListText; $a = ListLine
  ScDown 1
  $b = ListLine
  Check 'Down 13 times reaches the last match, once more goes back to the first' ("$($a.first)" -eq '5' -and "$($a.selected)" -eq '8' -and "$($b.first)" -eq '0' -and "$($b.selected)" -eq '0') "after 13: $l13; after 14: $(ListText)"
  ScPress 0x22; Start-Sleep -Milliseconds 450; $pd1 = ListLine; $t1 = ListText
  ScPress 0x22; Start-Sleep -Milliseconds 450; $pd2 = ListLine; $t2 = ListText
  ScPress 0x21; Start-Sleep -Milliseconds 450; $pu = ListLine; $t3 = ListText
  Check 'PageDown, PageDown, PageUp: a page at a time, stopping at the last match' ("$($pd1.first)" -eq '1' -and "$($pd1.selected)" -eq '8' -and "$($pd2.first)" -eq '5' -and "$($pd2.selected)" -eq '8' -and "$($pu.first)" -eq '4' -and "$($pu.selected)" -eq '0') "$t1 | $t2 | $t3"
  ScPress 0x24; Start-Sleep -Milliseconds 450; $hm = ListLine; $t1 = ListText
  ScPress 0x23; Start-Sleep -Milliseconds 450; $en = ListLine; $t2 = ListText
  Check 'Home goes to the first match, End to the last' ("$($hm.first)" -eq '0' -and "$($hm.selected)" -eq '0' -and "$($en.first)" -eq '5' -and "$($en.selected)" -eq '8') "$t1 | $t2"
  $r = ScEnterRow
  Check 'Enter after End opens the last match' ($r -eq 13) "opened match $r"

  Case 'PK-4 the wheel, the thumb, Ctrl+digit'
  if ($Record) { RecStart 'PK-4-wheel' -X 470 -Y 215 -W 660 -H 590 -Fps 15; Start-Sleep -Milliseconds 300; RecMark '速览里输入「报」，有 14 个匹配' }
  ScSearch
  $bx = BoxH; $br = BoxRect
  $strip0 = [Px]::Grab($br.R - 8, $br.T + 100, 8, 400)
  if ($Record) { RecMark '在列表上向下滚一格滚轮' }
  WheelAt $bx 300 300 -1; Start-Sleep -Milliseconds 500
  $w1 = ListLine; $t1 = ListText
  $strip1 = [Px]::Grab($br.R - 8, $br.T + 100, 8, 400)
  Shot 'B3-wheel'
  if ($Record) { RecMark '再滚五格' }
  WheelAt $bx 300 300 -5; Start-Sleep -Milliseconds 500
  $w2 = ListLine; $t2 = ListText
  WheelAt $bx 300 300 1; Start-Sleep -Milliseconds 500
  $w3 = ListLine; $t3 = ListText
  $br2 = BoxRect
  Check 'a wheel notch scrolls three rows, the list stops at the last match, a notch up scrolls back' ("$($w1.first)" -eq '3' -and "$($w2.first)" -eq '5' -and "$($w3.first)" -eq '2') "$t1 | $t2 | $t3"
  $moved = [math]::Round([Px]::Diff($strip0, $strip1, 0, 8, 0, 400), 2)
  Check 'the thumb at the right edge moves with the list' ($moved -gt 0.3) "right edge strip changed by $moved"
  Check 'the box keeps its place and size while the list scrolls' ((RectText $br) -eq (RectText $br2)) "$(RectText $br) -> $(RectText $br2)"
  if ($Record) { Start-Sleep -Milliseconds 500; RecStop }
  # The rows shown are matches 2 to 10: Ctrl+2 opens the second of them.
  [G]::KeyDown(0x11); Start-Sleep -Milliseconds 40; ScPress 0x32; Start-Sleep -Milliseconds 40; [G]::KeyUp(0x11); Start-Sleep -Milliseconds 1500
  $r = OpenedRow
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Milliseconds 400
  EndPeekHard
  Check 'Ctrl+2 opens the second row shown (the fourth match)' ($r -eq 3) "opened match $r"
  ScSearch
  ScDown 9
  [G]::Type('告'); Start-Sleep -Milliseconds 1200
  $n = ListLine
  Check 'typing on (报告, 3 matches) shows the short list from its top' ([int]$n.total -eq 3 -and [int]$n.rows -eq 3 -and [int]$n.first -eq 0 -and [M]::BoxText() -eq '报告') "$(ListText) text='$([M]::BoxText())'"
  # Home and End stay the text box's keys when the list has nothing to do with them.
  ScPress 0x24; Start-Sleep -Milliseconds 300; [G]::Type('年'); Start-Sleep -Milliseconds 900
  Check 'Home with the highlight on the first row moves the caret (年 typed at the start)' ([M]::BoxText() -eq '年报告') "text='$([M]::BoxText())'"
  EndPeekHard

  Case 'PK-4 with the whole computer (Everything): fence matches, then its results, the two action rows stay'
  $evRan = EvRunning
  if (EvStart) {
    # Twelve more files with 报 that are in no fence; Everything needs a moment to index them.
    New-Item -ItemType Directory -Force $ScPc | Out-Null
    for ($i = 1; $i -le 12; $i++) { Set-Content -LiteralPath (Join-Path $ScPc ('报刊 第{0:D2}期.txt' -f $i)) -Value 'x' -Encoding UTF8 }
    Pf settings set tools.everything true | Out-Null
    Start-Sleep -Seconds 4
    ScSearch
    for ($i = 0; $i -lt 16 -and [int](ListLine).action_glyphs -ne 2; $i++) { Start-Sleep -Milliseconds 500 }
    $e0 = ListLine; $t0 = ListText; $eb0 = BoxRect
    # Everything may know more files with 报 (shortcuts in Recent): at least the twelve.
    $win = [int]$e0.rows - 2; $of = [int]$e0.of
    Check 'the list holds the 14 fence matches and Everything''s files (12 or more), with the two action rows under them' ($of -ge 26 -and [int]$e0.action_glyphs -eq 2 -and $win -ge 5 -and "$($e0.first)" -eq '0') $t0
    Shot 'B4-everything-top'
    ScDown 14
    $e1 = ListLine; $t1 = ListText; $eb1 = BoxRect
    Shot 'B5-everything-scrolled'
    Check 'Down 14 times: the first of Everything''s files, the action rows still there, the box as high as before' ([int]$e1.first -eq 15 - $win -and "$($e1.selected)" -eq "$($win - 1)" -and [int]$e1.action_glyphs -eq 2 -and (RectText $eb0) -eq (RectText $eb1)) "$t1 | box $(RectText $eb0) -> $(RectText $eb1)"
    ScPress 0x23; Start-Sleep -Milliseconds 450
    $e2 = ListLine; $t2 = ListText
    ScPress 0x26; Start-Sleep -Milliseconds 300; ScPress 0x26; Start-Sleep -Milliseconds 450
    $e3 = ListLine; $t3 = ListText
    Check 'End: the last action row; Up twice from there: the last file, the list at its end' ("$($e2.selected)" -eq "$($win + 1)" -and "$($e3.selected)" -eq "$($win - 1)" -and [int]$e3.first -eq $of - $win -and (RectText $eb0) -eq (RectText (BoxRect))) "$t2 | $t3"
    EndPeekHard
    Pf settings set tools.everything false | Out-Null
    if (-not $evRan) { EvStop | Out-Null }
  } else {
    Check 'Everything runs (the case needs it)' $false "installed=$(EvInstalled)"
  }
  Check 'the app logged no error' (@(PfLogErrors).Count -eq 0) "$(@(PfLogErrors) | Select-Object -First 2)"

  Case 'PK-4 cleanup'
  PfQuit
  ScClean
}
if ($Part -match 'B') { PartB }

[Anim]::Set($ScAnim)
