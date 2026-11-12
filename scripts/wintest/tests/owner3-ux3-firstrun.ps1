# Owner round 2026-11-12, third batch, package ux3-firstrun: the first ten minutes of a new user
# and the fences by keyboard alone. Parts (run.sh --var Part=...):
#   W  the walk: a first start on a desktop with forty files and shortcuts, recorded (the start at
#      15 frames per second, the rest at 5), then what a new user tries. --var Lang=zh-CN sets
#      the language as soon as the app answers (the boxes have no Chinese display language).
#   K  the fences by keyboard alone: every step logs who has the keyboard and takes a screenshot.
#   S  a first start while the disk keeps writes waiting (a 6 GB copy, as big-copy-save.ps1), recorded.
#   C  the checks of this package's fixes (each fails on the round's base and passes after).
# W, K and S are for looking (frames, screenshots, the job log); only C is a suite line.
# Runs on a 1600x900 box at 100 %.
if (-not $Part) { $Part = 'C' }
if (-not $Lang) { $Lang = 'en' }
trap { PfTrap $_; continue }
. C:\pf\kit\rec.ps1

$FrFiles = @('2026年第三季度销售报告.docx', '预算表.xlsx', '产品发布会演示文稿最终版.pptx', '合同扫描件.pdf', '发票 2026-10.pdf',
  '简历.pdf', '会议记录.txt', '待办.md', '客户名单.csv', 'README.txt', '全家福.jpg', '截图 2026-11-01 093015.png', '设计稿 v3.png',
  'logo.svg', '壁纸.bmp', '生日视频.mp4', '录音.mp3', 'Podcast episode 12.mp3', '项目备份.zip', 'photos.7z', '安装包.rar',
  'setup.exe', 'script.py', 'index.html', 'data.json', '字体.ttf')
$FrDirs = @('工作', '照片', 'Projects', '下载的资料')
$FrLinks = @{ '记事本' = 'notepad.exe'; '计算器' = 'calc.exe'; '画图' = 'mspaint.exe'; '命令提示符' = 'cmd.exe'; '任务管理器' = 'taskmgr.exe'
  'Windows PowerShell' = 'WindowsPowerShell\v1.0\powershell.exe'; '资源管理器' = '..\explorer.exe'; '注册表编辑器' = '..\regedit.exe' }
$FrUrls = @('公司内网.url', 'GitHub.url')
function FrClean() {
  foreach ($n in @($FrFiles) + @($FrDirs) + @($FrUrls) + @($FrLinks.Keys | ForEach-Object { "$_.lnk" }) + @('fr-tmp.lnk')) {
    $p = Join-Path $PfDesk $n
    if (Test-Path -LiteralPath $p) { Remove-Item -LiteralPath $p -Recurse -Force -ErrorAction SilentlyContinue }
  }
}
# Forty items: 26 files, 4 folders, 8 shortcuts, 2 web links. (WScript.Shell cannot save a
# shortcut with a Chinese name: it is saved under a plain name and renamed.)
function FrSeed() {
  FrClean
  foreach ($n in $FrFiles) { DeskFile $n | Out-Null }
  foreach ($n in $FrDirs) { New-Item -ItemType Directory -Force (Join-Path $PfDesk $n) | Out-Null }
  foreach ($k in $FrLinks.Keys) {
    $tmp = Join-Path $PfDesk 'fr-tmp.lnk'
    $s = (New-Object -ComObject WScript.Shell).CreateShortcut($tmp); $s.TargetPath = "$env:WINDIR\System32\$($FrLinks[$k])"; $s.Save()
    Rename-Item -LiteralPath $tmp -NewName "$k.lnk"
  }
  foreach ($n in $FrUrls) { Set-Content -LiteralPath (Join-Path $PfDesk $n) -Value "[InternetShortcut]`r`nURL=https://example.com/" -Encoding ASCII }
}
# A first start as a user makes it: fresh data, no test script (the tips show).
function FrStart([switch]$Keep) {
  if (PfAlive) { PfQuit }
  if (-not $Keep) { Remove-Item -Recurse -Force "$PfApp\config", "$PfApp\notices.json" -ErrorAction SilentlyContinue }
  if (-not (Test-Path "$PfApp\pecofence.exe")) { New-Item -ItemType Directory -Force $PfApp | Out-Null; Copy-Item "$PfBin\*" $PfApp -Force }
  $env:PECOFENCE_INSTANCE = $PfInst
  $env:RUST_LOG = 'pecofence=debug'
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $PfApp
  $h = WaitWin 'PecoFence.Control' 20 -Hidden
  if ($Lang -ne 'en' -and -not $Keep) { & "$PfApp\pecofence-cli.exe" settings set language $Lang | Out-Null }
  return $h
}
function FrFences() { return @(@(PfJson fence list) | Where-Object { $_ -and $_.title -and $_.windowRect -and $_.kind -ne 'inbox' }) }
function FrNotice() {
  $n = [G]::FindWindowW('PecoFence.Notice', [NullString]::Value)
  if ($n -eq [IntPtr]::Zero -or -not [G]::IsWindowVisible($n)) { return 'none' }
  $r = Rect $n; return "$($r.L),$($r.T),$($r.R),$($r.B)"
}
function FrState($tag) {
  Log "== ${tag}: fg=$(Fg) notice=$(FrNotice) icons=$([W]::IconsVisible()) fences=$(VisFences)"
  Shot "w-$tag"
}

Case 'setup'
CloseWindows
Get-Process OpenWith -ErrorAction SilentlyContinue | Stop-Process -Force
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
[G]::SystemParametersInfoW(0x25, 1, [NullString]::Value, 3) | Out-Null
FrSeed
. C:\pf\kit\deskicons.ps1
# Explorer's desktop shows the new files (it missed some of forty made within a second).
DeskClick | Out-Null; [G]::Press(0x74); Start-Sleep -Seconds 4
Log "Explorer's desktop shows $(@([DeskIcons]::List()).Count) icons"
Check 'forty items on the desktop' (@(Get-ChildItem $PfDesk).Count -ge 40) "count=$(@(Get-ChildItem $PfDesk).Count)"

function FrWalk() {
  trap { PfTrap $_; continue }
  Case "walk ($Lang): the first start"
  ParkMouse 800 850
  FrState 'before'
  RecStart "start-$Lang" -Fps 15
  RecMark 'start PecoFence'
  $h = FrStart
  $n = WaitWin 'PecoFence.Notice' 30
  Start-Sleep -Seconds 7
  RecStop
  FrState 'started'
  foreach ($f in FrFences) { Log "fence '$($f.title)' $($f.windowRect.x),$($f.windowRect.y),$($f.windowRect.w)x$($f.windowRect.h) items=$($f.itemCount) rows=$($f.fit.rows) overflow=$($f.fit.overflow)" }
  Check 'the first tip card shows' ($n -ne [IntPtr]::Zero) "notice=$(FrNotice)"

  RecStart "walk-$Lang" -Fps 5
  # The tips: the next ones, then the last one's button is left alone.
  Case "walk ($Lang): the tips"
  foreach ($i in 1..3) {
    $nw = [G]::FindWindowW('PecoFence.Notice', [NullString]::Value)
    if ($nw -eq [IntPtr]::Zero -or -not [G]::IsWindowVisible($nw)) { break }
    $r = Rect $nw
    RecMark "tip $i"
    FrState "tip-$i"
    # The action button: bottom right of the card.
    [G]::MoveTo($r.R - 60, $r.B - 30); Start-Sleep -Milliseconds 500
    [G]::Click($r.R - 60, $r.B - 30); Start-Sleep -Milliseconds 1200
  }
  FrState 'tip-last'
  CloseNotice | Out-Null
  Start-Sleep -Milliseconds 600

  Case "walk ($Lang): drag an icon out and back"
  $fs = FrFences
  $f0 = $fs | Sort-Object { - $_.itemCount } | Select-Object -First 1
  $it = @(PfJson item list --fence $f0.id) | Where-Object { $_ -and $_.name } | Select-Object -First 1
  $p = ItemPt $f0.id $it.name
  $free = FreeSpot 500 500
  Log "drag '$($it.name)' of '$($f0.title)' from $($p -join ',') to $($free -join ',')"
  RecMark 'drag an icon out to the desktop'
  DragTo $p[0] $p[1] $free[0] $free[1] 20
  Start-Sleep -Seconds 2
  FrState 'icon-out'
  Log "item now in: $(ItemFence $it.name)"
  RecMark 'drag it back'
  $back = @(($f0.windowRect.x + 60), ($f0.windowRect.y + 90))
  $at = ItemPt (ItemFence $it.name) $it.name
  if ($at) { DragTo $at[0] $at[1] $back[0] $back[1] 20 } else { Log 'no point for the icon on the desktop' }
  Start-Sleep -Seconds 2
  FrState 'icon-back'
  Log "item now in: $(ItemFence $it.name)"

  Case "walk ($Lang): drag a fence, right-clicks, double-click"
  $f1 = $fs | Sort-Object { $_.itemCount } | Select-Object -First 1
  RecMark 'drag a fence by its title'
  DragTo ($f1.windowRect.x + 80) ($f1.windowRect.y + 14) ($f1.windowRect.x - 260) ($f1.windowRect.y + 300) 20
  Start-Sleep -Seconds 1
  FrState 'fence-dragged'
  $f1 = FenceGet $f1.id
  RecMark 'right-click the fence title'
  [G]::RClick($f1.windowRect.x + 80, $f1.windowRect.y + 14); $m = MenuWin 5
  Log "fence menu: $(MenuTexts $m)"
  FrState 'fence-menu'
  CloseMenus
  RecMark 'right-click an icon'
  $m = RClickItem $f0.id $it.name
  Log "item menu: $(MenuTexts $m)"
  FrState 'item-menu'
  CloseMenus
  RecMark 'right-click the desktop'
  $d = FreeSpot 700 600
  [G]::RClick($d[0], $d[1]); $m = MenuWin 5
  Log "desktop menu: $(MenuTexts $m)"
  FrState 'desk-menu'
  CloseMenus
  RecMark 'double-click the desktop'
  DoubleClick $d[0] $d[1]; Start-Sleep -Seconds 2
  FrState 'desk-dblclick'
  RecMark 'double-click again'
  DoubleClick $d[0] $d[1]; Start-Sleep -Seconds 2
  FrState 'desk-dblclick-2'

  Case "walk ($Lang): the tray icon and Settings"
  $tr = TrayIconRect
  Log "tray icon: $tr"
  RecMark 'click the tray icon'
  [G]::MoveTo([int]($tr.X + $tr.Width / 2), [int]($tr.Y + $tr.Height / 2)); Start-Sleep -Milliseconds 1500
  FrState 'tray-hover'
  [G]::Click([int]($tr.X + $tr.Width / 2), [int]($tr.Y + $tr.Height / 2)); Start-Sleep -Seconds 2
  FrState 'tray-click'
  $sw = [G]::FindWindowW('PecoFence.Settings', [NullString]::Value)
  if ((Fg) -match 'PecoFence.Settings') { Chord 0x12 0x73; Start-Sleep -Seconds 1 }
  Log "Settings closed: fg=$(Fg)"
  RecMark 'right-click the tray icon'
  $m = TrayMenu
  Log "tray menu: $(MenuTexts $m)"
  FrState 'tray-menu'
  CloseMenus

  Case "walk ($Lang): a rectangle on the desktop"
  $fr = FreeRect 360 260 300 300
  Log "free rect: $($fr -join ',')"
  RecMark 'drag a rectangle on the empty desktop'
  $ow = $null
  if ($fr) {
    DragTo ($fr[0] + 10) ($fr[1] + 10) ($fr[0] + 340) ($fr[1] + 240) 12
    $ow = @([W]::Tops('PecoFence.MarqueeOffer')) | Select-Object -First 1
  }
  if ($ow) {
    # The pointer on the right button (「新建栅栏」) keeps the buttons.
    $or = Rect $ow
    [G]::MoveTo($or.R - 40, [int](($or.T + $or.B) / 2)); Start-Sleep -Milliseconds 600
    Log "offer window $($or.L),$($or.T),$($or.R),$($or.B)"
    FrState 'marquee-offer'
    RecMark 'click the offer to make a fence'
    [G]::Click($or.R - 40, [int](($or.T + $or.B) / 2)); Start-Sleep -Milliseconds 1500
    FrState 'new-fence-namebox'
    [G]::Type('My stuff'); Start-Sleep -Milliseconds 400; [G]::Press(0x0D); Start-Sleep -Seconds 1
    FrState 'new-fence-named'
    RecMark 'drag three PDF files into the new fence'
    $nf = FrFences | Where-Object { $_.title -eq 'My stuff' } | Select-Object -First 1
    $src = FrFences | Sort-Object { - $_.itemCount } | Select-Object -First 1
    foreach ($pdf in @(@(PfJson item list --fence $src.id --ext pdf) | Where-Object { $_ -and $_.name })) {
      $pp = ItemPt $src.id $pdf.name
      if ($pp -and $nf) { DragTo $pp[0] $pp[1] ($nf.windowRect.x + 170) ($nf.windowRect.y + 150) 14; Start-Sleep -Milliseconds 600 }
    }
    Start-Sleep -Seconds 1
    FrState 'rule-offer'
    $nw = [G]::FindWindowW('PecoFence.Notice', [NullString]::Value)
    if ($nw -ne [IntPtr]::Zero -and [G]::IsWindowVisible($nw)) {
      $r = Rect $nw
      RecMark 'take the rule offer'
      [G]::Click($r.R - 60, $r.B - 30); Start-Sleep -Milliseconds 1500
      FrState 'rule-added'
      CloseNotice | Out-Null
    }
  } else { [G]::Press(0x1B); Start-Sleep -Milliseconds 500 }
  DeskClick | Out-Null

  Case "walk ($Lang): roll up, hover"
  $f0 = FenceGet $f0.id
  RecMark 'double-click a fence title'
  DoubleClick ($f0.windowRect.x + 100) ($f0.windowRect.y + 14); Start-Sleep -Seconds 2
  FrState 'rolled'
  ParkMouse 800 850; Start-Sleep -Seconds 1
  FrState 'rolled-away'
  RecMark 'point at the rolled title'
  [G]::MoveTo($f0.windowRect.x + 100, $f0.windowRect.y + 14); Start-Sleep -Seconds 2
  FrState 'rolled-hover'
  ParkMouse 800 850; Start-Sleep -Seconds 2
  FrState 'rolled-left'
  DoubleClick ($f0.windowRect.x + 100) ($f0.windowRect.y + 14); Start-Sleep -Seconds 2
  ParkMouse 800 850

  Case "walk ($Lang): Peek over a window"
  Start-Process notepad.exe; Start-Sleep -Seconds 3
  RecMark 'press the Peek hotkey over a window'
  PeekKey; Start-Sleep -Seconds 2
  FrState 'peek'
  [G]::Type('re'); Start-Sleep -Seconds 2
  FrState 'peek-typed'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 800
  [G]::Press(0x1B); Start-Sleep -Milliseconds 800
  FrState 'peek-closed'
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force

  Case "walk ($Lang): quit and start again, Windows' icons"
  RecStop
  $m = TrayMenu
  if ($m -eq [IntPtr]::Zero) { CloseMenus; DeskClick | Out-Null; $m = TrayMenu }
  $qp = MenuPoint $m '^(Exit|Quit|退出)'
  RecStart "quit-$Lang" -Fps 15
  RecMark 'tray menu: quit'
  if ($qp) { [G]::Click($qp[0], $qp[1]) }
  Start-Sleep -Seconds 4
  RecStop
  FrState 'after-quit'
  Check 'PecoFence quit from the tray menu' (-not (PfAlive)) "menu=$m point=$($qp -join ',')"
  ParkMouse 800 850
  RecStart "restart-$Lang" -Fps 15
  RecMark 'start again'
  FrStart -Keep | Out-Null
  Start-Sleep -Seconds 6
  RecStop
  FrState 'second-start'
  RecStart "windesk-$Lang" -Fps 5
  RecMark 'tray menu: show the Windows desktop'
  $m = TrayMenu
  Log "tray menu: $(MenuTexts $m)"
  $t = MenuPick '(Windows 桌面|Windows desktop)' $m
  Log "picked: $t"
  Start-Sleep -Seconds 3
  FrState 'windows-desktop'
  RecStop
}
if ($Part -match 'W') { FrWalk }

# ---- K: the fences by keyboard alone. Every step logs the foreground window and, per fence,
# how many icons are selected and which has the focus, and takes a screenshot.
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class Hc {
  [StructLayout(LayoutKind.Sequential)] public struct HIGHCONTRAST { public uint cb; public uint flags; public IntPtr scheme; }
  [DllImport("user32.dll", SetLastError = true)] static extern bool SystemParametersInfoW(uint a, uint b, ref HIGHCONTRAST v, uint f);
  public static bool On() { var h = new HIGHCONTRAST(); h.cb = (uint)Marshal.SizeOf(typeof(HIGHCONTRAST)); SystemParametersInfoW(0x42, h.cb, ref h, 0); return (h.flags & 1) != 0; }
  public static bool Set(bool on) {
    var h = new HIGHCONTRAST(); h.cb = (uint)Marshal.SizeOf(typeof(HIGHCONTRAST)); SystemParametersInfoW(0x42, h.cb, ref h, 0);
    h.flags = on ? (h.flags | 1) : (h.flags & ~1u);
    return SystemParametersInfoW(0x43, h.cb, ref h, 3);
  }
}
"@
function FkFocus($w) { if ($w -and $w.raw -match ' focused=Some\("((?:[^"\\]|\\.)*)"\)') { return $Matches[1] }; return '-' }
function FkState($tag) {
  $ws = @(PfDump 4)
  $s = ($ws | ForEach-Object { "$($_.title)[sel=$(DumpVal $_ 'selected') foc=$(FkFocus $_) rolled=$($_.rolled) at=$($_.l),$($_.t),$($_.r),$($_.b)]" }) -join ' '
  Log "== ${tag}: fg=$(Fg) :: $s"
  Shot "k-$tag"
}
# Picks a row of the open popup menu with the Down key and Enter.
function FkMenuKeys($regex) {
  $m = MenuWin 5
  if ($m -eq [IntPtr]::Zero) { Log "no menu for '$regex'"; return $false }
  $rows = @(MenuRows $m | Where-Object { $_.text })
  $i = 0; $at = -1
  foreach ($r in $rows) { if ($r.text -match $regex) { $at = $i; break }; $i++ }
  Log "menu rows: $(MenuTexts $m); '$regex' is row $at"
  if ($at -lt 0) { CloseMenus; return $false }
  foreach ($k in 0..$at) { [G]::Press(0x28); Start-Sleep -Milliseconds 120 }
  Start-Sleep -Milliseconds 300
  [G]::Press(0x0D); Start-Sleep -Milliseconds 900
  return $true
}
function FrKeys() {
  trap { PfTrap $_; continue }
  Case 'keys: setup'
  PfStart -Lang zh-CN -Script (DumpScript 250 14)
  Start-Sleep -Seconds 4
  ClearFences
  Start-Sleep -Seconds 1
  Pf fence create --title '文档' --rect '120,60,340,300' | Out-Null
  Pf item move --glob '*.pdf' --to '文档' | Out-Null
  Pf item move --glob '*.txt' --to '文档' | Out-Null
  Pf item move --glob '*.docx' --to '文档' | Out-Null
  Pf fence create --title '图片' --rect '480,60,340,300' | Out-Null
  Pf item move --glob '*.png' --to '图片' | Out-Null
  Pf item move --glob '*.jpg' --to '图片' | Out-Null
  Pf fence create --title '影音' --rect '840,60,340,300' | Out-Null
  Pf item move --glob '*.mp3' --to '影音' | Out-Null
  Pf item move --glob '*.mp4' --to '影音' | Out-Null
  Pf fence create --title '压缩包' --rect '840,400,340,200' | Out-Null
  Pf item move --glob '*.zip' --to '压缩包' | Out-Null
  Pf item move --glob '*.7z' --to '压缩包' | Out-Null
  Pf item move --glob '*.rar' --to '压缩包' | Out-Null
  Pf fence merge '压缩包' --into '影音' | Out-Null
  Pf fence create --title '收起的' --rect '1200,60,300,200' | Out-Null
  Pf item move --glob '*.lnk' --to '收起的' | Out-Null
  Pf fence roll '收起的' | Out-Null
  Pf fence create --tool note --title '便签' --text "[ ] 买牛奶`n[x] 交水电费`n普通的一行字" --rect '120,400,300,220' | Out-Null
  Set-Content -LiteralPath "$PfRun\rows.cmd" -Value "@echo off`r`necho Open notepad ^| run=notepad.exe`r`necho Copy hello ^| copy=hello`r`necho plain line" -Encoding ASCII
  Pf fence create --script "$PfRun\rows.cmd" --title '脚本' --rect '480,400,300,220' | Out-Null
  Start-Sleep -Seconds 4
  DeskClick 1300 600 | Out-Null
  FkState 'setup'

  Case 'keys: from the desktop'
  foreach ($i in 1..3) { [G]::Press(0x09); Start-Sleep -Milliseconds 500; Log "Tab $i from the desktop: fg=$(Fg)" }
  DeskClick 1300 600 | Out-Null
  PeekKey; Start-Sleep -Seconds 1
  FkState 'peek-open'
  [G]::Press(0x09); Start-Sleep -Milliseconds 700
  FkState 'peek-tab'
  [G]::Press(0x27); Start-Sleep -Milliseconds 400
  [G]::Press(0x28); Start-Sleep -Milliseconds 400
  FkState 'peek-arrows'
  [G]::Press(0x09); Start-Sleep -Milliseconds 700
  FkState 'peek-tab-2'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 900
  FkState 'peek-esc-1'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 900
  FkState 'peek-esc-2'
  [G]::Press(0x27); Start-Sleep -Milliseconds 500
  FkState 'after-peek-right'

  Case 'keys: inside a fence'
  # A mouse click puts the keyboard into 「文档」 (the only way without the Peek).
  $first = @(PfJson item list --fence '文档') | Where-Object { $_ -and $_.name } | Sort-Object name | Select-Object -First 1
  $p = ItemPt '文档' $first.name
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  ParkMouse 1300 700
  FkState 'click-first'
  [G]::Press(0x27); Start-Sleep -Milliseconds 400
  FkState 'right'
  [G]::KeyDown(0x11); [G]::Press(0x27); [G]::KeyUp(0x11); Start-Sleep -Milliseconds 400
  FkState 'ctrl-right'
  [G]::Press(0x20); Start-Sleep -Milliseconds 400
  FkState 'space'
  [G]::Type('会'); Start-Sleep -Milliseconds 600
  FkState 'typeahead'
  [G]::Press(0x71); Start-Sleep -Milliseconds 900
  FkState 'f2'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 700
  FkState 'f2-esc'
  [G]::Press(0x5D); Start-Sleep -Milliseconds 900
  FkState 'menukey'
  CloseMenus; Start-Sleep -Milliseconds 500
  FkState 'menukey-esc'
  [G]::KeyDown(0x12); [G]::Press(0x0D); [G]::KeyUp(0x12); Start-Sleep -Seconds 2
  FkState 'alt-enter'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 900
  FkState 'alt-enter-esc'
  [G]::Press(0x0D); Start-Sleep -Seconds 3
  FkState 'enter-opened'
  if ((Fg) -match 'Notepad') { Chord 0x12 0x73; Start-Sleep -Seconds 1 }
  FkState 'enter-closed'
  [G]::Press(0x28); Start-Sleep -Milliseconds 400
  FkState 'down-after-open'
  [G]::KeyDown(0x11); [G]::Press(0x41); [G]::KeyUp(0x11); Start-Sleep -Milliseconds 400
  FkState 'ctrl-a'
  [G]::Press(0x09); Start-Sleep -Milliseconds 700
  FkState 'tab-next'
  [G]::Press(0x09); Start-Sleep -Milliseconds 700
  FkState 'tab-next-2'

  Case 'keys: tabs'
  [G]::KeyDown(0x11); [G]::Press(0x09); [G]::KeyUp(0x11); Start-Sleep -Milliseconds 900
  FkState 'ctrl-tab'
  [G]::Press(0x27); Start-Sleep -Milliseconds 400
  FkState 'ctrl-tab-right'
  [G]::Press(0x09); Start-Sleep -Milliseconds 700
  FkState 'tab-next-3'
  [G]::Press(0x09); Start-Sleep -Milliseconds 700
  FkState 'tab-next-4'

  Case 'keys: the fence menu, roll up, delete'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  FkState 'esc-selection'
  [G]::Press(0x5D); Start-Sleep -Milliseconds 900
  FkState 'fence-menu'
  $ok = FkMenuKeys '^收起'
  FkState 'rolled-by-keys'
  [G]::Press(0x27); Start-Sleep -Milliseconds 400
  [G]::Press(0x0D); Start-Sleep -Milliseconds 600
  FkState 'rolled-right-enter'
  [G]::Press(0x09); Start-Sleep -Milliseconds 700
  FkState 'rolled-tab'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  [G]::Press(0x5D); Start-Sleep -Milliseconds 900
  $ok = FkMenuKeys '^删除栅栏'
  Start-Sleep -Milliseconds 800
  FkState 'fence-deleted'
  foreach ($i in 1..2) { [G]::Press(0x09); Start-Sleep -Milliseconds 500; Log "Tab $i with the undo card up: fg=$(Fg)" }
  FkState 'card-tab'
  [G]::KeyDown(0x11); [G]::Press(0x5A); [G]::KeyUp(0x11); Start-Sleep -Seconds 1
  FkState 'ctrl-z'

  Case 'keys: tool fences'
  $nw = [G]::FindWindowW('PecoFence.Fence', '便签'); $nr = Rect $nw
  [G]::Click($nr.L + 150, $nr.B - 30); Start-Sleep -Milliseconds 500
  FkState 'note-clicked'
  foreach ($k in 0x28, 0x20, 0x0D) { [G]::Press($k); Start-Sleep -Milliseconds 400 }
  FkState 'note-keys'
  if ([G]::FindWindowW('PecoFence.NoteEdit', [NullString]::Value) -ne [IntPtr]::Zero) { [G]::Press(0x1B); Start-Sleep -Milliseconds 500 }
  [G]::Press(0x09); Start-Sleep -Milliseconds 600
  FkState 'note-tab'
  $sw = [G]::FindWindowW('PecoFence.Fence', '脚本'); $sr = Rect $sw
  [G]::Click($sr.L + 200, $sr.B - 20); Start-Sleep -Milliseconds 500
  foreach ($k in 0x28, 0x0D) { [G]::Press($k); Start-Sleep -Milliseconds 400 }
  FkState 'script-keys'
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force

  Case 'keys: the Peek search list and the tray'
  DeskClick 1300 600 | Out-Null
  PeekKey; Start-Sleep -Seconds 1
  [G]::Type('re'); Start-Sleep -Seconds 1
  [G]::Press(0x28); Start-Sleep -Milliseconds 300; [G]::Press(0x28); Start-Sleep -Milliseconds 300
  FkState 'search-down'
  [G]::Press(0x09); Start-Sleep -Milliseconds 400
  FkState 'search-tab'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 600; [G]::Press(0x1B); Start-Sleep -Milliseconds 600
  FkState 'search-closed'
  # Win+B: the tray; the arrow keys to PecoFence's icon; the Menu key.
  [G]::KeyDown(0x5B); [G]::Press(0x42); [G]::KeyUp(0x5B); Start-Sleep -Milliseconds 900
  $name = ''
  foreach ($i in 1..8) {
    $name = [Windows.Automation.AutomationElement]::FocusedElement.Current.Name
    Log "tray focus ${i}: '$name' fg=$(Fg)"
    if ($name -match '^PecoFence') { break }
    if ($name -match 'hidden icons|chevron|overflow') { [G]::Press(0x0D); Start-Sleep -Milliseconds 800 } else { [G]::Press(0x27); Start-Sleep -Milliseconds 500 }
  }
  FkState 'tray-focus'
  [G]::Press(0x5D); Start-Sleep -Seconds 1
  $m = MenuWin 3
  Log "tray menu by the Menu key: $(MenuTexts $m)"
  FkState 'tray-menu-keys'
  [G]::Press(0x28); Start-Sleep -Milliseconds 300
  FkState 'tray-menu-down'
  CloseMenus; [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  FkState 'tray-menu-closed'

  Case 'keys: a high contrast theme'
  $p = ItemPt '文档' $first.name
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 400
  [G]::KeyDown(0x11); [G]::Press(0x27); [G]::KeyUp(0x11); Start-Sleep -Milliseconds 300
  ParkMouse 1300 700
  Log "high contrast on: $([Hc]::Set($true))"
  Start-Sleep -Seconds 8
  FkState 'hc-on'
  [G]::Press(0x5D); Start-Sleep -Seconds 1
  FkState 'hc-menu'
  CloseMenus
  Log "high contrast off: $([Hc]::Set($false))"
  Start-Sleep -Seconds 8
  FkState 'hc-off'
  Check 'high contrast is off again' (-not [Hc]::On()) ''
}
if ($Part -match 'K') { FrKeys }

# ---- S: a first start while C: keeps every write waiting. A 6 GB file without data, copied by
# the shell from a helper process, stalls the volume for about 40 s on these boxes; the job's
# own log and the recorder's frames would wait too, so the frames stay in memory.
function FrSlowDisk() {
  trap { PfTrap $_; continue }
  Case 'slow disk: the first start'
  if (PfAlive) { PfQuit }
  Remove-Item -Recurse -Force "$PfApp\config", "$PfApp\notices.json" -ErrorAction SilentlyContinue
  if (-not (Test-Path "$PfApp\pecofence.exe")) { New-Item -ItemType Directory -Force $PfApp | Out-Null; Copy-Item "$PfBin\*" $PfApp -Force }
  New-Item -ItemType Directory -Force "$PfRun\big" | Out-Null
  $f = [IO.File]::Create("$PfRun\big\fr-big.bin"); $f.SetLength([int64]6 * 1GB); $f.Close()
  ParkMouse 800 850
  $cmd = "Add-Type -AssemblyName Microsoft.VisualBasic; [Microsoft.VisualBasic.FileIO.FileSystem]::CopyFile('$PfRun\big\fr-big.bin', '$PfRun\big\fr-copy.bin', 'AllDialogs')"
  $sw = [Diagnostics.Stopwatch]::StartNew()
  $copy = Start-Process powershell -PassThru -WindowStyle Hidden -ArgumentList '-NoProfile', '-Command', $cmd
  Start-Sleep -Seconds 6
  RecStart 'slow-start' -Fps 10 -InMemory
  RecMark 'start PecoFence'
  $env:PECOFENCE_INSTANCE = $PfInst
  $env:RUST_LOG = 'pecofence=debug'
  $t0 = $sw.ElapsedMilliseconds
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $PfApp
  $tFence = -1; $tCard = -1; $tIcons = -1
  while ($sw.ElapsedMilliseconds - $t0 -lt 30000) {
    $t = $sw.ElapsedMilliseconds - $t0
    if ($tFence -lt 0 -and (VisFences) -gt 0) { $tFence = $t }
    if ($tIcons -lt 0 -and $tFence -ge 0 -and -not [W]::IconsVisible()) { $tIcons = $t }
    $nw = [G]::FindWindowW('PecoFence.Notice', [NullString]::Value)
    if ($tCard -lt 0 -and $nw -ne [IntPtr]::Zero -and [G]::IsWindowVisible($nw)) { $tCard = $t }
    if ($tFence -ge 0 -and $tIcons -ge 0 -and $tCard -ge 0 -and $t -gt $tCard + 3000) { break }
    Start-Sleep -Milliseconds 20
  }
  $tEnd = $sw.ElapsedMilliseconds - $t0
  RecStop
  $copyDone = $copy.WaitForExit(180000)
  $tCopy = $sw.ElapsedMilliseconds
  Log "slow disk: PecoFence started $t0 ms into the copy; first fence window at $tFence ms, Windows icons hidden at $tIcons ms, tip card at $tCard ms after the start; watched for $tEnd ms; the copy ended=$copyDone after $tCopy ms"
  Shot 's-after'
  Check 'slow disk: the fences, the hidden Windows icons and the tip card all came' ($tFence -ge 0 -and $tIcons -ge 0 -and $tCard -ge 0) "fence $tFence ms, icons hidden $tIcons ms, card $tCard ms"
  if (PfAlive) { PfQuit }
  Remove-Item -Recurse -Force "$PfRun\big" -ErrorAction SilentlyContinue
}
if ($Part -match 'S') { FrSlowDisk }

# ---- C: the checks of this package's fixes.
# The focus ring of the icon whose middle is at ($x, $y): the longest row of dark pixels in the
# band where the ring's top edge runs (the ring is about 76 px wide; an icon has no dark row there).
function FrRingRow($x, $y) {
  $b = [Px]::Grab($x - 34, $y - 40, 68, 14)
  $best = 0
  foreach ($r in 0..13) {
    $n = 0
    foreach ($c in 0..67) { $p = $b.GetPixel($c, $r); if ($p.R + $p.G + $p.B -lt 200) { $n++ } }
    if ($n -gt $best) { $best = $n }
  }
  $b.Dispose()
  return $best
}
function FrChecks() {
  trap { PfTrap $_; continue }
  # C1, C2: a first start as a user (the cards show). The fourth fence is made from a rectangle:
  # no card comes up while its name box is open, the roll-up tip shows once the name is typed,
  # and three PDF files dragged into the new fence bring the rule offer in its place.
  Case 'C1 the roll-up tip waits for the name box'
  FrStart | Out-Null
  $n = WaitWin 'PecoFence.Notice' 30
  Start-Sleep -Seconds 3
  # C7: the tip on screen follows a change of the display language (it stayed English under
  # fences named in Chinese until 「Next」 was pressed).
  $t0 = [L]::Text($n)
  & "$PfApp\pecofence-cli.exe" settings set language zh-CN | Out-Null; Start-Sleep -Milliseconds 800
  $t1 = [L]::Text($n)
  Shot 'c7-card-language'
  Check 'C7 the tip on screen follows a change of the display language' ($t0 -like 'Tip 1/*' -and $t1 -like '小技巧 1/*') "before=[$t0] after=[$t1]"
  & "$PfApp\pecofence-cli.exe" settings set language en | Out-Null; Start-Sleep -Milliseconds 800
  CloseNotice | Out-Null
  Start-Sleep -Seconds 1
  $fr = FreeRect 360 260 300 300
  DragTo ($fr[0] + 10) ($fr[1] + 10) ($fr[0] + 340) ($fr[1] + 240) 12
  $ow = @([W]::Tops('PecoFence.MarqueeOffer')) | Select-Object -First 1
  $or = Rect $ow
  [G]::Click($or.R - 40, [int](($or.T + $or.B) / 2)); Start-Sleep -Milliseconds 1800
  Shot 'c1-namebox'
  Check 'C1 setup: the new fence has its name box open' ((Fg) -match 'PecoFence.Rename') "fg=$(Fg) fences=$(@(FrFences).Count)"
  Check 'C1 no card comes up while the name box is open' ((FrNotice) -eq 'none') "notice=$(FrNotice)"
  [G]::Type('My stuff'); Start-Sleep -Milliseconds 400; [G]::Press(0x0D); Start-Sleep -Milliseconds 1500
  Shot 'c1-named'
  $roll = @(PfLogAll | Select-String 'welcome notice shown notice=RollTip' -SimpleMatch).Count
  Check 'C1 the roll-up tip shows once the name is typed' ((FrNotice) -ne 'none' -and $roll -eq 1) "notice=$(FrNotice) roll tip lines=$roll"

  Case 'C2 the rule offer takes the place of the roll-up tip'
  $nf = FrFences | Where-Object { $_.title -eq 'My stuff' } | Select-Object -First 1
  $src = FrFences | Sort-Object { - $_.itemCount } | Select-Object -First 1
  foreach ($pdf in @(@(PfJson item list --fence $src.id --ext pdf) | Where-Object { $_ -and $_.name })) {
    $pp = ItemPt $src.id $pdf.name
    if ($pp -and $nf) { DragTo $pp[0] $pp[1] ($nf.windowRect.x + 170) ($nf.windowRect.y + 150) 14; Start-Sleep -Milliseconds 600 }
  }
  Start-Sleep -Seconds 2
  ParkMouse 800 850
  Shot 'c2-rule-offer'
  $rule = @(PfLogAll | Select-String 'welcome notice shown notice=RuleTip' -SimpleMatch).Count
  $cards = @([W]::Tops('PecoFence.Notice')).Count
  Check 'C2 setup: three PDF files are in the new fence' (@(@(PfJson item list --fence $nf.id --ext pdf) | Where-Object { $_ -and $_.name }).Count -eq 3) ''
  Check 'C2 the rule offer shows although the roll-up tip was up' ($rule -eq 1) "rule offer lines=$rule cards on screen=$cards notice=$(FrNotice)"
  PfQuit

  # C3: Tab during a Peek puts the keyboard into a fence; after Esc the keys go to the window in
  # front, and no icon keeps the focus ring.
  Case 'C3 no focus ring is left after a Peek'
  PfStart -Lang zh-CN -Script (DumpScript 250 6)
  Start-Sleep -Seconds 4
  ClearFences
  Pf fence create --title 'Kb' --rect '520,160,340,240' | Out-Null
  Pf item move --glob '*.pdf' --to 'Kb' | Out-Null
  Start-Sleep -Seconds 2
  DeskClick 1300 600 | Out-Null
  ParkMouse 1300 700
  PeekKey; Start-Sleep -Seconds 1
  $w = $null
  foreach ($i in 1..4) { [G]::Press(0x09); Start-Sleep -Milliseconds 700; $w = DumpOf 'Kb' 4; if ((DumpVal $w 'selected') -eq '1') { break } }
  $name = FkFocus $w
  $p = ItemPt 'Kb' $name
  $during = FrRingRow $p[0] $p[1]
  Shot 'c3-during'
  Check 'C3 setup: Tab in the Peek gives an icon of the fence the focus ring' ((DumpVal $w 'selected') -eq '1' -and $during -ge 40) "focused=[$name] ring row=$during px"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 1500
  $after = FrRingRow $p[0] $p[1]
  $w = DumpOf 'Kb' 4
  Shot 'c3-after'
  Check 'C3 after Esc the keys are with another window and no icon shows the focus ring' ((Fg) -notmatch 'PecoFence' -and $after -lt 10) "fg=$(Fg) ring row=$after px (during the Peek $during) focus_ring=$(DumpVal $w 'focus_ring') selected=$(DumpVal $w 'selected')"

  # C5: the Peek was opened from the desktop and closed with Esc: the keyboard is back on the
  # desktop (Windows gave it to the taskbar). Opened while a fence had the keyboard: back in
  # that fence, and the arrow keys go on there.
  Case 'C5 the keyboard goes back where it was after a Peek'
  Check 'C5 opened from the desktop: after Esc the desktop has the keyboard' ((Fg) -match '^(Progman|WorkerW)') "fg=$(Fg)"
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  ParkMouse 1300 700
  $was = Fg
  PeekKey; Start-Sleep -Seconds 1
  $during = Fg
  [G]::Press(0x1B); Start-Sleep -Milliseconds 1500
  $now = Fg
  [G]::Press(0x27); Start-Sleep -Milliseconds 500
  $w = DumpOf 'Kb' 4
  Check 'C5 opened from a fence: after Esc that fence has the keyboard and Right moves on' ($was -match "PecoFence.Fence 'Kb'" -and $during -match 'PeekDimmer' -and $now -eq $was -and (FkFocus $w) -ne $name -and (DumpVal $w 'selected') -eq '1') "before=$was during=$during after=$now focused=[$(FkFocus $w)] (was [$name])"

  # C4: 退出 in the tray menu. File Explorer shows its icons about a third of a second after it
  # is asked; the fences used to be gone after 167 ms, and the desktop was empty in between.
  # (The sampling is the one of owner2-ux-fence-life.ps1 part Q.)
  Case 'C4 the desktop is not empty at exit'
  PfFresh
  Pf settings set language zh-CN | Out-Null
  Start-Sleep -Seconds 3
  $fr = Rect ([G]::FindWindowW('PecoFence.Fence', '程序'))
  ParkMouse 800 850
  $sy = 104; $sh = 60; $fx = $fr.L + 10
  # C6 first, with the same sampling: 「临时显示 Windows 桌面」 had the same empty moment.
  Case 'C6 the desktop is not empty on the way to the Windows desktop'
  $samples = New-Object System.Collections.Generic.List[object]
  $sw = [Diagnostics.Stopwatch]::StartNew()
  Start-Process "$PfApp\pecofence-cli.exe" -ArgumentList 'windows-desktop start' -WindowStyle Hidden | Out-Null
  while ($sw.ElapsedMilliseconds -lt 2000) {
    $t = $sw.ElapsedMilliseconds
    $bs = [Px]::Grab(0, $sy, 1600, $sh)
    $samples.Add(@($t, [Px]::MaxInk($bs, 8, 80, 0, $sh - 1), [Px]::MaxInk($bs, $fx, $fx + 200, 0, $sh - 1)))
    $bs.Dispose()
  }
  Log ("C6 samples (ms:Windows icon/fence): " + ((@($samples | Where-Object { $_[0] -lt 1400 }) | ForEach-Object { "$($_[0]):$($_[1])/$($_[2])" }) -join ' '))
  $goneAt = @($samples | Where-Object { $_[2] -lt 10 } | Select-Object -First 1)
  $tGone = -1; if ($goneAt.Count) { $tGone = $goneAt[0][0] }
  $back = @($samples | Where-Object { $_[1] -ge 25 } | Select-Object -First 1)
  $tBack = -1; if ($back.Count) { $tBack = $back[0][0] }
  Check 'C6 setup: the fences went and the Windows icons show' ((VisFences) -eq 0 -and [W]::IconsVisible() -and $tGone -gt 0 -and $tBack -gt 0) "fences=$(VisFences) fence gone at $tGone ms, Windows icon back at $tBack ms"
  Check 'C6 the fences are still there when the Windows icons come' ($tBack - $tGone -le 50) "the desktop is empty for $($tBack - $tGone) ms (fence gone at $tGone ms, Windows icon back at $tBack ms)"
  Pf windows-desktop end | Out-Null
  Start-Sleep -Seconds 2
  Check 'C6 back: the fences show and the Windows icons are hidden' ((VisFences) -ge 3 -and -not [W]::IconsVisible()) "fences=$(VisFences) icons=$([W]::IconsVisible())"

  Case 'C4 the desktop is not empty at exit'
  $m = TrayMenu
  Start-Sleep -Milliseconds 400
  $pt = MenuPoint $m '^退出'
  [G]::MoveTo($pt[0], $pt[1]); Start-Sleep -Milliseconds 300
  $b0 = [Px]::Grab(0, $sy, 1600, $sh); $f0 = [Px]::MaxInk($b0, $fx, $fx + 200, 0, $sh - 1); $b0.Dispose()
  RecStart "c4-quit" -Fps 15
  RecMark '托盘菜单 → 退出 PecoFence'
  [G]::Click($pt[0], $pt[1])
  # Every sample: ms, ink of the Windows icon in the second cell at the left, ink of the fence.
  $samples = New-Object System.Collections.Generic.List[object]
  $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.ElapsedMilliseconds -lt 2000) {
    $t = $sw.ElapsedMilliseconds
    $bs = [Px]::Grab(0, $sy, 1600, $sh)
    $samples.Add(@($t, [Px]::MaxInk($bs, 8, 80, 0, $sh - 1), [Px]::MaxInk($bs, $fx, $fx + 200, 0, $sh - 1)))
    $bs.Dispose()
  }
  RecStop
  $quit = WaitNoPf 15
  Log ("C4 samples (ms:Windows icon/fence): " + ((@($samples | Where-Object { $_[0] -lt 1200 }) | ForEach-Object { "$($_[0]):$($_[1])/$($_[2])" }) -join ' '))
  $goneAt = @($samples | Where-Object { $_[2] -lt 10 } | Select-Object -First 1)
  $tGone = -1; if ($goneAt.Count) { $tGone = $goneAt[0][0] }
  $back = @($samples | Where-Object { $_[1] -ge 25 } | Select-Object -First 1)
  $tBack = -1; if ($back.Count) { $tBack = $back[0][0] }
  Check 'C4 setup: the fence showed, PecoFence quit, the Windows icons are back' ($f0 -ge 100 -and $quit -and [W]::IconsVisible() -and $tGone -gt 0 -and $tBack -gt 0) "fence ink before $f0; quit=$quit; fence gone at $tGone ms, Windows icon back at $tBack ms"
  Check 'C4 the fences are still there when the Windows icons come back' ($tBack - $tGone -le 50) "the desktop is empty for $($tBack - $tGone) ms (fence gone at $tGone ms, Windows icon back at $tBack ms)"
}
if ($Part -match 'C') { FrChecks }

Case 'cleanup'
if (PfAlive) { PfQuit }
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
FrClean
[Anim]::Set($AnimWas)
if ([Hc]::On()) { [Hc]::Set($false) | Out-Null }
Check 'test files removed' (@(Get-ChildItem $PfDesk | Where-Object { $FrFiles -contains $_.Name -or $FrDirs -contains $_.Name }).Count -eq 0) ''
