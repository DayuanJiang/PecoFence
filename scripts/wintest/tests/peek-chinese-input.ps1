# Typing Chinese in the Peek search with Microsoft Pinyin (needs Everything: smart.ps1): 季度,
# Backspace to 季, then the single characters 的 文 报 图 新 年, each a query whose newest
# Everything hits are cold programs named with it (robocopy /J copies of System32's .exe, out of
# the file cache), with fences holding Chinese names and a few thousand Chinese file names
# indexed. A probe thread sends WM_NULL to the control window every few ms; each keystroke's
# longest wait is logged. Two rounds: as the disk is (EC2's answers in a millisecond or two),
# then with PECOFENCE_TEST_SLOW_READ_MS=400 standing in for a drive that spins up (debug
# builds). The app's own "a stage was slow" lines are summed per stage. The pf user's language
# list gets zh-Hans-CN for the run and loses it again at the end. Chinese UI, debug build.
$Zh = Join-Path $PfDocs 'pf-zh'
$Cold = "C:\pf-cold-$(Get-Date -Format 'MMddHHmmss')"
$Chars = '的', '文', '报', '图', '新', '年'
# What is typed for each character: a word whose first Pinyin candidate is stable (the IME
# learns single syllables: wen gave 问, then 文), then Backspace down to the character.
$Typed = [ordered]@{ '季' = @('jidubaogao', '季度报告'); '的' = @('de', '的'); '文' = @('wenjian', '文件'); '报' = @('baogao', '报告'); '图' = @('tupian', '图片'); '新' = @('xinnian', '新年'); '年' = @('niandu', '年度') }
Add-Type @"
using System; using System.Collections.Generic; using System.Diagnostics; using System.Threading; using System.Runtime.InteropServices;
public static class UiProbe {
  [DllImport("user32.dll")] static extern IntPtr SendMessageTimeoutW(IntPtr h, uint m, IntPtr w, IntPtr l, uint f, uint t, out IntPtr r);
  static Thread th; static volatile bool run; static List<double> waits = new List<double>();
  public static void Start(IntPtr h) {
    lock (waits) waits.Clear(); run = true;
    th = new Thread(() => { var sw = new Stopwatch(); IntPtr r; while (run) { sw.Restart(); SendMessageTimeoutW(h, 0, IntPtr.Zero, IntPtr.Zero, 0, 10000, out r); double ms = sw.Elapsed.TotalMilliseconds; lock (waits) waits.Add(ms); Thread.Sleep(2); } });
    th.IsBackground = true; th.Start();
  }
  public static double[] Stop() { run = false; th.Join(); lock (waits) return waits.ToArray(); }
}
public static class Ime {
  [DllImport("user32.dll")] static extern IntPtr LoadKeyboardLayoutW(string id, uint flags);
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern bool PostMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, IntPtr pid);
  [DllImport("user32.dll")] static extern IntPtr GetKeyboardLayout(uint thread);
  // Asks the foreground window to switch to Chinese (PRC) with its default IME, Microsoft Pinyin.
  public static void Chinese() { var hkl = LoadKeyboardLayoutW("00000804", 1); PostMessageW(GetForegroundWindow(), 0x50, IntPtr.Zero, hkl); }
  public static string Layout() { var t = GetWindowThreadProcessId(GetForegroundWindow(), IntPtr.Zero); return GetKeyboardLayout(t).ToInt64().ToString("X"); }
}
"@
# The longest wait (ms) while $action runs and 2.5 s after it.
function Waited($ctl, [scriptblock]$action) {
  [UiProbe]::Start($ctl); & $action; Start-Sleep -Milliseconds 2500
  $s = @([UiProbe]::Stop() | Sort-Object)
  if ($s.Count -eq 0) { return -1 }
  return [math]::Round($s[-1], 1)
}
function ClearColdCopies() {
  foreach ($d in @(Get-ChildItem -Path 'C:\' -Directory -Filter 'pf-cold-*' -ErrorAction SilentlyContinue)) {
    try { [IO.Directory]::Delete($d.FullName, $true) } catch { Log "cold copies: $($_.Exception.Message)" }
  }
}
# Pinyin letters, then Space for the first candidate; the longest wait of the whole word.
function TypeChinese($ctl, $keys) {
  # Virtual keys (the IME composes them; a Unicode character would bypass it); Space takes the
  # first candidate.
  return Waited $ctl { foreach ($c in $keys.ToUpper().ToCharArray()) { [G]::Press([uint16][int]$c); Start-Sleep -Milliseconds 120 }; [G]::Press(0x20) }
}
# The "a stage was slow" lines logged since $from (UTC, as the log writes it), each once (the
# kit keeps copies of earlier logs), as "stage n× max ms".
function StageSummary($from) {
  $lines = @(PfLogAll | Where-Object { $_ -match 'a (stage|message) was slow stage="\w+" ms=\d+ us=\d+' -and $_.Length -gt 27 -and $_.Substring(0, 27) -ge $from } | Sort-Object -Unique)
  $by = @{}
  foreach ($l in $lines) { if ($l -match 'stage="(\w+)" ms=\d+ us=(\d+)') { $k = $Matches[1]; $v = [int]$Matches[2] / 1000.0; if (-not $by[$k]) { $by[$k] = @() }; $by[$k] += $v } }
  return (($by.Keys | Sort-Object | ForEach-Object { "$_ $(@($by[$_]).Count)x max $([math]::Round((@($by[$_]) | Measure-Object -Maximum).Maximum, 1)) ms" }) -join '; ')
}
# Does the box's own Microsoft Pinyin compose at all? Typed into Notepad: "nihao" and Space must
# give 你好. On the EC2 boxes the input method's process hangs now and then (2026-11-12: three
# boxes); then no program takes Chinese, every key waits 10 s for it and arrives as a plain
# letter, in PecoFence's search box as in Notepad. Returns the text Notepad holds and how long
# Notepad took to answer after the word.
function PinyinInNotepad() {
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Process notepad
  $np = WaitWin 'Notepad' 10
  [L]::MoveWindow($np, 500, 300, 500, 300, $true) | Out-Null
  Start-Sleep -Milliseconds 800
  [G]::Click(750, 480); Start-Sleep -Milliseconds 500
  for ($i = 0; $i -lt 2 -and [Ime]::Layout() -notmatch '804$'; $i++) {
    [G]::KeyDown(0xA4); [G]::KeyDown(0xA0); Start-Sleep -Milliseconds 60; [G]::KeyUp(0xA0); [G]::KeyUp(0xA4); Start-Sleep -Milliseconds 800
  }
  $layout = [Ime]::Layout()
  foreach ($c in 'NIHAO'.ToCharArray()) { [G]::Press([uint16][int]$c); Start-Sleep -Milliseconds 120 }
  [G]::Press(0x20); Start-Sleep -Milliseconds 1500
  # How long Notepad takes to answer now (a hung input method keeps it waiting for seconds).
  $sw = [Diagnostics.Stopwatch]::StartNew(); $res = [IntPtr]::Zero
  $answered = [G]::SendMessageTimeoutW($np, 0, [IntPtr]0, [IntPtr]0, 2, 12000, [ref]$res) -ne [IntPtr]::Zero
  $ms = [int]$sw.ElapsedMilliseconds
  $text = ''
  if ($answered) { $e = [M]::FindWindowExW($np, [IntPtr]::Zero, 'Edit', $null); if ($e -ne [IntPtr]::Zero) { $text = [W]::Text($e) } }
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Milliseconds 500
  return @{ text = $text; ms = $ms; layout = $layout; answered = $answered }
}
function Round($name, $ctl) {
  DeskClick 1500 820 | Out-Null
  PeekKey; Start-Sleep -Milliseconds 800
  # The box opens on a typed letter; take it back, then switch the box to Pinyin.
  [G]::Type('a'); Start-Sleep -Milliseconds 900; [G]::Press(0x08); Start-Sleep -Milliseconds 500
  # Left Alt+Shift, Windows' own hotkey for the next input language (Microsoft Pinyin is a
  # text service: a posted language request does not reach it).
  for ($i = 0; $i -lt 2 -and [Ime]::Layout() -notmatch '804$'; $i++) {
    [G]::KeyDown(0xA4); [G]::KeyDown(0xA0); Start-Sleep -Milliseconds 60; [G]::KeyUp(0xA0); [G]::KeyUp(0xA4); Start-Sleep -Milliseconds 800
  }
  Log "$name keyboard layout: $([Ime]::Layout())"
  $from = [DateTime]::UtcNow.ToString('yyyy-MM-ddTHH:mm:ss.ffffff') + 'Z'
  $out = @(); $worst = 0
  foreach ($c in $Typed.Keys) {
    $keys, $word = $Typed[$c]
    $w = TypeChinese $ctl $keys; $out += "$keys $w ms '$([M]::BoxText())'"; $worst = [math]::Max($worst, $w)
    # Backspace down to the one character, then to an empty box.
    for ($k = $word.Length; $k -gt 1; $k--) {
      $w = Waited $ctl { [G]::Press(0x08) }; $out += "Backspace $w ms '$([M]::BoxText())'"; $worst = [math]::Max($worst, $w)
    }
    if ($c -eq '季' -or $c -eq '的') { Shot "$name-$c" }
    $w = Waited $ctl { [G]::Press(0x08) }; $out += "Backspace $w ms ''"; $worst = [math]::Max($worst, $w)
  }
  Log "$name per keystroke: $($out -join '; ')"
  $stages = StageSummary $from
  Log "$name slow stages: $stages"
  for ($i = 0; $i -lt 3; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 400 }
  return @{ worst = $worst; text = ($out -join '; '); stages = $stages }
}

Case 'setup'
CloseWindows
ClearColdCopies
Check 'Everything runs' (EvStart) "installed=$(EvInstalled)"
$langs = Get-WinUserLanguageList
$script:AddedZh = -not ($langs | Where-Object { $_.LanguageTag -like 'zh-Hans*' })
if ($AddedZh) { $langs.Add('zh-Hans-CN'); Set-WinUserLanguageList $langs -Force }
Check 'Microsoft Pinyin is an input method of pf' (@(Get-WinUserLanguageList | Where-Object { $_.InputMethodTips -match '0804:' }).Count -ge 1) "added=$AddedZh"
# The box's input method itself, before PecoFence is asked to take Chinese from it.
$np = PinyinInNotepad
$script:ImeWorks = ($np.text -match '你好')
Check 'the box''s Microsoft Pinyin composes in Notepad (nihao gives 你好); if not, the box is at fault and the Chinese rounds are left out' $script:ImeWorks "Notepad holds '$($np.text)', layout $($np.layout), answered=$($np.answered) after $($np.ms) ms"
# A few thousand Chinese file names to index, a folder fence with some of them, desktop files.
$words = '季度报告', '年度预算', '工作计划', '会议记录', '新年活动', '文件整理', '图片素材', '报销单', '发票', '合同', '项目进度', '的说明'
New-Item -ItemType Directory -Force $Zh | Out-Null
for ($i = 0; $i -lt 3000; $i++) { [IO.File]::WriteAllText((Join-Path $Zh ('{0} {1:D4}.txt' -f $words[$i % $words.Count], $i)), 'x') }
$fold = Join-Path $PfDocs 'pf-zh-fence'
New-Item -ItemType Directory -Force $fold | Out-Null
for ($i = 0; $i -lt 120; $i++) { [IO.File]::WriteAllText((Join-Path $fold ('{0} {1:D3}.docx' -f $words[$i % $words.Count], $i)), 'x') }
foreach ($n in '季度报告.docx', '新年计划.xlsx', '图片说明.txt', '文档模板.docx') { Set-Content "$PfDesk\$n" 'sample' }
# Cold programs named with each character (and 季度), dated now: the newest hits.
robocopy C:\Windows\System32 "$Cold\a" *.exe /J /COPY:D /R:0 /W:0 /NFL /NDL /NJH /NJS | Out-Null
$tags = @('季度') + $Chars
$now = Get-Date; $n = 0
foreach ($f in @(Get-ChildItem -LiteralPath "$Cold\a" -File)) {
  $to = Join-Path $f.DirectoryName "$($tags[$n % $tags.Count])$($f.Name)"
  try { $f.Attributes = 'Normal'; [IO.File]::Move($f.FullName, $to); [IO.File]::SetLastWriteTime($to, $now.AddSeconds(-($n % 600))); $n++ } catch { }
}
Check 'cold programs named with the characters' ($n -ge 300) "files=$n"
Start-Sleep -Seconds 5
# Debug builds log every stage with this (the numbers per stage of the first round).
$env:PECOFENCE_TEST_SLOW_MS = '0'
PfStart -Lang zh-CN
Remove-Item Env:\PECOFENCE_TEST_SLOW_MS
Pf settings set tools.everything true | Out-Null
Pf fence create --portal $fold --title '工作文件' --rect '24,420,420,300' | Out-Null
Start-Sleep -Seconds 3
$ctl = [G]::FindWindowW('PecoFence.Control', [NullString]::Value)
Check 'the control window is there' ($ctl -ne [IntPtr]::Zero) "$ctl"

Case 'Chinese input, the disk as it is'
if (-not $script:ImeWorks) { Log 'the box''s input method does not compose: the Chinese rounds are left out' }
$r = if ($script:ImeWorks) { Round 'disk' $ctl } else { @{ worst = 0; text = "left out: the box's input method does not compose"; stages = '' } }
Check 'Pinyin reached the box (季度报告 down to 季, then 的 文 报 图 新 年)' (-not $script:ImeWorks -or ($r.text -match "'季度报告'" -and $r.text -match "'季度'" -and $r.text -match "'季'" -and @($Chars | Where-Object { $r.text -match "'$_'" }).Count -eq $Chars.Count)) $r.text
Check 'disk as it is: no keystroke keeps the UI thread over 100 ms' ($r.worst -lt 100) "worst $($r.worst) ms; stages: $($r.stages)"

Case 'Chinese input, a drive that spins up: each own icon 400 ms'
PfQuit
$env:PECOFENCE_TEST_SLOW_READ_MS = '400'
PfStart -Lang zh-CN
Remove-Item Env:\PECOFENCE_TEST_SLOW_READ_MS
Start-Sleep -Seconds 3
$ctl = [G]::FindWindowW('PecoFence.Control', [NullString]::Value)
$r = if ($script:ImeWorks) { Round 'slow' $ctl } else { @{ worst = 0; text = "left out: the box's input method does not compose"; stages = '' } }
Check 'slow drive: no keystroke keeps the UI thread over 100 ms' ($r.worst -lt 100) "worst $($r.worst) ms; stages: $($r.stages)"

Case 'cleanup'
PfQuit
ClearColdCopies
foreach ($d in $Zh, (Join-Path $PfDocs 'pf-zh-fence')) { if (Test-Path -LiteralPath $d) { [IO.Directory]::Delete($d, $true) } }
foreach ($n in '季度报告.docx', '新年计划.xlsx', '图片说明.txt', '文档模板.docx') { Remove-Item -LiteralPath "$PfDesk\$n" -ErrorAction SilentlyContinue }
if ($AddedZh) {
  $langs = Get-WinUserLanguageList
  $zh = $langs | Where-Object { $_.LanguageTag -like 'zh-Hans*' }
  foreach ($x in @($zh)) { [void]$langs.Remove($x) }
  Set-WinUserLanguageList $langs -Force
}
Check 'pf''s languages are as before' (-not $AddedZh -or -not (Get-WinUserLanguageList | Where-Object { $_.LanguageTag -like 'zh-Hans*' })) ''
