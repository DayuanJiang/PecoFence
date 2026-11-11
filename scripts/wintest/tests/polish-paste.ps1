# Fence polish 2026-11-12, package paste (reviews: interaction UX2 and UX8, code C8).
# Part A: what is pasted into a fence lands in that fence and is selected there, as Explorer
#   puts a copy beside its original and selects it: a copy made beside its original (Ctrl+V in
#   the same fence, in another fence, the menu's 粘贴, with a rule that would file it elsewhere,
#   two files at once), a cut, a file from another folder, a folder fence.
# Part B: the keyboard after the name box over a fence's title closes (Enter, Esc, a new fence,
#   the menu's 重命名, a click on another window) and after 删除栅栏 from the fence menu; the
#   command line takes the keyboard from nobody.
# Part C: the selection of pasted files gives way to the user: a file the fence's filter hid is
#   not selected later over another selection, or long after the paste.
# Part D: of two pasted files that arrive one after the other, the second leaves a click on
#   another icon in between alone (copies a 6 GB file: 12 GB free on the box). The same for an
#   open name box is a unit test (app/tests.rs): the copy progress window keeps the keyboard.
# Parts: run.sh --var Part=A, B, C (about 2 min each) or D (about 2 min); default all.
if (-not $Part) { $Part = 'ABCD' }
trap { PfTrap $_; continue }

function PpClean() {
  Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue | Where-Object { $_.Name -like 'pp *' } |
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
  Remove-Item -Recurse -Force "$PfRun\pp-src", "$PfRun\pp-folder", "$PfRun\pp-filter" -ErrorAction SilentlyContinue
}
function CtrlKey($vk, $ms = 500) { Chord 0x11 $vk; Start-Sleep -Milliseconds $ms }
# What the fence's dump line says about the selection: how many icons, and the keyboard cursor.
function PpSel($t) {
  $d = DumpOf $t 3
  $f = if ($d -and $d.raw -match ' focused=Some\("((?:[^"\\]|\\.)*)"\)') { $Matches[1] } else { '' }
  return @{ n = [int](DumpVal $d 'selected'); focus = $f; text = "selected=$(DumpVal $d 'selected') focused='$f'" }
}
# File names the fence's selection puts on the clipboard with Ctrl+C (which icons are selected).
function PpCopied() {
  [Windows.Forms.Clipboard]::Clear()
  CtrlKey 0x43 600
  return ((@(ClipFiles) | ForEach-Object { Split-Path $_ -Leaf } | Sort-Object) -join ' | ')
}
# Screen point of a file's icon: its place in the order the fence shows.
function PpPt($fence, $file) {
  $order = @(@(PfJson item list --fence $fence) | Where-Object { $_ -and $_.fileName } | ForEach-Object { $_.fileName })
  $i = [array]::IndexOf($order, $file)
  if ($i -lt 0) { Log "PpPt: '$file' is not in '$fence' ($($order -join ', '))"; return $null }
  return (CellPt $fence $i)
}
function PpClickItem($fence, $file) { $p = PpPt $fence $file; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 400 }
# A click on the empty lower part of a fence: it takes the keyboard and selects nothing.
function PpClickEmpty($fence) { $w = (FenceGet $fence).windowRect; [G]::Click($w.x + $w.w - 30, $w.y + $w.h - 24); Start-Sleep -Milliseconds 400 }
# Desktop entries named like a copy of $stem that were not there before.
function PpNewCopies($stem, $before) {
  return @(Get-ChildItem -LiteralPath $PfDesk | Where-Object { $_.Name -like "$stem *" -and $before -notcontains $_.Name } | ForEach-Object { $_.Name })
}
function PpWaitCopies($stem, $before, $count = 1, $sec = 8) {
  for ($k = 0; $k -lt $sec * 4; $k++) { $c = @(PpNewCopies $stem $before); if ($c.Count -ge $count) { return $c }; Start-Sleep -Milliseconds 250 }
  return @(PpNewCopies $stem $before)
}
function PpDeskNames() { return @(Get-ChildItem -LiteralPath $PfDesk | ForEach-Object { $_.Name }) }
function PpDisplay($file) { return (@(PfJson item list) | Where-Object { $_ -and $_.fileName -eq $file } | Select-Object -First 1).name }
# The copy landed in $fence and is the one selected icon there.
function PpCheckCopy($tag, $copy, $fence) {
  Check "$tag the copy is in the fence it was pasted into" ((WaitItemFence $copy $fence 6) -eq $fence) "copy '$copy' is in '$(ItemFence $copy)', wanted '$fence'"
  Start-Sleep -Milliseconds 600
  $s = PpSel $fence
  Check "$tag the copy is selected" ($s.n -eq 1 -and $s.focus -eq (PpDisplay $copy)) "$($s.text) in '$fence', copy shows as '$(PpDisplay $copy)'"
  $c = PpCopied
  Check "$tag Ctrl+C right after the paste takes the copy" ($c -eq $copy) "clipboard: '$c'"
}
function PpTitles() { return ((@(PfJson fence list) | Where-Object { $_ -and $_.id -and $_.kind -ne 'inbox' } | ForEach-Object { $_.title }) -join ' | ') }
# The fence nearest to a rect's centre among the fences left (「桌面」 only when no other is left).
function PpNearest($rect) {
  $cx = $rect.x + $rect.w / 2; $cy = $rect.y + $rect.h / 2
  $best = $null; $bd = [double]::MaxValue
  $all = @(@(PfJson fence list) | Where-Object { $_ -and $_.id -and $_.windowRect })
  $plates = @($all | Where-Object { $_.kind -ne 'inbox' })
  if ($plates.Count -gt 0) { $all = $plates }
  foreach ($f in $all) {
    $w = $f.windowRect
    $d = [math]::Pow($w.x + $w.w / 2 - $cx, 2) + [math]::Pow($w.y + $w.h / 2 - $cy, 2)
    if ($d -lt $bd) { $bd = $d; $best = $f.title }
  }
  return $best
}

Case 'setup'
CloseWindows
PpClean
$PpFiles = @('pp 报告.txt', 'pp 预算表 2026.txt', 'pp 会议纪要 第三季度.txt', 'pp 照片说明.md', 'pp 合同.txt')
foreach ($n in $PpFiles) { DeskFile $n | Out-Null }
DeskFile 'pp 发票.txt' | Out-Null; DeskFile 'pp 行程单.txt' | Out-Null
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
PfStart -Script (DumpScript 300 14) -Lang 'zh-CN'
Start-Sleep -Seconds 3
ClearFences | Out-Null
ClearPfNotices
Pf fence create --title Alpha --rect '300,120,280,300' | Out-Null
Pf fence create --title Bravo --rect '700,120,280,300' | Out-Null
Pf item move @($PpFiles | ForEach-Object { Join-Path $PfDesk $_ }) --to Alpha | Out-Null
Pf item move (Join-Path $PfDesk 'pp 发票.txt') (Join-Path $PfDesk 'pp 行程单.txt') --to Bravo | Out-Null
Start-Sleep -Seconds 2
$Inbox = (@(PfJson fence list) | Where-Object { $_ -and $_.kind -eq 'inbox' } | Select-Object -First 1).title
Log "fences: $(PpTitles); inbox '$Inbox'"

if ($Part -match 'A') {
  Case 'A1 a copy pasted into the same fence'
  $before = PpDeskNames
  PpClickItem 'Alpha' 'pp 报告.txt'
  CtrlKey 0x43; CtrlKey 0x56 300
  $copy = @(PpWaitCopies 'pp 报告' $before)[0]
  Log "A1 copy: '$copy'"
  Shot 'A1-pasted-same-fence'
  PpCheckCopy 'A1' $copy 'Alpha'

  Case 'A2 a copy pasted into another fence'
  $before = PpDeskNames
  PpClickItem 'Alpha' 'pp 预算表 2026.txt'
  CtrlKey 0x43
  PpClickEmpty 'Bravo'
  CtrlKey 0x56 300
  $copy = @(PpWaitCopies 'pp 预算表 2026' $before)[0]
  Log "A2 copy: '$copy'"
  Shot 'A2-pasted-other-fence'
  PpCheckCopy 'A2' $copy 'Bravo'

  Case 'A3 a rule that would file the copy elsewhere'
  Pf rule add --name 'pp txt to Alpha' --ext txt --to Alpha --index 0 | Out-Null
  Start-Sleep -Milliseconds 800
  $before = PpDeskNames
  PpClickItem 'Alpha' 'pp 合同.txt'
  CtrlKey 0x43
  PpClickEmpty 'Bravo'
  CtrlKey 0x56 300
  $copy = @(PpWaitCopies 'pp 合同' $before)[0]
  Log "A3 copy: '$copy'"
  PpCheckCopy 'A3' $copy 'Bravo'
  Pf rule remove 'pp txt to Alpha' | Out-Null
  Start-Sleep -Milliseconds 500

  Case 'A4 the menu''s paste'
  $before = PpDeskNames
  PpClickItem 'Alpha' 'pp 照片说明.md'
  CtrlKey 0x43
  $w = (FenceGet 'Bravo').windowRect
  [G]::RClick($w.x + $w.w - 30, $w.y + $w.h - 24)
  $m = MenuWin 5
  Log "A4 menu: $(MenuTexts $m)"
  MenuPick '^粘贴' $m | Out-Null
  $copy = @(PpWaitCopies 'pp 照片说明' $before)[0]
  Log "A4 copy: '$copy'"
  PpCheckCopy 'A4' $copy 'Bravo'

  Case 'A5 two files copied at once'
  $before = PpDeskNames
  PpClickItem 'Bravo' 'pp 发票.txt'
  $p = PpPt 'Bravo' 'pp 行程单.txt'
  [G]::KeyDown(0x11); [G]::Click($p[0], $p[1]); [G]::KeyUp(0x11); Start-Sleep -Milliseconds 400
  CtrlKey 0x43
  PpClickEmpty 'Alpha'
  CtrlKey 0x56 300
  $c1 = @(PpWaitCopies 'pp 发票' $before)[0]; $c2 = @(PpWaitCopies 'pp 行程单' $before)[0]
  Log "A5 copies: '$c1', '$c2'"
  Check 'A5 both copies are in the fence pasted into' ((WaitItemFence $c1 'Alpha' 6) -eq 'Alpha' -and (WaitItemFence $c2 'Alpha' 6) -eq 'Alpha') "'$c1' in '$(ItemFence $c1)', '$c2' in '$(ItemFence $c2)'"
  Start-Sleep -Milliseconds 600
  $s = PpSel 'Alpha'
  Check 'A5 both copies are selected' ($s.n -eq 2) $s.text
  $c = PpCopied
  Check 'A5 Ctrl+C right after the paste takes both copies' ($c -eq ((@($c1, $c2) | Sort-Object) -join ' | ')) "clipboard: '$c'"
  Shot 'A5-two-copies'

  Case 'A6 cut and paste'
  PpClickItem 'Alpha' 'pp 会议纪要 第三季度.txt'
  CtrlKey 0x58
  PpClickEmpty 'Bravo'
  CtrlKey 0x56 900
  Check 'A6 the cut file is in the fence pasted into' ((WaitItemFence 'pp 会议纪要 第三季度.txt' 'Bravo' 6) -eq 'Bravo') "in '$(ItemFence 'pp 会议纪要 第三季度.txt')'"
  $s = PpSel 'Bravo'
  Check 'A6 the moved icon is selected' ($s.n -eq 1 -and $s.focus -eq (PpDisplay 'pp 会议纪要 第三季度.txt')) $s.text
  $c = PpCopied
  Check 'A6 Ctrl+C right after the paste takes the moved file' ($c -eq 'pp 会议纪要 第三季度.txt') "clipboard: '$c'"

  Case 'A7 a file copied in another folder'
  New-Item -ItemType Directory -Force "$PfRun\pp-src" | Out-Null
  Set-Content -LiteralPath "$PfRun\pp-src\pp 外来文件.txt" -Value 'from another folder' -Encoding ASCII
  $list = New-Object System.Collections.Specialized.StringCollection
  $list.Add("$PfRun\pp-src\pp 外来文件.txt") | Out-Null
  [Windows.Forms.Clipboard]::SetFileDropList($list)
  PpClickEmpty 'Bravo'
  CtrlKey 0x56 900
  Check 'A7 the file is copied to the desktop and shown in the fence pasted into' ((WaitItemFence 'pp 外来文件.txt' 'Bravo' 8) -eq 'Bravo') "in '$(ItemFence 'pp 外来文件.txt')'; on the desktop: $(Test-Path -LiteralPath "$PfDesk\pp 外来文件.txt")"
  Start-Sleep -Milliseconds 800
  $s = PpSel 'Bravo'
  Check 'A7 it is selected' ($s.n -eq 1 -and $s.focus -eq (PpDisplay 'pp 外来文件.txt')) $s.text
  Check 'A7 the original stays in its folder' (Test-Path -LiteralPath "$PfRun\pp-src\pp 外来文件.txt") ''

  Case 'A9 a copy that lands below the rows in view'
  # Bravo one row high: its seven icons take three rows, the copy goes after them.
  Pf fence move Bravo --rect '700,120,280,150' | Out-Null
  Start-Sleep -Milliseconds 900
  $before = PpDeskNames
  PpClickItem 'Bravo' 'pp 发票.txt'
  CtrlKey 0x43; CtrlKey 0x56 300
  $copy = @(PpWaitCopies 'pp 发票' $before)[0]
  Check 'A9 the copy is in the fence it was pasted into' ((WaitItemFence $copy 'Bravo' 6) -eq 'Bravo') "copy '$copy' is in '$(ItemFence $copy)'"
  Start-Sleep -Milliseconds 1200
  $d = DumpOf 'Bravo' 3
  $s = PpSel 'Bravo'
  Shot 'A9-scrolled-to-copy'
  Check 'A9 the fence scrolls to the copy and selects it' ([double](DumpVal $d 'scroll') -gt 0 -and $s.n -eq 1 -and $s.focus -eq (PpDisplay $copy)) "scroll=$(DumpVal $d 'scroll') $($s.text); items: $(@(@(PfJson item list --fence Bravo) | ForEach-Object { $_.name }) -join ', ')"
  Pf fence move Bravo --rect '700,120,280,300' | Out-Null
  Start-Sleep -Milliseconds 600

  Case 'A8 a copy pasted in a folder fence'
  New-Item -ItemType Directory -Force "$PfRun\pp-folder" | Out-Null
  foreach ($n in 'pf 清单.txt', 'pf 说明.txt') { Set-Content -LiteralPath "$PfRun\pp-folder\$n" -Value $n -Encoding ASCII }
  Pf fence create --portal "$PfRun\pp-folder" --title 'PpFolder' --rect '300,460,280,220' | Out-Null
  Start-Sleep -Seconds 2
  $p = CellPt 'PpFolder' 0; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 400
  $first = (PpSel 'PpFolder').focus
  CtrlKey 0x43; CtrlKey 0x56 300
  $made = @()
  for ($k = 0; $k -lt 32 -and $made.Count -eq 0; $k++) { $made = @(Get-ChildItem -LiteralPath "$PfRun\pp-folder" | Where-Object { $_.Name -notin 'pf 清单.txt', 'pf 说明.txt' } | ForEach-Object { $_.Name }); Start-Sleep -Milliseconds 250 }
  Start-Sleep -Milliseconds 1200
  $s = PpSel 'PpFolder'
  Log "A8 first icon '$first', copy in the folder: '$($made -join ', ')'"
  Check 'A8 the copy is made in the folder and selected in its fence' ($made.Count -eq 1 -and $s.n -eq 1 -and $s.focus -and $s.focus -ne $first) "$($s.text); new in the folder: $($made -join ', ')"
  $c = PpCopied
  Check 'A8 Ctrl+C right after the paste takes the copy' ($made.Count -eq 1 -and $c -eq $made[0]) "clipboard: '$c'"
  Shot 'A8-folder-fence'
  Pf fence delete 'PpFolder' | Out-Null
  Start-Sleep -Milliseconds 800
}

if ($Part -match 'B') {
  # A Notepad window to click on and to hold the keyboard.
  $np = Start-Process notepad -PassThru
  Start-Sleep -Seconds 2
  $nph = [IntPtr]::Zero
  for ($k = 0; $k -lt 20 -and $nph -eq [IntPtr]::Zero; $k++) { $nph = (Get-Process -Id $np.Id -ErrorAction SilentlyContinue).MainWindowHandle; Start-Sleep -Milliseconds 300 }
  if ($nph -eq [IntPtr]::Zero) { $nph = (Get-Process notepad | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1).MainWindowHandle }
  [L]::MoveWindow($nph, 1050, 60, 480, 300, $true) | Out-Null
  Start-Sleep -Milliseconds 500
  function PpMarquee() {
    $r = FreeRect 320 240 1040 420
    [G]::MoveTo($r[0], $r[1]); Start-Sleep -Milliseconds 200; MDown; Start-Sleep -Milliseconds 200
    Glide $r[0] $r[1] $r[2] $r[3] 14 30
    Start-Sleep -Milliseconds 300; MUp; Start-Sleep -Milliseconds 700
    $off = [G]::FindWindowW('PecoFence.MarqueeOffer', [NullString]::Value)
    return (Rect $off)
  }
  function PpNewFenceByMouse() {
    $or = PpMarquee
    [G]::Click($or.R - 40, [int](($or.T + $or.B) / 2))
    return (WaitWin 'PecoFence.Rename' 6)
  }

  Case 'B1 a new fence, named with Enter'
  $box = PpNewFenceByMouse
  Check 'B1 the name box opens on the new fence' ($box -ne [IntPtr]::Zero) "box=$box fg=$(Fg)"
  [G]::Type('新栅栏甲'); Start-Sleep -Milliseconds 300; [G]::Press(0x0D); Start-Sleep -Milliseconds 900
  $fg = Fg
  Shot 'B1-named'
  Check 'B1 the new fence has the keyboard after Enter' ($fg -eq "PecoFence.Fence '新栅栏甲'") "foreground: $fg"
  # The next key goes to it: a file on the clipboard is pasted into the new fence.
  New-Item -ItemType Directory -Force "$PfRun\pp-src" | Out-Null
  Set-Content -LiteralPath "$PfRun\pp-src\pp 新栅栏里的文件.txt" -Value 'pasted' -Encoding ASCII
  $list = New-Object System.Collections.Specialized.StringCollection
  $list.Add("$PfRun\pp-src\pp 新栅栏里的文件.txt") | Out-Null
  [Windows.Forms.Clipboard]::SetFileDropList($list)
  CtrlKey 0x56 900
  Check 'B1 Ctrl+V right after naming pastes into the new fence' ((WaitItemFence 'pp 新栅栏里的文件.txt' '新栅栏甲' 6) -eq '新栅栏甲') "the file is in '$(ItemFence 'pp 新栅栏里的文件.txt')'; foreground: $(Fg)"

  Case 'B2 a new fence, Esc at the name'
  $had = @(PpTitles) -split ' \| '
  $box = PpNewFenceByMouse
  [G]::Press(0x1B); Start-Sleep -Milliseconds 900
  $new = @((PpTitles) -split ' \| ' | Where-Object { $had -notcontains $_ })[0]
  $fg = Fg
  Check 'B2 the new fence has the keyboard after Esc' ($new -and $fg -eq "PecoFence.Fence '$new'") "foreground: $fg; new fence '$new'"

  Case 'B3 the menu''s rename'
  foreach ($key in @(@(0x0D, 'Enter'), @(0x1B, 'Esc'))) {
    [G]::Click(1290, 200); Start-Sleep -Milliseconds 500   # Notepad has the keyboard first
    $w = (FenceGet 'Bravo').windowRect
    [G]::RClick($w.x + 120, $w.y + 18)
    $m = MenuWin 5
    MenuPick '^重命名' $m | Out-Null
    $box = WaitWin 'PecoFence.Rename' 6
    Start-Sleep -Milliseconds 300
    [G]::Press($key[0]); Start-Sleep -Milliseconds 900
    $fg = Fg
    Check "B3 Bravo has the keyboard after $($key[1]) in its name box" ($box -ne [IntPtr]::Zero -and $fg -eq "PecoFence.Fence 'Bravo'") "foreground: $fg; box=$box"
  }

  Case 'B4 the name box closed by a click on another window'
  $w = (FenceGet 'Bravo').windowRect
  [G]::RClick($w.x + 120, $w.y + 18)
  $m = MenuWin 5
  MenuPick '^重命名' $m | Out-Null
  $box = WaitWin 'PecoFence.Rename' 6
  [G]::Type('Bravo2'); Start-Sleep -Milliseconds 300
  [G]::Click(1290, 200); Start-Sleep -Milliseconds 900
  $fg = Fg
  Check 'B4 the clicked window keeps the keyboard and the name is taken' ($fg -match '^Notepad' -and (PfFence 'Bravo2')) "foreground: $fg; fences: $(PpTitles)"
  Pf fence rename 'Bravo2' 'Bravo' | Out-Null
  Start-Sleep -Milliseconds 500

  Case 'B5 an empty fence deleted from its menu'
  $t = '新栅栏甲'
  Pf item move "$PfDesk\pp 新栅栏里的文件.txt" --to Alpha | Out-Null
  Start-Sleep -Milliseconds 600
  $gone = (FenceGet $t).windowRect
  [G]::RClick($gone.x + 100, $gone.y + 18)
  $m = MenuWin 5
  MenuPick '^删除栅栏' $m | Out-Null
  Start-Sleep -Milliseconds 1200
  $near = PpNearest $gone
  $fg = Fg
  Shot 'B5-deleted-empty'
  Check 'B5 the nearest fence has the keyboard after the delete' ($fg -eq "PecoFence.Fence '$near'") "foreground: $fg; nearest fence '$near'; fences: $(PpTitles)"
  CtrlKey 0x5A 1200
  Check 'B5 Ctrl+Z right after brings the fence back' ([bool](PfFence $t)) "fences: $(PpTitles)"

  Case 'B6 a fence with icons deleted from its menu'
  $gone = (FenceGet 'Bravo').windowRect
  [G]::RClick($gone.x + 120, $gone.y + 18)
  $m = MenuWin 5
  MenuPick '^删除栅栏' $m | Out-Null
  Start-Sleep -Milliseconds 1000
  [G]::Press(0x59); Start-Sleep -Milliseconds 1300   # Y in the question
  $near = PpNearest $gone
  $fg = Fg
  Shot 'B6-deleted-with-icons'
  Check 'B6 the nearest fence has the keyboard after the delete' (-not (PfFence 'Bravo') -and $fg -eq "PecoFence.Fence '$near'") "foreground: $fg; nearest fence '$near'; fences: $(PpTitles)"
  CtrlKey 0x5A 1500
  Check 'B6 Ctrl+Z right after brings the fence and its icons back' ((PfFence 'Bravo') -and (ItemFence 'pp 发票.txt') -eq 'Bravo') "fences: $(PpTitles); pp 发票.txt is in '$(ItemFence 'pp 发票.txt')'"

  Case 'B8 a name box over a fence that another window covers'
  # As when a fence is made from the tray menu while a program covers the desktop: the name box
  # floats over that program, and the fence under it must not come up over it.
  $had = @(@(PfJson fence list) | Where-Object { $_ -and $_.id } | ForEach-Object { "$($_.id)" })
  $box = PpNewFenceByMouse
  $w = (@(PfJson fence list) | Where-Object { $_ -and $_.id -and $had -notcontains "$($_.id)" } | Select-Object -First 1).windowRect
  [L]::MoveWindow($nph, $w.x - 40, $w.y - 40, $w.w + 80, $w.h + 80, $true) | Out-Null
  Start-Sleep -Milliseconds 600
  $fgBox = Fg
  [G]::Type('被盖住的栅栏'); Start-Sleep -Milliseconds 300; [G]::Press(0x0D); Start-Sleep -Milliseconds 900
  $fg = Fg
  $top = [W]::RootClassAt($w.x + [int]($w.w / 2), $w.y + [int]($w.h / 2))
  Shot 'B8-covered'
  Check 'B8 the covering window stays on top and the fence does not take the keyboard' ($fgBox -match '^PecoFence\.Rename' -and [bool](PfFence '被盖住的栅栏') -and $fg -notmatch '^PecoFence\.Fence' -and $top -eq 'Notepad') "name box up: $fgBox; after Enter: foreground $fg, window at the fence's centre: $top; fences: $(PpTitles)"
  [L]::MoveWindow($nph, 1050, 60, 480, 300, $true) | Out-Null
  Start-Sleep -Milliseconds 500

  Case 'B7 the command line leaves the keyboard where it is'
  [G]::Click(1290, 200); Start-Sleep -Milliseconds 600
  $fg0 = Fg
  Pf fence create --title 'PpCli' --rect '300,460,240,200' | Out-Null; Start-Sleep -Milliseconds 700
  $fg1 = Fg
  Pf fence rename 'PpCli' 'PpCli2' | Out-Null; Start-Sleep -Milliseconds 700
  $fg2 = Fg
  Pf fence delete 'PpCli2' | Out-Null; Start-Sleep -Milliseconds 900
  $fg3 = Fg
  Check 'B7 create, rename and delete from the command line leave Notepad in front' ($fg0 -match '^Notepad' -and $fg1 -eq $fg0 -and $fg2 -eq $fg0 -and $fg3 -eq $fg0) "before: $fg0; after create: $fg1; after rename: $fg2; after delete: $fg3"
  Stop-Process -Id $np.Id -Force -ErrorAction SilentlyContinue
}

# The clipboard holds these files, in this order (as copied in another folder).
function PpClip([string[]]$paths) {
  $list = New-Object System.Collections.Specialized.StringCollection
  foreach ($p in $paths) { $list.Add($p) | Out-Null }
  [Windows.Forms.Clipboard]::SetFileDropList($list)
}
# A file of $gb GB that takes the copy engine several seconds (no data is written to make it;
# 2 GB took under a second on the test boxes).
function PpBigFile($path, $gb = 6) { $f = [IO.File]::Create($path); $f.SetLength([int64]$gb * 1GB); $f.Close() }
function PpWaitSel($fence, $like, $sec = 12) {
  for ($k = 0; $k -lt $sec * 4; $k++) { $s = PpSel $fence; if ($s.n -eq 1 -and $s.focus -like $like) { return $s }; Start-Sleep -Milliseconds 250 }
  return (PpSel $fence)
}

if ($Part -match 'C') {
  New-Item -ItemType Directory -Force "$PfRun\pp-src", "$PfRun\pp-filter" | Out-Null
  foreach ($n in 'pf 报告 一月.txt', 'pf 报告 二月.txt', 'pf 预算.txt') { Set-Content -LiteralPath "$PfRun\pp-filter\$n" -Value $n -Encoding ASCII }
  foreach ($n in 'pp 合同 原件.txt', 'pp 发票 原件.txt') { Set-Content -LiteralPath "$PfRun\pp-src\$n" -Value $n -Encoding ASCII }
  Pf fence create --portal "$PfRun\pp-filter" --title 'PpFilter' --rect '300,460,300,220' | Out-Null
  Start-Sleep -Seconds 2
  # The folder fence shows only the 报告 files (Ctrl+F, the text, Enter keeps the filter).
  function PpFilterOn() {
    $p = CellPt 'PpFilter' 0; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 400
    Chord 0x11 0x46; Start-Sleep -Milliseconds 800
    [G]::Type('报告'); Start-Sleep -Milliseconds 900
    [G]::Press(0x0D); Start-Sleep -Milliseconds 900
    return (FenceGet 'PpFilter').filter
  }

  Case 'C1 a pasted file the filter hides, then another icon selected'
  $f = PpFilterOn
  PpClip @("$PfRun\pp-src\pp 合同 原件.txt")
  PpClickEmpty 'PpFilter'
  CtrlKey 0x56 1500
  $f2 = (FenceGet 'PpFilter').filter
  Check 'C1 setup: the file is pasted into the folder and the filter hides it' ($f.shown -eq 2 -and $f2.shown -eq 2 -and (Test-Path -LiteralPath "$PfRun\pp-filter\pp 合同 原件.txt")) "filter before: $($f | ConvertTo-Json -Compress); after the paste: $($f2 | ConvertTo-Json -Compress)"
  $p = CellPt 'PpFilter' 0; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  $mine = (PpSel 'PpFilter').focus
  [G]::Press(0x1B); Start-Sleep -Milliseconds 900   # the first Esc clears the filter and keeps the selection
  # The folder changes: the pasted file shows by now, and the user's selection must stay.
  Set-Content -LiteralPath "$PfRun\pp-filter\pf 新来的.txt" -Value 'new' -Encoding ASCII
  Start-Sleep -Milliseconds 2000
  $s = PpSel 'PpFilter'
  $shown = [int](DumpVal (DumpOf 'PpFilter' 3) 'items')
  Shot 'C1-own-selection-kept'
  Check 'C1 the icon selected after the paste stays selected when the folder changes' ($mine -and $s.n -eq 1 -and $s.focus -eq $mine -and $shown -ge 5) "selected '$mine' before; now $($s.text); icons shown: $shown"

  Case 'C2 a pasted file the filter hides, shown a long time later'
  $f = PpFilterOn
  PpClip @("$PfRun\pp-src\pp 发票 原件.txt")
  PpClickEmpty 'PpFilter'
  CtrlKey 0x56 1500
  $s0 = PpSel 'PpFilter'
  Check 'C2 setup: pasted, hidden by the filter, nothing selected' ((Test-Path -LiteralPath "$PfRun\pp-filter\pp 发票 原件.txt") -and $s0.n -eq 0) "$($s0.text); filter: $((FenceGet 'PpFilter').filter | ConvertTo-Json -Compress)"
  Start-Sleep -Seconds 17   # longer than a paste counts as just done (15 s)
  [G]::Press(0x1B); Start-Sleep -Milliseconds 900
  Set-Content -LiteralPath "$PfRun\pp-filter\pf 更晚来的.txt" -Value 'later' -Encoding ASCII
  Start-Sleep -Milliseconds 2000
  $s = PpSel 'PpFilter'
  Check 'C2 a folder change long after the paste selects nothing' ($s.n -eq 0) "$($s.text); filter: $((FenceGet 'PpFilter').filter | ConvertTo-Json -Compress)"
  Pf fence delete 'PpFilter' | Out-Null
}

if ($Part -match 'D') {
  New-Item -ItemType Directory -Force "$PfRun\pp-src" | Out-Null
  Set-Content -LiteralPath "$PfRun\pp-src\pp 小文件.txt" -Value 'small' -Encoding ASCII
  foreach ($n in 'pp 清单.txt', 'pp 名单.txt') { DeskFile $n | Out-Null }
  Start-Sleep -Seconds 2
  # The second file of a paste is on the desktop (the copy of the first is over by then). The
  # command line is left alone while the big file is copied.
  function PpWaitFile($name, $sec = 150) {
    for ($k = 0; $k -lt $sec * 4; $k++) { if (Test-Path -LiteralPath "$PfDesk\$name") { return $true }; Start-Sleep -Milliseconds 250 }
    return $false
  }

  Case 'D1 two pasted files arrive one after the other, another icon is clicked in between'
  Pf fence create --title 'PpLate' --rect '300,460,300,300' | Out-Null
  Pf item move (Join-Path $PfDesk 'pp 清单.txt') (Join-Path $PfDesk 'pp 名单.txt') --to PpLate | Out-Null
  Start-Sleep -Seconds 2
  # The big file is copied first and shows as soon as its copy starts; the small one follows
  # when that copy is done.
  PpBigFile "$PfRun\pp-src\pp 大文件.bin"
  PpClip @("$PfRun\pp-src\pp 大文件.bin", "$PfRun\pp-src\pp 小文件.txt")
  $other = PpPt 'PpLate' 'pp 清单.txt'
  PpClickEmpty 'PpLate'
  CtrlKey 0x56 300
  $s1 = PpWaitSel 'PpLate' 'pp 大文件*'
  $early = Test-Path -LiteralPath "$PfDesk\pp 小文件.txt"
  [G]::Click($other[0], $other[1]); Start-Sleep -Milliseconds 500
  $mine = PpSel 'PpLate'
  Check 'D1 setup: the first file is selected, the second is still to come, then another icon is clicked' ($s1.n -eq 1 -and $s1.focus -like 'pp 大文件*' -and -not $early -and $mine.n -eq 1 -and $mine.focus -like 'pp 清单*') "first arrival: $($s1.text); second already there: $early; after the click: $($mine.text)"
  $came = PpWaitFile 'pp 小文件.txt'
  Start-Sleep -Milliseconds 2500
  $s = PpSel 'PpLate'
  $in = WaitItemFence 'pp 小文件.txt' 'PpLate' 8
  Shot 'D1-click-kept'
  Check 'D1 the clicked icon stays the only one selected when the second file arrives' ($came -and $in -eq 'PpLate' -and $s.n -eq 1 -and $s.focus -like 'pp 清单*') "second file is in '$in'; $($s.text)"
}

Case 'cleanup'
# A name box a failed step left open is closed first.
if ([G]::FindWindowW('PecoFence.Rename', [NullString]::Value) -ne [IntPtr]::Zero) { [G]::Press(0x1B); Start-Sleep -Milliseconds 500 }
PfQuit
[Anim]::Set($AnimWas)
[Windows.Forms.Clipboard]::Clear()
PpClean
