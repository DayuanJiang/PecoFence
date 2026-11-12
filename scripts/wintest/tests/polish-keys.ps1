# Fence polish round 2026-11-11, package "keys": what the keyboard does to the items of a fence.
#   R  a rolled-up fence leaves its hidden items alone (Ctrl+A, Ctrl+C, F2, the Menu key, Delete,
#      Enter), drops the selection when the roll-up lands, and Tab still moves on (C3)
#   D  after Delete the icon that followed takes the selection and the focus, from the key and
#      from the item menu (C7 / UX3)
#   H  after icons slide, the hover fill is on the icon under the resting pointer (C1)
#   F  F2 with several icons selected renames the focused one (UX10)
#   E  Esc in a filtered folder fence clears the filter first, the selection on the next press (S4)
#   V  a rename that sorts the icon out of view: the fence scrolls to it (C9)
#   N  a Delete that removes nothing (No in Windows' question, the Recycle Bin icon) leaves no
#      wait behind: the icon's neighbour takes no selection when the icon leaves later; a Yes
#      still selects the icon that followed (follow-up to C7)
# Parts: run.sh --var Part=RD ... (default: all). --var Rec=1 also records each part with rec.ps1
# (for before / after animations). Runs on a 1600x900 box at 100 %.
if (-not $Part) { $Part = 'RDHFEVN' }
# An error inside a part's block becomes a failed check and the next line runs.
trap { PfTrap $_; continue }
if ($Rec) { . C:\pf\kit\rec.ps1 }
function PkRec($name, $x, $y, $w, $h) { if ($Rec) { RecStart $name -X $x -Y $y -W $w -H $h -Fps 30 } }
function PkMark($t) { if ($Rec) { RecMark $t } }
function PkStop() { if ($Rec) { RecStop } }

$PkDir = 'C:\pf\pk-folder'
# The files of this test (every name starts with "pk "), its folder, and what it sent to the
# Recycle Bin.
function PkClean() {
  Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue | Where-Object { $_.Name -like 'pk *' } | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
  if (Test-Path $PkDir) { Remove-Item -LiteralPath $PkDir -Recurse -Force -ErrorAction SilentlyContinue }
  Clear-RecycleBin -Force -ErrorAction SilentlyContinue
}
# A fence sorted by name that holds new desktop files with these names.
function PkFence($title, $rect, $names) {
  foreach ($n in $names) { DeskFile $n | Out-Null }
  Start-Sleep -Seconds 2
  Pf fence create --title $title --rect $rect | Out-Null
  Pf item move @($names | ForEach-Object { Join-Path $PfDesk $_ }) --to $title | Out-Null
  Pf fence set $title sort name | Out-Null
  Start-Sleep -Seconds 2
}
# From a dump line: how many icons are selected, and the name of the focused one ('' for none).
function PkSelOf($w) { return "$(DumpVal $w 'selected')" }
function PkFocusOf($w) { if ($w -and $w.raw -match ' focused=Some\("((?:[^"\\]|\\.)*)"\)') { return $Matches[1] }; return '' }
function PkSel($title) { return (PkSelOf (DumpOf $title)) }
function PkFocus($title) { return (PkFocusOf (DumpOf $title)) }
# What Ctrl+C puts on the clipboard (file names), '' for nothing.
function PkCopy() {
  [Windows.Forms.Clipboard]::Clear()
  Chord 0x11 0x43; Start-Sleep -Milliseconds 500
  return ((ClipFiles | ForEach-Object { Split-Path $_ -Leaf }) -join ' | ')
}
# Which of these desktop files still exist.
function PkLeft($names) { return (@($names | Where-Object { Test-Path -LiteralPath (Join-Path $PfDesk $_) }) -join ' | ') }

Case 'setup'
CloseWindows
PkClean
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
GreyWallpaper
PfStart -Script (DumpScript 250 20)
Pf settings set featureTips false | Out-Null
Start-Sleep -Seconds 3

if ($Part -match 'R') {
  Case 'R a rolled-up fence leaves its hidden items alone'
  ClearFences | Out-Null
  $rn = @('pk r1 季度报告.txt', 'pk r2 预算.txt', 'pk r3 会议纪要与分工.txt')
  PkFence 'PkR' '300,200,420,170' $rn
  PkFence 'PkT' '300,440,300,150' @('pk t1 便签.txt')
  $p = CellPt 'PkR' 0
  $p1 = CellPt 'PkR' 1
  PkRec 'R' 280 180 460 240
  PkMark '选中一个文件'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
  Check 'R setup: one icon is selected' ((PkSel 'PkR') -eq '1') "selected=$(PkSel 'PkR')"
  PkMark '双击标题，把栅栏收起'
  $wr = (FenceGet 'PkR').windowRect
  DblClick ($wr.x + 300) ($wr.y + 14); Start-Sleep -Milliseconds 700
  ParkMouse; Start-Sleep -Milliseconds 1500
  $w = DumpOf 'PkR'
  Check 'R the fence is rolled up and has the keyboard' ($w.rolled -and (DumpVal $w 'peeking') -eq 'false' -and (Fg) -match 'PkR') "rolled=$($w.rolled) peeking=$(DumpVal $w 'peeking') foreground=$(Fg)"
  Check 'R the selection is dropped when the roll-up lands' ((PkSelOf $w) -eq '0') "selected=$(PkSelOf $w)"
  # The keys that act on items, on the rolled fence: Ctrl+A, Ctrl+C, F2, the Menu key, Delete.
  PkMark '按 Ctrl+A，再按 Delete'
  Chord 0x11 0x41; Start-Sleep -Milliseconds 500
  Check 'R Ctrl+A selects nothing in a rolled fence' ((PkSel 'PkR') -eq '0') "selected=$(PkSel 'PkR')"
  $clip = PkCopy
  Check 'R Ctrl+C copies nothing' ($clip -eq '') "clipboard=[$clip]"
  [G]::Press(0x71); Start-Sleep -Milliseconds 300
  $ren = WaitWin 'PecoFence.Rename' 1
  Check 'R F2 opens no name box' ($ren -eq [IntPtr]::Zero) "rename window=$ren"
  if ($ren -ne [IntPtr]::Zero) { [G]::Press(0x1B); Start-Sleep -Milliseconds 400 }
  [G]::Press(0x5D); Start-Sleep -Milliseconds 700
  $m = MenuWin 3
  $texts = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
  Check 'R the Menu key opens the fence menu, not an item menu' ($texts -match 'Lock position and size') "menu: $texts"
  CloseMenus; Start-Sleep -Milliseconds 500
  [G]::Press(0x2E); Start-Sleep -Milliseconds 2500
  $left = PkLeft $rn; $bin = BinNames
  Check 'R Delete on a rolled fence deletes nothing' ($left -eq ($rn -join ' | ') -and $bin -notmatch 'pk r') "files left: [$left]; Recycle Bin: [$bin]"
  PkMark '展开栅栏'
  Pf fence unroll PkR | Out-Null; Start-Sleep -Milliseconds 1200
  $n = @(@(PfJson item list --fence PkR) | Where-Object { $_ -and $_.name }).Count
  Check 'R unrolled again: the three icons are there' ($n -eq 3) "items=$n"
  PkStop
  foreach ($n in $rn) { if (-not (Test-Path -LiteralPath (Join-Path $PfDesk $n))) { DeskFile $n | Out-Null } }
  Start-Sleep -Seconds 2
  Pf item move @($rn | ForEach-Object { Join-Path $PfDesk $_ }) --to PkR | Out-Null

  Case 'R the keys a rolled fence keeps'
  # Enter opens nothing; Tab still moves the keyboard to the next fence that has icons.
  $wr = (FenceGet 'PkR').windowRect
  DblClick ($wr.x + 300) ($wr.y + 14); Start-Sleep -Milliseconds 700
  ParkMouse; Start-Sleep -Milliseconds 1500
  $w = DumpOf 'PkR'
  if (-not $w.rolled) { Pf fence roll PkR | Out-Null; Start-Sleep -Milliseconds 900 }
  $np0 = @(Get-Process notepad -ErrorAction SilentlyContinue).Count
  [G]::Press(0x0D); Start-Sleep -Milliseconds 2000
  $np1 = @(Get-Process notepad -ErrorAction SilentlyContinue).Count
  Check 'R Enter opens nothing' ($np1 -eq $np0 -and (Fg) -match 'PkR') "notepad processes before=$np0 after=$np1 foreground=$(Fg)"
  [G]::Press(0x09); Start-Sleep -Milliseconds 600
  $fg1 = Fg
  Check 'R Tab moves the keyboard to another fence' ($fg1 -match 'PecoFence\.Fence' -and $fg1 -notmatch 'PkR') "after Tab: $fg1"
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force

  Case 'R a hover peek that closed'
  # The fence opens under the pointer, an icon is clicked, the pointer leaves and the fence
  # rolls up again: Delete must not reach the icon that was selected in it.
  $wr = (FenceGet 'PkR').windowRect
  [G]::MoveTo($wr.x + 296, $wr.y + 14); Start-Sleep -Milliseconds 120; [G]::MoveTo($wr.x + 300, $wr.y + 14)
  Start-Sleep -Milliseconds 1500
  [G]::Click($p1[0], $p1[1]); Start-Sleep -Milliseconds 700
  $w = DumpOf 'PkR'
  Check 'R hover peek: an icon of the opened fence is selected' ((PkSelOf $w) -eq '1' -and (DumpVal $w 'peeking') -eq 'true') "selected=$(PkSelOf $w) peeking=$(DumpVal $w 'peeking')"
  ParkMouse; Start-Sleep -Milliseconds 2000
  $w = DumpOf 'PkR'
  Check 'R hover peek closed: rolled up, nothing selected' ($w.rolled -and (DumpVal $w 'peeking') -eq 'false' -and (PkSelOf $w) -eq '0') "rolled=$($w.rolled) peeking=$(DumpVal $w 'peeking') selected=$(PkSelOf $w)"
  [G]::Press(0x2E); Start-Sleep -Milliseconds 2500
  $left = PkLeft $rn
  Check 'R hover peek closed: Delete deletes nothing' ($left -eq ($rn -join ' | ')) "files left: [$left]; Recycle Bin: [$(BinNames)]"
  Pf fence unroll PkR | Out-Null; Start-Sleep -Milliseconds 900

  Case 'R rolled from the command line with an icon selected'
  $had = PkLeft $rn
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600
  ParkMouse
  Pf fence roll PkR | Out-Null; Start-Sleep -Milliseconds 1000
  $w = DumpOf 'PkR'
  Check 'R rolled by the CLI: nothing selected, the fence has the keyboard' ($w.rolled -and (PkSelOf $w) -eq '0' -and (Fg) -match 'PkR') "rolled=$($w.rolled) selected=$(PkSelOf $w) foreground=$(Fg)"
  [G]::Press(0x2E); Start-Sleep -Milliseconds 2500
  $left = PkLeft $rn
  Check 'R rolled by the CLI: Delete deletes nothing' ($left -eq $had) "files before: [$had]; after: [$left]"
  Pf fence unroll PkR | Out-Null; Start-Sleep -Milliseconds 900
}

if ($Part -match 'D') {
  Case 'D after Delete the next icon takes the selection'
  ClearFences | Out-Null
  $dn = @(1..6 | ForEach-Object { "pk d$_ 文件$_.txt" })
  PkFence 'PkD' '300,200,520,170' $dn
  $p = CellPt 'PkD' 2
  PkRec 'D' 280 180 560 210
  PkMark '选中第三个文件'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
  ParkMouse
  PkMark '按 Delete'
  [G]::Press(0x2E); Start-Sleep -Milliseconds 2500
  $w = DumpOf 'PkD'
  Check 'D Delete: the icon that followed is selected and focused' ((PkSelOf $w) -eq '1' -and (PkFocusOf $w) -like 'pk d4*') "selected=$(PkSelOf $w) focused=[$(PkFocusOf $w)]"
  PkMark '再按 Delete'
  [G]::Press(0x2E); Start-Sleep -Milliseconds 2500
  $w = DumpOf 'PkD'
  Check 'D a second Delete deletes that one and moves on' ((PkSelOf $w) -eq '1' -and (PkFocusOf $w) -like 'pk d5*') "selected=$(PkSelOf $w) focused=[$(PkFocusOf $w)]"
  PkMark '第三次按 Delete'
  [G]::Press(0x2E); Start-Sleep -Milliseconds 2500
  $w = DumpOf 'PkD'
  $left = PkLeft $dn
  Check 'D three Deletes walked down the list' ($left -eq "$($dn[0]) | $($dn[1]) | $($dn[5])" -and (PkFocusOf $w) -like 'pk d6*') "files left: [$left] focused=[$(PkFocusOf $w)]"
  PkStop
  # The last icon: the one before it takes over, and an arrow goes on from there.
  [G]::Press(0x2E); Start-Sleep -Milliseconds 2500
  $w = DumpOf 'PkD'
  Check 'D Delete on the last icon: the one before it is selected' ((PkSelOf $w) -eq '1' -and (PkFocusOf $w) -like 'pk d2*') "selected=$(PkSelOf $w) focused=[$(PkFocusOf $w)]"
  [G]::Press(0x25); Start-Sleep -Milliseconds 600
  $w = DumpOf 'PkD'
  Check 'D the Left arrow goes on from there' ((PkSelOf $w) -eq '1' -and (PkFocusOf $w) -like 'pk d1*') "selected=$(PkSelOf $w) focused=[$(PkFocusOf $w)]"
  # Delete from the item menu.
  $p = CellPt 'PkD' 0
  [G]::RClick($p[0], $p[1])
  $m = MenuWin 8
  $t = MenuPick '^Delete' $m
  Start-Sleep -Milliseconds 2500
  $w = DumpOf 'PkD'
  Check 'D Delete in the item menu: the icon that followed is selected' ($t -and (PkSelOf $w) -eq '1' -and (PkFocusOf $w) -like 'pk d2*') "picked=[$t] selected=$(PkSelOf $w) focused=[$(PkFocusOf $w)] files left: [$(PkLeft $dn)]"
  CloseMenus
}

if ($Part -match 'H') {
  Case 'H the hover fill after icons slide under a resting pointer'
  ClearFences | Out-Null
  $hn = @('pk h1 报告.txt', 'pk h2 预算表.txt', 'pk h3 会议记录.txt', 'pk h4 设计稿.txt', 'pk h5 合同扫描件.txt')
  PkFence 'PkH' '300,200,420,170' $hn
  $wr = (FenceGet 'PkH').windowRect
  $cx = @(0..3 | ForEach-Object { (CellPt 'PkH' $_)[0] })
  $cy = (CellPt 'PkH' 0)[1]
  # How much darker the strip just above each of the first four icons is than with the pointer
  # away (the hover fill darkens it by about 6 of 255; the plate itself is not even).
  function PkStrips() { return @(0..3 | ForEach-Object { (MeanRgb ([Px]::Grab($cx[$_] - 15, $cy - 28, 30, 3)))[0] }) }
  ParkMouse; Start-Sleep -Milliseconds 500
  $ref = PkStrips
  function PkFill() { $s = PkStrips; return @(0..3 | ForEach-Object { $ref[$_] - $s[$_] }) }
  [G]::MoveTo($cx[2] - 3, $cy); Start-Sleep -Milliseconds 120; [G]::MoveTo($cx[2], $cy)
  Start-Sleep -Milliseconds 300
  $h0 = PkFill
  Check 'H setup: the hover fill is on the third icon' ($h0[2] -ge 3 -and [math]::Abs($h0[1]) -le 1) "darker by $($h0 -join ',') (strips without hover: $($ref -join ','))"
  PkRec 'H' 280 180 460 210
  PkMark '鼠标停在第三个图标上，第一个文件被删除'
  Remove-Item -LiteralPath (Join-Path $PfDesk $hn[0]) -Force
  Start-Sleep -Milliseconds 1200
  $h1 = PkFill
  PkMark '图标滑动之后，鼠标一直没动'
  Start-Sleep -Milliseconds 1800
  $h2 = PkFill
  PkStop
  Check 'H after the slide the fill is on the icon under the pointer' ($h1[2] -ge 3 -and [math]::Abs($h1[1]) -le 1 -and $h2[2] -ge 3) "darker by $($h1 -join ',') after 1.2 s, $($h2 -join ',') after 3 s (before the slide: $($h0 -join ','))"
  ParkMouse
}

if ($Part -match 'F') {
  Case 'F F2 with several icons selected renames the focused one'
  ClearFences | Out-Null
  $fn = @('pk f1 草稿.txt', 'pk f2 清单.txt', 'pk f3 备忘.txt')
  PkFence 'PkF' '300,200,420,170' $fn
  $p = CellPt 'PkF' 1
  PkRec 'F' 280 180 460 210
  PkMark '点第二个文件，再按 Ctrl+A 全选'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600
  ParkMouse
  Chord 0x11 0x41; Start-Sleep -Milliseconds 600
  $w = DumpOf 'PkF'
  Check 'F setup: three selected, the focus on the second' ((PkSelOf $w) -eq '3' -and (PkFocusOf $w) -like 'pk f2*') "selected=$(PkSelOf $w) focused=[$(PkFocusOf $w)]"
  PkMark '按 F2'
  [G]::Press(0x71); Start-Sleep -Milliseconds 300
  $ren = WaitWin 'PecoFence.Rename' 2
  Start-Sleep -Milliseconds 600
  Check 'F F2 opens the name box' ($ren -ne [IntPtr]::Zero) "rename window=$ren"
  if ($ren -ne [IntPtr]::Zero) {
    PkMark '输入新名字，按 Enter'
    [G]::Type('pk f9 改好了'); Start-Sleep -Milliseconds 500
    [G]::Press(0x0D); Start-Sleep -Milliseconds 2000
  }
  $now = (@(Get-ChildItem -LiteralPath $PfDesk | Where-Object { $_.Name -like 'pk f*' } | ForEach-Object { $_.Name } | Sort-Object) -join ' | ')
  Check 'F only the focused file got the new name' ($now -eq 'pk f1 草稿.txt | pk f3 备忘.txt | pk f9 改好了.txt') "files: [$now]"
  PkStop
}

if ($Part -match 'E') {
  Case 'E Esc in a filtered folder fence'
  ClearFences | Out-Null
  New-Item -ItemType Directory -Force $PkDir | Out-Null
  foreach ($n in '报告 一月.txt', '报告 二月.txt', '预算.txt', '合同.txt') { Set-Content -LiteralPath (Join-Path $PkDir $n) -Value 'x' -Encoding ASCII }
  Pf fence create --portal $PkDir --title 'PkE' --rect '300,200,420,170' | Out-Null
  Start-Sleep -Seconds 3
  $p = CellPt 'PkE' 0
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  PkRec 'E' 280 180 460 210
  PkMark '按 Ctrl+F，输入 bg（报告的拼音首字母），按 Enter'
  Chord 0x11 0x46; Start-Sleep -Milliseconds 800
  [G]::Type('bg'); Start-Sleep -Milliseconds 900
  [G]::Press(0x0D); Start-Sleep -Milliseconds 900
  $f = (FenceGet 'PkE').filter
  Check 'E setup: the fence shows the two 报告 files' ($f -and $f.shown -eq 2) ($f | ConvertTo-Json -Compress)
  PkMark '选中一个文件'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600
  ParkMouse
  $w = DumpOf 'PkE'
  $name0 = PkFocusOf $w
  Check 'E setup: one icon of the filtered fence is selected' ((PkSelOf $w) -eq '1' -and (DumpVal $w 'filtered') -eq 'true' -and $name0) "selected=$(PkSelOf $w) filtered=$(DumpVal $w 'filtered') focused=[$name0]"
  PkMark '按 Esc'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 1000
  $w = DumpOf 'PkE'
  Check 'E the first Esc clears the filter and keeps the selection' ((DumpVal $w 'filtered') -eq 'false' -and (PkSelOf $w) -eq '1' -and (PkFocusOf $w) -eq $name0) "filtered=$(DumpVal $w 'filtered') selected=$(PkSelOf $w) focused=[$(PkFocusOf $w)] items=$(DumpVal $w 'items')"
  PkMark '再按 Esc'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 700
  $w = DumpOf 'PkE'
  Check 'E the second Esc clears the selection' ((PkSelOf $w) -eq '0') "selected=$(PkSelOf $w)"
  PkStop
}

if ($Part -match 'V') {
  Case 'V a rename that sorts the icon out of view'
  ClearFences | Out-Null
  $vn = @(1..18 | ForEach-Object { 'pk {0} 文件 {1:D2}.txt' -f [char](96 + $_), $_ })
  PkFence 'PkV' '300,200,520,240' $vn
  $p = CellPt 'PkV' 0
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
  ParkMouse
  $w = DumpOf 'PkV'
  Check 'V setup: the first icon is selected, the fence is at its top' ((PkSelOf $w) -eq '1' -and [double](DumpVal $w 'scroll') -eq 0) "selected=$(PkSelOf $w) scroll=$(DumpVal $w 'scroll')"
  PkRec 'V' 280 180 560 280
  PkMark '按 F2，改成排在最后的名字，按 Enter'
  [G]::Press(0x71); Start-Sleep -Milliseconds 900
  [G]::Type('pk zz 改名后'); Start-Sleep -Milliseconds 500
  [G]::Press(0x0D); Start-Sleep -Milliseconds 2000
  PkMark '改名之后'
  Start-Sleep -Milliseconds 1200
  PkStop
  $w = DumpOf 'PkV'
  $scroll = [double](DumpVal $w 'scroll')
  Check 'V the renamed icon is still selected' ((PkSelOf $w) -eq '1' -and (PkFocusOf $w) -like 'pk zz*') "selected=$(PkSelOf $w) focused=[$(PkFocusOf $w)]"
  # 18 icons in rows of six: the last row starts 200 px down, the fence shows 204 px.
  Check 'V the fence scrolled to the renamed icon' ($scroll -ge 90) "scroll=$scroll"
}

if ($Part -match 'N') {
  # A Delete that removes nothing (No in Windows' question, an icon that cannot be deleted)
  # must not leave the fence waiting: when that icon leaves the fence another way later, its
  # neighbour takes no selection (one more Delete would remove the neighbour).
  # Windows' question of a delete (a dialog of the PecoFence process), zero when none shows.
  function PkQuestion($sec = 6) {
    for ($i = 0; $i -lt $sec * 5; $i++) {
      $d = @([W]::Tops('#32770') | Where-Object { [W]::ProcOf($_) -eq 'pecofence' -and [W]::Title($_) -match 'Delete' }) | Select-Object -First 1
      if ($d) { return $d }
      Start-Sleep -Milliseconds 200
    }
    return [IntPtr]::Zero
  }
  # Answers it with the key given (N or Y; Esc when N did not close it); true when it closed.
  function PkAnswer($d, $vk) {
    if ($d -eq [IntPtr]::Zero) { return $false }
    [G]::Press($vk); Start-Sleep -Milliseconds 800
    if ([G]::IsWindowVisible($d)) { [G]::Press(0x1B); Start-Sleep -Milliseconds 800 }
    return (-not [G]::IsWindowVisible($d))
  }
  # Shift+Delete from the keyboard, or Delete in the item menu with Shift held, on the icon at $p.
  function PkAskDelete($p, $menu) {
    if ($menu) {
      [G]::RClick($p[0], $p[1])
      $m = MenuWin 8
      [G]::KeyDown(0x10); Start-Sleep -Milliseconds 100
      MenuPick '^Delete' $m | Out-Null
      Start-Sleep -Milliseconds 400; [G]::KeyUp(0x10)
    } else {
      [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600
      ParkMouse
      Chord 0x10 0x2E
    }
    return (PkQuestion)
  }
  $nn = @('pk n1 合同.txt', 'pk n2 报价单.txt', 'pk n3 会议记录.txt', 'pk n4 清单.txt', 'pk n5 发票.txt')
  New-Item -ItemType Directory -Force $PkDir | Out-Null
  foreach ($n in $nn) { Set-Content -LiteralPath (Join-Path $PkDir $n) -Value 'x' -Encoding ASCII }
  foreach ($kind in 'fence', 'folder fence') {
    ClearFences | Out-Null
    if ($kind -eq 'fence') { PkFence 'PkN' '300,200,520,170' $nn; $dir = $PfDesk }
    else { Pf fence create --portal $PkDir --title 'PkN' --rect '300,200,520,170' | Out-Null; Start-Sleep -Seconds 3; $dir = $PkDir }
    Pf fence create --title 'PkO' --rect '300,440,300,150' | Out-Null
    Start-Sleep -Milliseconds 800
    foreach ($menu in $false, $true) {
      $how = if ($menu) { 'Delete in the item menu' } else { 'Shift+Delete' }
      Case "N $kind, $how, answered No"
      $q = PkAskDelete (CellPt 'PkN' 1) $menu
      $closed = PkAnswer $q 0x4E
      Start-Sleep -Milliseconds 1200
      ParkMouse
      $w = DumpOf 'PkN'
      $name = PkFocusOf $w
      # The focused icon's file (a name is shown with or without its .txt).
      $file = if ($name) { @((Join-Path $dir $name), (Join-Path $dir "$name.txt")) | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | Select-Object -First 1 } else { $null }
      Check "N $kind, ${how}: Windows asked, No keeps the file and its selection" ($q -ne [IntPtr]::Zero -and $closed -and $file -and (PkSelOf $w) -eq '1') "question=$q closed=$closed focused=[$name] selected=$(PkSelOf $w) file=[$file]"
      # The icon leaves the fence another way: moved to another fence, or removed by a program.
      $left = if ($kind -eq 'fence' -and -not $menu) { 'moved to another fence' } else { 'removed by another program' }
      if ($file -and $left -like 'moved*') { Pf item move $file --to PkO | Out-Null }
      elseif ($file) { Remove-Item -LiteralPath $file -Force }
      Start-Sleep -Milliseconds 2000
      $w = DumpOf 'PkN'
      Check "N $kind, ${how}: the icon is $left later, its neighbour takes no selection" ((PkSelOf $w) -eq '0') "selected=$(PkSelOf $w) focused=[$(PkFocusOf $w)] items=$(DumpVal $w 'items')"
    }
  }

  Case 'N an icon that cannot be deleted'
  # The Recycle Bin in a fence: Delete does nothing; moved to another fence later, the icon
  # beside it takes no selection.
  ClearFences | Out-Null
  PkFence 'PkN' '300,200,520,170' @('pk n6 备忘.txt', 'pk n7 草稿.txt')
  Pf fence create --title 'PkO' --rect '300,440,300,150' | Out-Null
  Pf item move 'Recycle Bin' --to PkN | Out-Null
  Start-Sleep -Seconds 2
  # A fence sorted by name shows the Recycle Bin before the files.
  $p = CellPt 'PkN' 0
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600
  ParkMouse
  [G]::Press(0x2E); Start-Sleep -Milliseconds 1500
  $w = DumpOf 'PkN'
  Check 'N Delete on the Recycle Bin icon does nothing' ((PkSelOf $w) -eq '1' -and (PkFocusOf $w) -eq 'Recycle Bin' -and (PkQuestion 1) -eq [IntPtr]::Zero) "selected=$(PkSelOf $w) focused=[$(PkFocusOf $w)]"
  Pf item move 'Recycle Bin' --to PkO | Out-Null
  Start-Sleep -Milliseconds 2000
  $w = DumpOf 'PkN'
  Check 'N the Recycle Bin icon is moved to another fence later: its neighbour takes no selection' ((PkSelOf $w) -eq '0') "selected=$(PkSelOf $w) focused=[$(PkFocusOf $w)] items=$(DumpVal $w 'items')"

  Case 'N Shift+Delete answered with Yes'
  # The wait survives Windows' question: the icon that followed takes the selection.
  $q = PkAskDelete (CellPt 'PkN' 0) $false
  $closed = PkAnswer $q 0x59
  Start-Sleep -Milliseconds 2500
  $w = DumpOf 'PkN'
  Check 'N answered Yes: the file is gone and the icon that followed is selected' ($q -ne [IntPtr]::Zero -and $closed -and -not (Test-Path -LiteralPath (Join-Path $PfDesk 'pk n6 备忘.txt')) -and (PkSelOf $w) -eq '1' -and (PkFocusOf $w) -like 'pk n7*') "question=$q closed=$closed selected=$(PkSelOf $w) focused=[$(PkFocusOf $w)]"
}

Case 'cleanup'
PfQuit
PkClean
[Anim]::Set($AnimWas)
