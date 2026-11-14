# Owner round 2026-11-12, third batch, third pass on icons.
#   run.sh --var Part=H tests/owner3-ux3-icons3.ps1    the one-line messages of an empty fence in narrow fences, four languages (pictures)
#   run.sh --var Part=K ...                            three clock fences asked for within a second from the command line (the desktop takes one)
#   run.sh --var Part=E ...                            exploration: double-click, Enter, five files arriving, paste, Ctrl / Shift clicks (recordings)
#   run.sh --var Part=F ...                            nine files arriving in a fence that is full and in one of automatic height (the text of the empty fence stays in place), a paste of nine (recordings)
#   run.sh --var Part=X ...                            Explorer's own desktop without PecoFence: a double-click on an icon (recording, the busy pointer)
# Real mouse and keys, a new configuration. Every part ends with the desktop cleaned.
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'H' }
. C:\pf\kit\rec.ps1
function U3Wait([scriptblock]$ok, $sec = 8) { for ($k = 0; $k -lt $sec * 5; $k++) { if (& $ok) { return $true }; Start-Sleep -Milliseconds 200 }; return $false }
function U3Fences() { return @(@(PfJson fence list) | Where-Object { $_ -and $_.id -and $_.kind -ne 'inbox' }) }
$U3Dir = 'C:\pf\run\u3'
# The pointer Windows shows while a program starts (the arrow with the ring, IDC_APPSTARTING).
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class U3Cur {
  [StructLayout(LayoutKind.Sequential)] struct CI { public int cb; public int flags; public IntPtr h; public int x; public int y; }
  [DllImport("user32.dll")] static extern bool GetCursorInfo(ref CI c);
  [DllImport("user32.dll")] static extern IntPtr LoadCursorW(IntPtr inst, IntPtr id);
  public static bool Busy() { CI c = new CI(); c.cb = Marshal.SizeOf(typeof(CI)); GetCursorInfo(ref c); return c.h == LoadCursorW(IntPtr.Zero, (IntPtr)32650); }
  // Looks at the pointer every 15 ms for `ms`: "first,last,busy/all" (ms after the call; -1 = never).
  public static string Watch(int ms) {
    var sw = System.Diagnostics.Stopwatch.StartNew(); long first = -1, last = -1; int n = 0, busy = 0;
    while (sw.ElapsedMilliseconds < ms) { n++; if (Busy()) { busy++; if (first < 0) first = sw.ElapsedMilliseconds; last = sw.ElapsedMilliseconds; } System.Threading.Thread.Sleep(15); }
    return first + "," + last + "," + busy + "/" + n;
  }
}
'@

if ($Part -eq 'H') {
  # 「将项目拖到此处」, 「此文件夹为空。」 and 「找不到此文件夹。」 at the least width (64), 150 and 220 px.
  $widths = @(64, 150, 220)
  foreach ($lang in @('zh-CN', 'ja', 'de', 'en')) {
    Case "The messages of an empty fence in narrow fences, $lang"
    # One folder per fence: a folder is shown by one fence at most.
    foreach ($w in $widths) { New-Item -ItemType Directory -Force "$U3Dir\empty$w" | Out-Null; New-Item -ItemType Directory -Force "$U3Dir\gone$w" | Out-Null }
    PfStart -Lang $lang
    Start-Sleep -Seconds 4
    CloseNotice | Out-Null
    ClearFences | Out-Null
    $x = 40
    foreach ($w in $widths) {
      Pf fence create --title "a$w" --rect "$x,60,$w,150" | Out-Null
      Pf fence create --title "b$w" --portal "$U3Dir\empty$w" --rect "$x,230,$w,150" | Out-Null
      Pf fence create --title "c$w" --portal "$U3Dir\gone$w" --rect "$x,400,$w,150" | Out-Null
      $x += $w + 30
    }
    foreach ($w in $widths) { Remove-Item -LiteralPath "$U3Dir\gone$w" -Recurse -Force }
    # A folder fence looks at its folder again when the pointer comes onto it.
    $x = 40
    foreach ($w in $widths) { [G]::MoveTo($x + [int]($w / 2), 480); Start-Sleep -Milliseconds 700; $x += $w + 30 }
    ParkMouse 1100 700
    Start-Sleep -Milliseconds 1500
    $n = @(U3Fences).Count
    Crop "hint-$lang" 30 50 560 520 | Out-Null
    Check "nine fences stand, $lang" ($n -eq 9) "fences=$n"
    PfQuit
  }
  Remove-Item -LiteralPath $U3Dir -Recurse -Force -ErrorAction SilentlyContinue
}

if ($Part -eq 'K') {
  Case 'Three clocks asked for within a second: one clock, two commands refused with the reason'
  PfStart -Lang 'zh-CN'
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  ClearFences | Out-Null
  $cli = "$PfApp\pecofence-cli.exe"
  $t0 = Get-Date
  $ps = @(1..3 | ForEach-Object { Start-Process -FilePath $cli -ArgumentList 'fence', 'create', '--tool', 'clock' -WindowStyle Hidden -PassThru })
  $ps | ForEach-Object { $_.WaitForExit(15000) | Out-Null }
  $ms = [int]((Get-Date) - $t0).TotalMilliseconds
  Start-Sleep -Milliseconds 1500
  $f = @(U3Fences)
  $rects = @($f | ForEach-Object { $g = FenceGet $_.id; "$($g.windowRect.x),$($g.windowRect.y),$($g.windowRect.w),$($g.windowRect.h)" })
  Shot 'k-three-clocks'
  # The desktop takes one clock: the first command makes it, the others are refused and say why.
  $codes = (($ps | ForEach-Object { $_.ExitCode }) | Sort-Object) -join ','
  Check 'one clock stands, the two other commands end with an error' ($f.Count -eq 1 -and $codes -eq '0,1,1') "fences=$($f.Count) exit codes=$codes in $ms ms; rect $($rects -join ' | ')"
  $again = Pf fence create --tool clock
  Check 'a refused command says that the desktop has a clock already' ($again -match 'has a clock already') "$($again.Trim())"
  ClearFences | Out-Null
}

if ($Part -eq 'E') {
  $names = @('季度预算.fpx', '图.fpx', '一个名字特别长的客户往来文件最终版.fpx', 'b.fpx', '说明.txt')
  function U3Clean() {
    Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue | Where-Object { $_.Name -match '\.fpx$|^说明|^来的|Copy|副本' } | Remove-Item -Force -Recurse -ErrorAction SilentlyContinue
    Get-Process OpenWith, notepad -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  }
  Case 'Exploration: open, arrive, paste, select'
  CloseWindows; U3Clean
  foreach ($n in $names) { DeskFile $n | Out-Null }
  $animWas = [Anim]::Get(); [Anim]::Set($true)
  PfStart -Lang 'zh-CN'
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  ClearFences | Out-Null
  Pf fence create --title '甲' --rect '60,80,340,330' | Out-Null
  Pf fence create --title '乙' --rect '440,80,340,330' | Out-Null
  Pf item move --glob '*.fpx' --to '甲' | Out-Null
  Pf item move --glob '说明.txt' --to '甲' | Out-Null
  Start-Sleep -Milliseconds 1500
  $order = @(@(PfJson item list --fence '甲') | ForEach-Object { $_.fileName })
  Log "order in the fence: $($order -join ' | ')"
  $iTxt = [array]::IndexOf($order, '说明.txt')
  $p = CellPt '甲' $iTxt
  ParkMouse 1100 700

  # 1. double-click on the text file: the frames from the second press on.
  RecStart 'e1-dblclick' -X 60 -Y 80 -W 340 -H 330 -Fps 30
  Start-Sleep -Milliseconds 300
  RecMark '双击'
  DoubleClick $p[0] $p[1]
  $cur = [U3Cur]::Watch(2500)
  RecStop
  Log "the busy pointer after a double-click in a fence (first ms, last ms, samples): $cur"
  Check 'a double-click in a fence shows the busy pointer while the program starts' ($cur -notmatch '^-1') "first,last,busy/all = $cur"
  Shot 'e1-after-dblclick'
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Milliseconds 600

  # 1b. Enter on the selected icon.
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
  RecStart 'e1-enter' -X 60 -Y 80 -W 340 -H 330 -Fps 30
  Start-Sleep -Milliseconds 300
  RecMark 'Enter'
  [G]::Press(0x0D)
  $cur = [U3Cur]::Watch(2500)
  RecStop
  Log "the busy pointer after Enter in a fence: $cur"
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Milliseconds 600

  # 4. Ctrl and Shift clicks, then a Shift click after a keyboard move.
  RecStart 'e4-select' -X 60 -Y 80 -W 340 -H 330 -Fps 15
  $c0 = CellPt '甲' 0; $c1 = CellPt '甲' 1; $c2 = CellPt '甲' 2; $c3 = CellPt '甲' 3; $c4 = CellPt '甲' 4
  [G]::Click($c0[0], $c0[1]); Start-Sleep -Milliseconds 500
  RecMark 'Ctrl 点第三个'
  [G]::KeyDown(0x11); [G]::Click($c2[0], $c2[1]); [G]::KeyUp(0x11); Start-Sleep -Milliseconds 600
  RecMark 'Shift 点第五个'
  [G]::KeyDown(0x10); [G]::Click($c4[0], $c4[1]); [G]::KeyUp(0x10); Start-Sleep -Milliseconds 600
  Shot 'e4-after-shift'
  RecMark '点第一个, 右箭头两次, Shift 点第五个'
  [G]::Click($c0[0], $c0[1]); Start-Sleep -Milliseconds 400
  [G]::Press(0x27); Start-Sleep -Milliseconds 300; [G]::Press(0x27); Start-Sleep -Milliseconds 400
  [G]::KeyDown(0x10); [G]::Click($c4[0], $c4[1]); [G]::KeyUp(0x10); Start-Sleep -Milliseconds 600
  Shot 'e4-after-keys-shift'
  $sel = @(@(PfJson item list --fence '甲') | Where-Object { $_.selected } | ForEach-Object { $_.fileName })
  Log "selected after arrows and a Shift click: $($sel -join ' | ')"
  RecStop

  # 3. copy two icons, paste them into the other fence.
  [G]::Click($c0[0], $c0[1]); Start-Sleep -Milliseconds 300
  [G]::KeyDown(0x11); [G]::Click($c1[0], $c1[1]); [G]::KeyUp(0x11); Start-Sleep -Milliseconds 300
  Chord 0x11 0x43; Start-Sleep -Milliseconds 400
  [G]::Click(700, 330); Start-Sleep -Milliseconds 500
  RecStart 'e3-paste' -X 60 -Y 80 -W 720 -H 330 -Fps 30
  Start-Sleep -Milliseconds 300
  RecMark 'Ctrl+V 到另一个栅栏'
  Chord 0x11 0x56
  Start-Sleep -Milliseconds 3000
  RecStop
  Shot 'e3-after-paste'
  Log ("after the paste: " + ((@(PfJson item list) | ForEach-Object { "$($_.fileName)@$($_.fenceTitle)" }) -join ' | '))

  # 2. five files arrive at once in a fence that takes new files by a rule.
  Pf rule add --name fpy --ext fpy --to '乙' | Out-Null
  Start-Sleep -Milliseconds 500
  RecStart 'e2-arrive' -X 440 -Y 80 -W 340 -H 330 -Fps 30
  Start-Sleep -Milliseconds 300
  RecMark '五个文件同时出现'
  foreach ($k in 1..5) { DeskFile "来的$k.fpy" | Out-Null }
  Start-Sleep -Milliseconds 3500
  RecStop
  Shot 'e2-after-arrive'
  Get-ChildItem -LiteralPath $PfDesk -Filter '*.fpy' -ErrorAction SilentlyContinue | Remove-Item -Force
  [Anim]::Set($animWas); U3Clean
  Check 'exploration ran' $true "order $($order -join ' | ')"
}

if ($Part -eq 'X') {
  # Explorer's own desktop, no PecoFence: what a double-click on an icon shows.
  . C:\pf\kit\deskicons.ps1
  Case 'Explorer own desktop: a double-click on an icon'
  CloseWindows
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  DeskFile '说明.txt' | Out-Null
  Start-Sleep -Seconds 3
  $o = [DeskIcons]::Origin() -split ',' | ForEach-Object { [int]$_ }
  $line = @([DeskIcons]::List() | Where-Object { $_ -match '说明' }) | Select-Object -First 1
  Log "desktop icon: $line; origin $($o -join ',')"
  $f = $line -split '\|' | ForEach-Object { $_.Trim() }
  $px = $o[0] + [int]$f[1] + 37; $py = $o[1] + [int]$f[2] + 28
  ParkMouse 1100 700
  RecStart 'x-dblclick' -X ($px - 70) -Y ($py - 50) -W 180 -H 150 -Fps 30
  Start-Sleep -Milliseconds 300
  RecMark '双击'
  DoubleClick $px $py
  $cur = [U3Cur]::Watch(2500)
  RecStop
  $np = @(Get-Process notepad -ErrorAction SilentlyContinue).Count
  Check 'Explorer: the double-click opened the file' ($np -ge 1) "notepads=$np at $px,$py"
  Check 'Explorer: the busy pointer shows while the program starts' ($cur -notmatch '^-1') "first,last,busy/all = $cur"
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  Remove-Item -LiteralPath (Join-Path $PfDesk '说明.txt') -Force -ErrorAction SilentlyContinue
}

if ($Part -eq 'F') {
  function U3CleanF() { Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue | Where-Object { $_.Name -match '\.fpy$|\.fpz$' } | Remove-Item -Force -ErrorAction SilentlyContinue }
  Case 'Nine files arrive at once in a full fence and in one of automatic height, a paste of nine'
  CloseWindows; U3CleanF
  $animWas = [Anim]::Get(); [Anim]::Set($true)
  PfStart -Lang 'zh-CN'
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  ClearFences | Out-Null
  Pf fence create --title '丙' --rect '60,80,340,150' | Out-Null
  Pf fence create --title '丁' --rect '440,80,340,150' | Out-Null
  Log ("auto height: " + (Pf fence set '丁' autoHeight true))
  Pf rule add --name fpy --ext fpy --to '丙' | Out-Null
  Pf rule add --name fpz --ext fpz --to '丁' | Out-Null
  Start-Sleep -Milliseconds 800
  ParkMouse 1100 700
  RecStart 'f1-arrive' -X 60 -Y 80 -W 720 -H 420 -Fps 30
  Start-Sleep -Milliseconds 300
  RecMark '两个栅栏各来九个文件'
  foreach ($k in 1..9) { DeskFile "来的文件$k.fpy" | Out-Null; DeskFile "另一批$k.fpz" | Out-Null }
  Start-Sleep -Milliseconds 3500
  RecStop
  Shot 'f1-after-arrive'
  # The fence of automatic height grows under the nine icons: the text of the empty fence
  # fades out where it stood (it jumped down 46 px in one frame, across the first row's names).
  $kept = @(PfLogAll | Where-Object { $_ -match 'the text of an empty fence stays where it stood' })
  Check 'files arrive in an empty fence of automatic height: its text stays where it stood while it fades out' ($kept.Count -ge 1) "log lines: $($kept.Count); $(($kept | Select-Object -First 1) -replace '^.*DEBUG ', '')"
  # A paste of nine into the fence of automatic height.
  [G]::Click(100, 150); Start-Sleep -Milliseconds 300
  Chord 0x11 0x41; Start-Sleep -Milliseconds 300
  Chord 0x11 0x43; Start-Sleep -Milliseconds 400
  $g = FenceGet '丁'
  [G]::Click($g.windowRect.x + 300, $g.windowRect.y + $g.windowRect.h - 12); Start-Sleep -Milliseconds 500
  RecStart 'f2-paste' -X 60 -Y 80 -W 720 -H 620 -Fps 30
  Start-Sleep -Milliseconds 300
  RecMark 'Ctrl+V 九个文件'
  Chord 0x11 0x56
  Start-Sleep -Milliseconds 4000
  RecStop
  Shot 'f2-after-paste'
  Log ("after the paste: " + ((@(PfJson item list --fence '丁') | ForEach-Object { $_.fileName }) -join ' | '))
  [Anim]::Set($animWas); U3CleanF
  Check 'exploration ran' $true ''
}
