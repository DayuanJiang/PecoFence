# Smart fences and fences per virtual desktop: what the hunt of 2026-11-12 found (Everything is
# installed on the box, see smart.ps1). --var Part=A to H runs one part (default: all but G on a release build, G on a debug build).
#   A  a smart fence that waits for Everything: started outside PecoFence, Everything is found
#      within seconds; a click on 「启动 Everything」 changes the fence at once
#   B  a smart fence whose folder is gone says that the folder cannot be found
#   C  a fence sent to another virtual desktop from its menu: the notice has 撤销
#   D  without Everything a new file shows in a smart fence within half a second
#   E  a fence that comes to a desktop (its own was closed, or it was set to show here) does not
#      land on a fence that stands at the same spot there
#   F  PecoFence starts while Everything is there but has not read its index (a stand-in window
#      that answers like one): the fence is not left empty until the next once-a-minute search
#   G  (a debug build: run.sh --bin target/wintest-bin-debug) the fence picker of Settings says
#      which fence is a smart fence and which shows on one virtual desktop only
#   H  a smart fence whose results all go fades to its empty state (Windows' animations on)
if (-not $Part) { $Part = if ($PfDebugBuild) { 'G' } else { 'ABCDEFH' } }
$HsDir = "$PfDocs\智能栅栏检查"
function HsFiles() {
  Remove-Item -Recurse -Force $HsDir -ErrorAction SilentlyContinue
  AgedFile $HsDir '会议纪要 11月.txt' 1 | Out-Null
  AgedFile $HsDir '年度报告 2025 董事会审阅版（含附录和财务报表）.pdf' 2 | Out-Null
  AgedFile $HsDir 'a.txt' 3 | Out-Null
}
# The words Windows' OCR reads in a fence (English interface: the box has no Chinese OCR).
function FenceWords($title, $name) { $r = (FenceGet $title).windowRect; return (OcrText $r.x $r.y $r.w $r.h $name) }

function PartA {
  trap { PfTrap $_; continue }
  Case 'A: setup'
  CloseWindows
  HsFiles
  EvStop | Out-Null
  PfStart -Lang en -Script (DumpScript 500 20)
  Pf settings set theme light | Out-Null
  ClearFences
  Pf fence create --title 'Whole PC' --everything '会议纪要' --rect '60,60,440,220' | Out-Null
  Check 'A the fence waits for Everything' (WaitSmart 'Whole PC' { $s.source -eq 'needsEverything' } 10) (SmartLine 'Whole PC')
  [G]::MoveTo(1200, 800)

  Case 'A: Everything started outside PecoFence'
  # Well before the first once-a-minute search, which found it on the build before.
  $t0 = Get-Date
  $started = EvStart
  $ok = WaitSmart 'Whole PC' { $s.source -eq 'everything' -and $s.shown -ge 1 } 12
  Check 'A the fence searches within 12 s of Everything starting' ($started -and $ok) "after $([math]::Round(((Get-Date) - $t0).TotalSeconds, 1)) s: $(SmartLine 'Whole PC')"

  Case 'A: a click on Start Everything'
  EvStop | Out-Null
  Pf fence refresh 'Whole PC' | Out-Null
  Check 'A the fence waits for Everything again' (WaitSmart 'Whole PC' { $s.source -eq 'needsEverything' } 10) (SmartLine 'Whole PC')
  Start-Sleep -Milliseconds 500
  $r = (FenceGet 'Whole PC').windowRect
  $row = OcrFind 'Start Everything' $r.x $r.y $r.w $r.h
  Check 'A the row Start Everything is under the sentence' ($null -ne $row) "row=$row words='$(FenceWords 'Whole PC' 'a-before')'"
  [G]::Click($row[0], $row[1]); Start-Sleep -Milliseconds 450
  $words = FenceWords 'Whole PC' 'a-after-click'
  Shot 'a-after-click'
  Check 'A half a second after the click the row is gone (Searching, or the results)' ($words -notmatch 'Start Everything') "words='$words'"
  Check 'A the fence searches once Everything runs' (WaitSmart 'Whole PC' { $s.source -eq 'everything' -and $s.shown -ge 1 } 15) (SmartLine 'Whole PC')
  PfQuit
  Remove-Item -Recurse -Force $HsDir -ErrorAction SilentlyContinue
}
if ($Part -match 'A') { PartA }

function PartB {
  trap { PfTrap $_; continue }
  Case 'B: a smart fence whose folder is gone'
  CloseWindows
  HsFiles
  EvStart | Out-Null
  PfStart -Lang en -Script (DumpScript 500 20)
  Pf settings set theme light | Out-Null
  ClearFences
  Pf fence create --title 'Reports' --smart-folder $HsDir --rect '60,60,440,220' | Out-Null
  Check 'B the fence shows the three files' (WaitSmart 'Reports' { $s.shown -eq 3 } 15) (SmartLine 'Reports')
  Remove-Item -Recurse -Force $HsDir
  Check 'B the files leave with the folder' (WaitSmart 'Reports' { $s.shown -eq 0 } 20) (SmartLine 'Reports')
  [G]::MoveTo(1200, 800); Start-Sleep -Milliseconds 800
  $words = FenceWords 'Reports' 'b-gone'
  $d = DumpOf 'Reports'
  Shot 'b-folder-gone'
  Check 'B the fence says the folder cannot be found, without a count' ($words -match 'cannot be found' -and $d.raw -match 'title_count=None') "words='$words' $([regex]::Match($d.raw, 'title_count=\S+').Value)"
  # The folder read without Everything says the same.
  EvStop | Out-Null
  Pf fence refresh 'Reports' | Out-Null
  WaitSmart 'Reports' { $s.source -eq 'folder' } 10 | Out-Null
  Start-Sleep -Milliseconds 600
  $words = FenceWords 'Reports' 'b-gone-folder-read'
  Check 'B without Everything too' ($words -match 'cannot be found') "words='$words' $(SmartLine 'Reports')"
  HsFiles
  Start-Sleep -Seconds 1
  Pf fence refresh 'Reports' | Out-Null
  Check 'B the folder is back: its files and the count show again' ((WaitSmart 'Reports' { $s.shown -eq 3 } 10) -and (DumpOf 'Reports').raw -match 'title_count=Some\("3"\)') (SmartLine 'Reports')
  PfQuit
  Remove-Item -Recurse -Force $HsDir -ErrorAction SilentlyContinue
}
if ($Part -match 'B') { PartB }

function PartD {
  trap { PfTrap $_; continue }
  Case 'D: without Everything a new file shows soon'
  CloseWindows
  HsFiles
  EvStop | Out-Null
  PfStart -Lang en -Script (DumpScript 500 20)
  ClearFences
  Pf fence create --title 'Reports' --smart-folder $HsDir --rect '60,60,440,220' | Out-Null
  Check 'D the folder is read without Everything' (WaitSmart 'Reports' { $s.source -eq 'folder' -and $s.shown -eq 3 } 15) (SmartLine 'Reports')
  Start-Sleep -Seconds 6
  # The app's own times: from the watcher's first event to the changed results.
  $t = { param($l) [datetime]::Parse(($l -split ' ')[0]).ToUniversalTime() }
  $t0 = (Get-Date).ToUniversalTime().AddMilliseconds(-50)
  Set-Content -LiteralPath "$HsDir\刚保存的文件.txt" -Value 'new'
  $shown = WaitSmart 'Reports' { $s.shown -eq 4 } 10
  Start-Sleep -Milliseconds 300
  $log = @(PfLogAll | Where-Object { $_ -match '^\d{4}-\d\d-\d\dT' -and (& $t $_) -ge $t0 })
  $ev = @($log | Where-Object { $_ -match 'smart fence folders changed' }) | Select-Object -First 1
  $done = @($log | Where-Object { $_ -match 'smart fence results changed.*count=4' }) | Select-Object -First 1
  $ms = if ($ev -and $done) { [int]((& $t $done) - (& $t $ev)).TotalMilliseconds } else { -1 }
  Check 'D the new file shows within 600 ms of the folder change' ($shown -and $ms -ge 0 -and $ms -le 600) "from the watcher's event to the results: $ms ms; $(SmartLine 'Reports')"
  PfQuit
  Remove-Item -Recurse -Force $HsDir -ErrorAction SilentlyContinue
}
if ($Part -match 'D') { PartD }

function HsNotice() { $n = [G]::FindWindowW('PecoFence.Notice', [NullString]::Value); if ($n -ne [IntPtr]::Zero -and [G]::IsWindowVisible($n)) { return $n }; return [IntPtr]::Zero }
function PartC {
  trap { PfTrap $_; continue }
  Case 'C: setup'
  CloseWindows
  $sav = DeskFile '存档 第3关.sav'
  $txt = DeskFile '会议纪要 11月.txt'
  VdTwo '工作' '游戏'
  $ids = @(VdIds)
  PfStart -Lang zh-CN -Script (DumpScript 500 20)
  ClearFences
  Pf fence create --title 游戏存档 --rect '420,80,300,240' | Out-Null
  Pf fence create --title 都显示 --rect '760,80,300,240' | Out-Null
  Pf item move $sav --to 游戏存档 | Out-Null
  Pf item move $txt --to 都显示 | Out-Null
  Start-Sleep -Seconds 2
  # The first-run tips are a card too: closed, so the next card can show.
  for ($i = 0; $i -lt 4 -and (HsNotice) -ne [IntPtr]::Zero; $i++) { CloseNotice | Out-Null; Start-Sleep -Milliseconds 700 }
  Check 'C two desktops, no card on screen' ($ids.Count -eq 2 -and (HsNotice) -eq [IntPtr]::Zero) (VdState)

  Case 'C: sent to the other desktop from the fence menu'
  $sub = VdSubmenu '游戏存档'
  Start-Sleep -Milliseconds 400
  MenuPick '^游戏' $sub | Out-Null
  [G]::MoveTo(600, 700); Start-Sleep -Milliseconds 1500
  $n = HsNotice
  Shot 'c-card'
  Check 'C the fence left this desktop' ((FVis '游戏存档') -eq $false) "vd=$((FenceGet 游戏存档).virtualDesktops)"
  Check 'C a card of PecoFence says where it went' ($n -ne [IntPtr]::Zero) "notice=$n $(if ($n -ne [IntPtr]::Zero) { [W]::Info($n) })"
  if ($n -ne [IntPtr]::Zero) { $nr = Rect $n; [G]::Click($nr.R - 50, $nr.B - 30); Start-Sleep -Milliseconds 1500 }
  Shot 'c-after-undo'
  Check 'C its 撤销 brings the fence back, on every desktop as before' ((FVis '游戏存档') -eq $true -and @((FenceGet 游戏存档).virtualDesktops).Count -eq 0) "visible=$(FVis '游戏存档') vd=$((FenceGet 游戏存档).virtualDesktops)"
  Check 'C the card is gone' ((HsNotice) -eq [IntPtr]::Zero) ''

  Case 'C: Ctrl+Z in a fence takes it back too'
  $sub = VdSubmenu '游戏存档'
  Start-Sleep -Milliseconds 400
  MenuPick '^游戏' $sub | Out-Null
  Start-Sleep -Milliseconds 1200
  Check 'C sent away again' ((FVis '游戏存档') -eq $false) ''
  $p = ItemPt '都显示' '会议纪要 11月'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 400
  Chord 0x11 0x5A; Start-Sleep -Milliseconds 1500
  Check 'C Ctrl+Z: the fence is back and the card closed' ((FVis '游戏存档') -eq $true -and @((FenceGet 游戏存档).virtualDesktops).Count -eq 0 -and (HsNotice) -eq [IntPtr]::Zero) "visible=$(FVis '游戏存档') vd=$((FenceGet 游戏存档).virtualDesktops) notice=$(HsNotice)"

  Case 'C: cleanup'
  ClearPfNotices
  PfQuit
  VdOnlyOne
  Remove-Item -LiteralPath $sav, $txt -Force -ErrorAction SilentlyContinue
}
if ($Part -match 'C') { PartC }

function HsApart($a, $b) { return (($a.x + $a.w -le $b.x) -or ($b.x + $b.w -le $a.x) -or ($a.y + $a.h -le $b.y) -or ($b.y + $b.h -le $a.y)) }
function HsAt($r) { return "$($r.x),$($r.y) $($r.w)x$($r.h)" }
function PartE {
  trap { PfTrap $_; continue }
  Case 'E: setup: two fences at one spot, each on its own desktop'
  CloseWindows
  VdTwo '工作' '游戏'
  PfStart -Lang zh-CN -Script (DumpScript 500 20)
  ClearFences
  Pf fence create --title 工作文件 --rect '420,80,300,240' | Out-Null
  Pf fence set 工作文件 virtualDesktops 1 | Out-Null
  VdRight
  Pf fence create --title 游戏存档 --rect '420,80,300,240' | Out-Null
  Pf fence set 游戏存档 virtualDesktops 2 | Out-Null
  Start-Sleep -Seconds 1
  $a = (FenceGet 工作文件).windowRect; $b = (FenceGet 游戏存档).windowRect
  Check 'E both fences at the same spot, 游戏存档 shows on desktop 2' ($a.x -eq $b.x -and $a.y -eq $b.y -and (FVis 游戏存档) -eq $true -and (FVis 工作文件) -eq $false) "工作文件=$(HsAt $a) 游戏存档=$(HsAt $b) $(VisText @('工作文件', '游戏存档'))"

  Case 'E: desktop 2 is closed'
  VdClose
  Start-Sleep -Seconds 2
  $a = (FenceGet 工作文件).windowRect; $b = (FenceGet 游戏存档).windowRect
  Shot 'e-after-close'
  Check 'E on the desktop that is left both fences show, not one on the other' ((FVis 工作文件) -eq $true -and (FVis 游戏存档) -eq $true -and (HsApart $a $b)) "工作文件=$(HsAt $a) 游戏存档=$(HsAt $b) $(VisText @('工作文件', '游戏存档'))"
  Check 'E the fence that was here kept its spot' ($a.x -eq 420 -and $a.y -eq 80) "工作文件=$(HsAt $a)"

  Case 'E: a fence set to show on a desktop where its spot is taken'
  VdNew
  Start-Sleep -Seconds 1
  Pf fence create --title 新桌面的栅栏 --rect '420,80,300,240' | Out-Null
  Pf fence set 新桌面的栅栏 virtualDesktops current | Out-Null
  Start-Sleep -Milliseconds 500
  $c0 = (FenceGet 新桌面的栅栏).windowRect
  Pf fence set 工作文件 virtualDesktops all | Out-Null
  Start-Sleep -Seconds 1
  $a = (FenceGet 工作文件).windowRect; $c = (FenceGet 新桌面的栅栏).windowRect
  Shot 'e-after-set-all'
  Check 'E the fence brought here stands beside the one at its spot' ((FVis 工作文件) -eq $true -and (FVis 新桌面的栅栏) -eq $true -and (HsApart $a $c) -and $c.x -eq $c0.x -and $c.y -eq $c0.y) "工作文件=$(HsAt $a) 新桌面的栅栏=$(HsAt $c) (was $(HsAt $c0))"

  Case 'E: cleanup'
  ClearPfNotices
  PfQuit
  VdOnlyOne
}
if ($Part -match 'E') { PartE }

# A stand-in for an Everything that is running but has not read its index: a window of
# Everything's class that names itself 1.4, says "not loaded" (EVERYTHING_IPC_IS_DB_LOADED) and
# answers every search with an empty list, as Everything does until its index is read.
$HsFake = @'
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class FakeEv {
  delegate IntPtr WndProc(IntPtr h, uint m, IntPtr w, IntPtr l);
  [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)] struct WNDCLASS { public uint style; public WndProc proc; public int a, b; public IntPtr inst, icon, cursor, brush; public string menu, name; }
  [StructLayout(LayoutKind.Sequential)] struct MSG { public IntPtr h; public uint m; public IntPtr w, l; public uint t; public int x, y; }
  [StructLayout(LayoutKind.Sequential)] struct CDS { public IntPtr id; public int size; public IntPtr data; }
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern ushort RegisterClassW(ref WNDCLASS c);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr CreateWindowExW(uint ex, string cls, string name, uint style, int x, int y, int w, int h, IntPtr parent, IntPtr menu, IntPtr inst, IntPtr p);
  [DllImport("user32.dll")] static extern IntPtr DefWindowProcW(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] static extern int GetMessageW(out MSG m, IntPtr h, uint a, uint b);
  [DllImport("user32.dll")] static extern IntPtr DispatchMessageW(ref MSG m);
  [DllImport("user32.dll")] static extern bool PostMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] static extern IntPtr SendMessageTimeoutW(IntPtr h, uint m, IntPtr w, ref CDS l, uint f, uint ms, out IntPtr r);
  static WndProc keep;
  static IntPtr Proc(IntPtr h, uint m, IntPtr w, IntPtr l) {
    if (m == 0x400) { int q = (int)w; return (IntPtr)(q == 0 ? 1 : q == 1 ? 4 : 0); }
    if (m == 0x4A) {
      CDS c = (CDS)Marshal.PtrToStructure(l, typeof(CDS));
      if ((int)c.id == 18 && c.size >= 8) { PostMessageW(h, 0x8001, (IntPtr)Marshal.ReadInt32(c.data, 0), (IntPtr)Marshal.ReadInt32(c.data, 4)); return (IntPtr)1; }
      return IntPtr.Zero;
    }
    if (m == 0x8001) {
      // EVERYTHING_IPC_LIST2 with no items: totitems, numitems, offset, request_flags (name, path), sort_type.
      IntPtr buf = Marshal.AllocHGlobal(20);
      for (int i = 0; i < 20; i += 4) Marshal.WriteInt32(buf, i, i == 12 ? 3 : i == 16 ? 14 : 0);
      CDS r = new CDS(); r.id = l; r.size = 20; r.data = buf; IntPtr res;
      SendMessageTimeoutW(w, 0x4A, h, ref r, 2, 2000, out res);
      Marshal.FreeHGlobal(buf);
      return IntPtr.Zero;
    }
    return DefWindowProcW(h, m, w, l);
  }
  public static void Run() {
    keep = Proc;
    WNDCLASS c = new WNDCLASS(); c.proc = keep; c.name = "EVERYTHING_TASKBAR_NOTIFICATION";
    RegisterClassW(ref c);
    CreateWindowExW(0, c.name, "", 0, 0, 0, 0, 0, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero);
    MSG m; while (GetMessageW(out m, IntPtr.Zero, 0, 0) > 0) DispatchMessageW(ref m);
  }
}
"@
[FakeEv]::Run()
'@
function PartF {
  trap { PfTrap $_; continue }
  Case 'F: setup: a whole-computer fence that finds a file'
  CloseWindows
  HsFiles
  EvStart | Out-Null
  PfStart -Lang en -Script (DumpScript 500 20)
  ClearFences
  Pf fence create --title 'Whole PC' --everything '会议纪要' --rect '60,60,440,220' | Out-Null
  Check 'F the fence shows the file' (WaitSmart 'Whole PC' { $s.source -eq 'everything' -and $s.shown -ge 1 } 15) (SmartLine 'Whole PC')
  PfQuit
  EvStop | Out-Null

  Case 'F: PecoFence starts while Everything has not read its index'
  $file = "$PfRun\fake-everything.ps1"
  $HsFake | Set-Content -LiteralPath $file -Encoding UTF8
  $fake = Start-Process powershell -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', $file -WindowStyle Hidden -PassThru
  for ($i = 0; $i -lt 50 -and -not (EvRunning); $i++) { Start-Sleep -Milliseconds 200 }
  Check 'F the stand-in for a loading Everything is up' (EvRunning) "pid=$($fake.Id)"
  PfStart -Lang '' -Script (DumpScript 500 20)
  [G]::MoveTo(1200, 800); Start-Sleep -Milliseconds 500
  $words = FenceWords 'Whole PC' 'f-loading'
  Shot 'f-loading'
  Check 'F the fence does not say that no file matches' ($words -notmatch 'No files match' -and (DumpOf 'Whole PC').raw -match 'title_count=None') "words='$words' $(SmartLine 'Whole PC')"
  # The real Everything takes over (the stand-in goes, as if the index were read now).
  Stop-Process -Id $fake.Id -Force -ErrorAction SilentlyContinue
  $t0 = Get-Date
  $started = EvStart
  $ok = WaitSmart 'Whole PC' { $s.source -eq 'everything' -and $s.shown -ge 1 } 12
  Check 'F the file shows within 12 s of the index being there' ($started -and $ok) "after $([math]::Round(((Get-Date) - $t0).TotalSeconds, 1)) s: $(SmartLine 'Whole PC')"
  PfQuit
  Get-Process powershell -ErrorAction SilentlyContinue | Where-Object { $_.Id -eq $fake.Id } | Stop-Process -Force -ErrorAction SilentlyContinue
  Remove-Item -Recurse -Force $HsDir -ErrorAction SilentlyContinue
}
if ($Part -match 'F') { PartF }

function PartG {
  trap { PfTrap $_; continue }
  Case 'G: the fence picker of Settings'
  Check 'G a debug build (the page is read over CDP)' $PfDebugBuild ''
  CloseWindows
  HsFiles
  VdTwo '工作' '游戏'
  $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
  PfStart -Lang zh-CN -Script (DumpScript 500 20)
  ClearFences
  Pf fence create --title 游戏存档 --rect '420,80,300,240' | Out-Null
  Pf fence set 游戏存档 virtualDesktops 2 | Out-Null
  Pf fence create --title 最近文件 --smart-folder $HsDir --rect '760,80,300,240' | Out-Null
  Pf fence create --title 普通栅栏 --rect '420,360,300,200' | Out-Null
  PfSettings | Out-Null
  Js "(() => { document.querySelector('[data-page=fences]')?.click(); return true; })()" | Out-Null
  Start-Sleep -Milliseconds 800
  $opts = Js "[...document.getElementById('fenceSel').options].map(o => o.textContent).join(' | ')"
  # The open list, for the picture.
  PageClick '#fenceSel' | Out-Null; Start-Sleep -Milliseconds 900
  Shot 'g-picker'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  Check 'G the picker says which fence shows on one desktop only' ($opts -match '游戏存档（只在“游戏”上显示）') "$opts"
  Check 'G and which one is a smart fence' ($opts -match '最近文件（智能栅栏）') "$opts"
  Check 'G a plain fence has its name only' ($opts -match '(^| \| )普通栅栏( \| |$)') "$opts"
  $h = [G]::FindWindowW('PecoFence.Settings', [NullString]::Value)
  if ($h -ne [IntPtr]::Zero) { [W]::PostMessageW($h, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; Start-Sleep -Seconds 1 }
  ClearPfNotices
  PfQuit
  VdOnlyOne
  Remove-Item -Recurse -Force $HsDir -ErrorAction SilentlyContinue
}
if ($Part -match 'G') { PartG }

function PartH {
  trap { PfTrap $_; continue }
  Case 'H: the results of a smart fence all go'
  CloseWindows
  HsFiles
  EvStart | Out-Null
  # An RDP session has Windows' animations off: on for this part.
  $anim = [Anim]::Get(); [Anim]::Set($true)
  PfStart -Lang en -Script (DumpScript 500 20)
  ClearFences
  Pf fence create --title 'Whole PC' --everything '会议纪要' --rect '60,60,440,220' | Out-Null
  Check 'H the fence shows the file' (WaitSmart 'Whole PC' { $s.source -eq 'everything' -and $s.shown -ge 1 } 15) (SmartLine 'Whole PC')
  [G]::MoveTo(1200, 800); Start-Sleep -Seconds 1
  EvStop | Out-Null
  $t = { param($l) [datetime]::Parse(($l -split ' ')[0]).ToUniversalTime() }
  $t0 = (Get-Date).ToUniversalTime().AddMilliseconds(-50)
  Pf fence refresh 'Whole PC' | Out-Null
  $gone = WaitSmart 'Whole PC' { $s.source -eq 'needsEverything' -and $s.shown -eq 0 } 10
  Start-Sleep -Milliseconds 600
  $fade = @(PfLogAll | Where-Object { $_ -match 'refill cross-fades|replace_items animate=true' -and (& $t $_) -ge $t0 }) | Select-Object -First 1
  Check 'H the icons fade out under the sentence (a cross-fade, no cut)' ($gone -and $fade) "results gone=$gone; log: $(if ($fade) { $fade.Substring([math]::Min(28, $fade.Length)) } else { 'no cross-fade' })"
  PfQuit
  [Anim]::Set($anim)
  Remove-Item -Recurse -Force $HsDir -ErrorAction SilentlyContinue
}
if ($Part -match 'H') { PartH }
