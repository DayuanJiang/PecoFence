# Fence polish round 2026-11-12 (interaction review UX1): Ctrl+Z in a fence undoes the newest
# thing the user did there. A rename, a Delete, a drop into a folder, a pasted copy and a new
# folder are undone (the rename and the new folder by PecoFence, the others by Windows' own
# undo, which PecoFence asks for); a Shift+Delete stops Ctrl+Z instead of letting it undo an
# older layout step; fence moves and icon moves made before stay undoable, newest first.
# Parts (run.sh --var Part=...): A rename, delete, drop into a folder, paste; B new folder,
# permanent delete, the command line's file moves. Default: both.
if (-not $Part) { $Part = 'AB' }
trap { PfTrap $_; continue }

function UFiles() { return @(Get-ChildItem $PfDesk -Filter 'u? *' -ErrorAction SilentlyContinue) }
function UClean() {
  UFiles | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
  foreach ($n in 'New folder', 'New folder (2)') { if (Test-Path -LiteralPath "$PfDesk\$n") { Remove-Item -LiteralPath "$PfDesk\$n" -Recurse -Force -ErrorAction SilentlyContinue } }
}
function FRect($t) { $f = PfFence $t; if ($f) { $w = $f.windowRect; return "$($w.x),$($w.y),$($w.w),$($w.h)" }; return 'gone' }
function In($t) { return (@(PfJson item list) | Where-Object { $_ -and $_.fenceTitle -eq $t } | ForEach-Object { $_.fileName } | Sort-Object) -join ',' }
function FenceOf($file) { $it = @(PfJson item list) | Where-Object { $_ -and $_.fileName -eq $file } | Select-Object -First 1; if ($it) { return $it.fenceTitle }; return '(no item)' }
function Has($name) { return (Test-Path -LiteralPath "$PfDesk\$name") }
function FocusTitle($t) { $w = (PfFence $t).windowRect; [G]::Click($w.x + 40, $w.y + 14); Start-Sleep -Milliseconds 500 }
# Ctrl+Z on fence UA; Windows' own undo needs a moment to put a file back.
function Undo() { FocusTitle 'UA'; Chord 0x11 0x5A; Start-Sleep -Milliseconds 2500 }
function ClickItem($name) { $p = ItemPt 'UA' $name; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 400 }
# A title drag of UA: the layout step that Ctrl+Z must not undo before the newer file step.
function DragFence($dx) {
  $w = (PfFence 'UA').windowRect
  DragTo ($w.x + 150) ($w.y + 14) ($w.x + 150 + $dx) ($w.y + 14) 12
  Start-Sleep -Milliseconds 800
}
function UndoLog() { return ((@(PfLogAll | Where-Object { $_ -match 'undo step noted|undone|nothing to undo' -and $_ -notmatch '<omitted>' } | Select-Object -Last 4 | ForEach-Object { $_.Substring([math]::Min(28, $_.Length)) })) -join ' / ') }

Case 'setup'
CloseWindows
UClean
Clear-RecycleBin -Force -ErrorAction SilentlyContinue
PfStart
Start-Sleep -Seconds 2
ClearFences | Out-Null
foreach ($n in 'u1 报告.txt', 'u2 预算表 2026.txt', 'u3 会议记录.txt', 'u4 照片说明文档.txt', 'u5 合同.txt', 'u6 发票.txt', 'u7 行程单.txt') { DeskFile $n | Out-Null }
New-Item -ItemType Directory -Force "$PfDesk\u9 归档夹" | Out-Null
Start-Sleep -Seconds 2
Pf fence create --title UA --rect '300,160,420,260' | Out-Null
Pf fence create --title UB --rect '800,160,300,200' | Out-Null
Pf item move "$PfDesk\u1 报告.txt" "$PfDesk\u2 预算表 2026.txt" "$PfDesk\u3 会议记录.txt" "$PfDesk\u4 照片说明文档.txt" "$PfDesk\u5 合同.txt" "$PfDesk\u6 发票.txt" "$PfDesk\u7 行程单.txt" "$PfDesk\u9 归档夹" --to UA | Out-Null
Pf fence set UA sort name | Out-Null
Start-Sleep -Seconds 1
$all = 'u1 报告.txt,u2 预算表 2026.txt,u3 会议记录.txt,u4 照片说明文档.txt,u5 合同.txt,u6 发票.txt,u7 行程单.txt,u9 归档夹'
Check 'setup: fence UA holds seven files and a folder' ((In 'UA') -eq $all) "UA has: $(In 'UA')"

if ($Part -match 'A') {
  Case 'Ctrl+Z after a rename'
  $home0 = FRect 'UA'
  DragFence 60
  $moved = FRect 'UA'
  ClickItem 'u1 报告'
  [G]::Press(0x71); Start-Sleep -Milliseconds 700
  [G]::Type('u0 改名'); Start-Sleep -Milliseconds 300
  [G]::Press(0x0D); Start-Sleep -Milliseconds 1500
  $renamed = (Has 'u0 改名.txt') -and -not (Has 'u1 报告.txt')
  Undo
  Shot 'undo-rename'
  Check 'Ctrl+Z after a rename gives the file its old name back' ($renamed -and (Has 'u1 报告.txt') -and -not (Has 'u0 改名.txt')) "renamed=$renamed; after Ctrl+Z: old name there=$(Has 'u1 报告.txt') new name there=$(Has 'u0 改名.txt'); $(UndoLog)"
  Check 'Ctrl+Z after a rename leaves the fence where it was dragged' ($moved -ne $home0 -and (FRect 'UA') -eq $moved) "start $home0, dragged $moved, after Ctrl+Z $(FRect 'UA')"
  Check 'the renamed file is still in its fence' ((FenceOf 'u1 报告.txt') -eq 'UA') "in: $(FenceOf 'u1 报告.txt')"
  Undo
  Check 'the next Ctrl+Z undoes the older fence drag' ((FRect 'UA') -eq $home0 -and (Has 'u1 报告.txt')) "start $home0, now $(FRect 'UA'); $(UndoLog)"

  Case 'Ctrl+Z after Delete'
  $p = ItemPt 'UA' 'u7 行程单'
  $t = (PfFence 'UB').windowRect
  DragTo $p[0] $p[1] ($t.x + 150) ($t.y + 110) 18
  Start-Sleep -Seconds 1
  $inB = In 'UB'
  ClickItem 'u2 预算表 2026'
  [G]::Press(0x2E); Start-Sleep -Milliseconds 2500
  $deleted = -not (Has 'u2 预算表 2026.txt')
  Undo
  Shot 'undo-delete'
  Check 'Ctrl+Z after Delete brings the file back from the Recycle Bin' ($deleted -and (Has 'u2 预算表 2026.txt')) "deleted=$deleted; after Ctrl+Z: there=$(Has 'u2 预算表 2026.txt'); bin: $(BinNames); $(UndoLog)"
  Check 'the file is back in its fence' ((FenceOf 'u2 预算表 2026.txt') -eq 'UA') "in: $(FenceOf 'u2 预算表 2026.txt')"
  Check 'Ctrl+Z after Delete leaves the icon moved before it where it is' ($inB -eq 'u7 行程单.txt' -and (In 'UB') -eq 'u7 行程单.txt') "UB after the drag: $inB; after Ctrl+Z: $(In 'UB')"
  Undo
  Check 'the next Ctrl+Z puts the icon moved before back' ((In 'UB') -eq '' -and (FenceOf 'u7 行程单.txt') -eq 'UA') "UB: $(In 'UB'); u7 in: $(FenceOf 'u7 行程单.txt'); $(UndoLog)"

  Case 'Ctrl+Z after a drop into a folder'
  $p = ItemPt 'UA' 'u3 会议记录'
  $q = ItemPt 'UA' 'u9 归档夹'
  DragTo $p[0] $p[1] $q[0] $q[1] 18
  Start-Sleep -Milliseconds 2500
  $inFolder = (Test-Path -LiteralPath "$PfDesk\u9 归档夹\u3 会议记录.txt") -and -not (Has 'u3 会议记录.txt')
  Undo
  Check 'Ctrl+Z after a drop into a folder takes the file back out' ($inFolder -and (Has 'u3 会议记录.txt') -and -not (Test-Path -LiteralPath "$PfDesk\u9 归档夹\u3 会议记录.txt")) "in the folder after the drop=$inFolder; after Ctrl+Z: on the desktop=$(Has 'u3 会议记录.txt'); $(UndoLog)"
  Check 'the file taken back out is in its fence' ((FenceOf 'u3 会议记录.txt') -eq 'UA') "in: $(FenceOf 'u3 会议记录.txt')"

  Case 'Ctrl+Z after a pasted copy'
  ClickItem 'u4 照片说明文档'
  Chord 0x11 0x43; Start-Sleep -Milliseconds 600
  Chord 0x11 0x56; Start-Sleep -Milliseconds 3000
  $copies = @(UFiles | Where-Object { $_.Name -like 'u4 *' } | ForEach-Object { $_.Name })
  Undo
  $left = @(UFiles | Where-Object { $_.Name -like 'u4 *' } | ForEach-Object { $_.Name })
  Check 'Ctrl+Z after a pasted copy removes the copy' ($copies.Count -eq 2 -and $left.Count -eq 1 -and $left[0] -eq 'u4 照片说明文档.txt') "after the paste: $($copies -join ' | '); after Ctrl+Z: $($left -join ' | '); $(UndoLog)"

  Case 'Ctrl+Z after a cut icon is pasted into another fence'
  $home3 = FRect 'UA'
  DragFence 60
  $moved = FRect 'UA'
  ClickItem 'u6 发票'
  Chord 0x11 0x58; Start-Sleep -Milliseconds 600
  $t = (PfFence 'UB').windowRect
  [G]::Click($t.x + 150, $t.y + 110); Start-Sleep -Milliseconds 400
  Chord 0x11 0x56; Start-Sleep -Milliseconds 1500
  $inB = In 'UB'
  Undo
  Check 'Ctrl+Z after a cut icon is pasted into another fence puts it back' ($inB -eq 'u6 发票.txt' -and (In 'UB') -eq '' -and (FenceOf 'u6 发票.txt') -eq 'UA') "UB after the paste: $inB; after Ctrl+Z: UB has '$(In 'UB')', u6 in $(FenceOf 'u6 发票.txt'); $(UndoLog)"
  Check 'Ctrl+Z after the paste leaves the fence where it was dragged' ($moved -ne $home3 -and (FRect 'UA') -eq $moved) "start $home3, dragged $moved, after Ctrl+Z $(FRect 'UA')"
  Undo
  Check 'the next Ctrl+Z undoes the fence drag made before the paste' ((FRect 'UA') -eq $home3) "start $home3, now $(FRect 'UA'); $(UndoLog)"
}

if ($Part -match 'B') {
  Case 'Ctrl+Z after a new folder'
  FocusTitle 'UA'
  [G]::KeyDown(0x11); [G]::KeyDown(0x10); [G]::Press(0x4E); [G]::KeyUp(0x10); [G]::KeyUp(0x11)
  Start-Sleep -Milliseconds 2000
  [G]::Type('u8 新文件夹'); Start-Sleep -Milliseconds 300
  [G]::Press(0x0D); Start-Sleep -Milliseconds 1500
  $made = (Has 'u8 新文件夹') -and -not (Has 'New folder')
  Undo
  Check 'Ctrl+Z after naming a new folder gives it the first name back' ($made -and (Has 'New folder') -and -not (Has 'u8 新文件夹')) "made and named=$made; after Ctrl+Z: 'New folder' there=$(Has 'New folder'), named one there=$(Has 'u8 新文件夹'); $(UndoLog)"
  Undo
  Check 'the next Ctrl+Z takes the new folder away, into the Recycle Bin' (-not (Has 'New folder') -and -not (Has 'u8 新文件夹') -and (BinNames) -match 'New folder') "'New folder' there=$(Has 'New folder'); bin: $(BinNames); $(UndoLog)"

  Case 'Ctrl+Z after Shift+Delete'
  $home1 = FRect 'UA'
  DragFence 60
  $moved = FRect 'UA'
  ClickItem 'u5 合同'
  [G]::KeyDown(0x10); [G]::Press(0x2E); [G]::KeyUp(0x10)
  Start-Sleep -Milliseconds 1500
  $box = @([W]::Tops('#32770'))
  if ($box.Count -gt 0) { Log "confirm: $([W]::Statics($box[0]))"; ClickButton $box[0] '^&?Yes' | Out-Null }
  Start-Sleep -Milliseconds 2500
  $gone = -not (Has 'u5 合同.txt')
  Undo
  $said = @(PfLogAll | Where-Object { $_ -match 'undone' -and $_ -match 'permanent delete' }).Count
  Check 'Ctrl+Z after Shift+Delete does not undo the older fence drag instead' ($gone -and $moved -ne $home1 -and (FRect 'UA') -eq $moved -and $said -ge 1) "deleted for good=$gone; start $home1, dragged $moved, after Ctrl+Z $(FRect 'UA'); $(UndoLog)"
  Undo
  Check 'the next Ctrl+Z undoes the fence drag' ((FRect 'UA') -eq $home1) "start $home1, now $(FRect 'UA'); $(UndoLog)"

  Case 'a file moved by the command line is no step of Ctrl+Z'
  New-Item -ItemType Directory -Force "$PfRun\u-portal" | Out-Null
  Pf fence create --portal "$PfRun\u-portal" --title UP --rect '800,420,300,200' | Out-Null
  Start-Sleep -Seconds 1
  $home2 = FRect 'UA'
  DragFence 60
  $moved = FRect 'UA'
  Pf item move "$PfDesk\u6 发票.txt" --to UP | Out-Null
  Start-Sleep -Milliseconds 2500
  $inPortal = Test-Path -LiteralPath "$PfRun\u-portal\u6 发票.txt"
  Undo
  Check 'Ctrl+Z after a command-line file move undoes the user''s fence drag, not the move' ($inPortal -and $moved -ne $home2 -and (FRect 'UA') -eq $home2 -and (Test-Path -LiteralPath "$PfRun\u-portal\u6 发票.txt")) "moved by the command line=$inPortal; start $home2, dragged $moved, after Ctrl+Z $(FRect 'UA'); still in the folder=$(Test-Path -LiteralPath "$PfRun\u-portal\u6 发票.txt"); $(UndoLog)"

  Case 'Ctrl+Z after a rename in a folder fence'
  $p = ItemPt 'UP' 'u6 发票'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 400
  [G]::Press(0x71); Start-Sleep -Milliseconds 700
  [G]::Type('u6 改名'); Start-Sleep -Milliseconds 300
  [G]::Press(0x0D); Start-Sleep -Milliseconds 1500
  $renamed = (Test-Path -LiteralPath "$PfRun\u-portal\u6 改名.txt") -and -not (Test-Path -LiteralPath "$PfRun\u-portal\u6 发票.txt")
  # The rename gave the keyboard back to the folder fence: Ctrl+Z goes there.
  Chord 0x11 0x5A; Start-Sleep -Milliseconds 2000
  Check 'Ctrl+Z after a rename in a folder fence gives the file its old name back' ($renamed -and (Test-Path -LiteralPath "$PfRun\u-portal\u6 发票.txt") -and -not (Test-Path -LiteralPath "$PfRun\u-portal\u6 改名.txt")) "renamed=$renamed; after Ctrl+Z the folder holds: $((@(Get-ChildItem "$PfRun\u-portal" | ForEach-Object { $_.Name })) -join ' | '); $(UndoLog)"

  Case 'Ctrl+Z after a file copied in another folder is pasted'
  New-Item -ItemType Directory -Force "$PfRun\u-src" | Out-Null
  Set-Content -LiteralPath "$PfRun\u-src\u8 外来文件.txt" -Value 'from another folder' -Encoding ASCII
  Set-Clipboard -Path "$PfRun\u-src\u8 外来文件.txt"
  FocusTitle 'UA'
  Chord 0x11 0x56; Start-Sleep -Milliseconds 3000
  $pasted = (Has 'u8 外来文件.txt') -and (FenceOf 'u8 外来文件.txt') -eq 'UA'
  Undo
  Check 'Ctrl+Z after a file copied in another folder is pasted takes the copy away again' ($pasted -and -not (Has 'u8 外来文件.txt') -and (Test-Path -LiteralPath "$PfRun\u-src\u8 外来文件.txt")) "pasted into UA=$pasted; after Ctrl+Z: on the desktop=$(Has 'u8 外来文件.txt'), original still there=$(Test-Path -LiteralPath "$PfRun\u-src\u8 外来文件.txt"); $(UndoLog)"
}

Case 'cleanup'
PfQuit
UClean
Clear-RecycleBin -Force -ErrorAction SilentlyContinue
