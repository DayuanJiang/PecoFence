# Owner round 2026-11-12, package search-core: the Peek search box.
# PK-1: with Microsoft Pinyin in Chinese mode, the first key typed during a Peek opens the search
#       box at once and the word is composed in it (it was composed in the input method's own
#       window at the screen's top-left corner, the box opened only when a candidate was taken).
# PK-4: a list with more matches than rows scrolls: Down past the last row shown, the wheel,
#       PageDown / PageUp, End / Home, Ctrl+digit on the rows shown, a thumb at the right.
# PK-3: the box moves aside when it would cover a fence that holds highlighted matches, once.
# Parts (run.sh --var Part=...): A PK-1, B PK-4, C PK-3. Default: all. Record=1 also records
# the clips for the report. Keys are pressed with their scan codes, as a keyboard sends them.
if (-not $Part) { $Part = 'ABC' }
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
  $l = @(PfLogAll | Where-Object { $_ -match 'search list laid out' }) | Select-Object -Last 1
  $h = @{}
  if ($l) { foreach ($m in [regex]::Matches($l, '(\w+)=(Some\((\d+)\)|[^\s]+)')) { $h[$m.Groups[1].Value] = if ($m.Groups[3].Success) { $m.Groups[3].Value } else { $m.Groups[2].Value } } }
  return $h
}
function ListText() { $h = ListLine; return (($h.Keys | Sort-Object | ForEach-Object { "$_=$($h[$_])" }) -join ' ') }
# The file a row opened last ("search row opened row=Fence(n)").
function OpenedRow() {
  $l = @(PfLogAll | Where-Object { $_ -match 'search row opened' }) | Select-Object -Last 1
  if ($l -match 'row=Fence\((\d+)\)') { return [int]$Matches[1] }
  return -1
}
$ScDir = Join-Path $PfDocs 'pf-owner-search'
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
  if (Test-Path -LiteralPath $ScDir) { [IO.Directory]::Delete($ScDir, $true) }
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
  if ($Record) { RecStart 'PK-1' -X 0 -Y 0 -W 1136 -H 560 -Fps 15; Start-Sleep -Milliseconds 300; RecMark '按速览快捷键' }
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
  ScKeys 'dan'; Start-Sleep -Milliseconds 500; ScPress 0x20; Start-Sleep -Milliseconds 900
  Check 'a second word is composed in the box as before (报告单)' ([M]::BoxText() -eq '报告单') "text='$([M]::BoxText())'"
  if ($Record) { Start-Sleep -Milliseconds 600; RecStop }
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

[Anim]::Set($ScAnim)
