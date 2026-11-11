# Fence polish 2026-11-12, package paste (reviews: interaction UX2, code C8).
# Part A: what is pasted into a fence lands in that fence and is selected there, as Explorer
#   puts a copy beside its original and selects it: a copy made beside its original (Ctrl+V in
#   the same fence, in another fence, the menu's 粘贴, with a rule that would file it elsewhere,
#   two files at once), a cut, a file from another folder, a folder fence.
if (-not $Part) { $Part = 'A' }
trap { PfTrap $_; continue }

function PpClean() {
  Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue | Where-Object { $_.Name -like 'pp *' } |
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
  Remove-Item -Recurse -Force "$PfRun\pp-src", "$PfRun\pp-folder" -ErrorAction SilentlyContinue
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

Case 'cleanup'
# A name box a failed step left open is closed first.
if ([G]::FindWindowW('PecoFence.Rename', [NullString]::Value) -ne [IntPtr]::Zero) { [G]::Press(0x1B); Start-Sleep -Milliseconds 500 }
PfQuit
[Anim]::Set($AnimWas)
[Windows.Forms.Clipboard]::Clear()
PpClean
