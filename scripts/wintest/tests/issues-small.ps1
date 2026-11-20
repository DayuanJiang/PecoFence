# Issues round 2026-11-15, package small: three small requests.
#   C  feedback #175: a rolled-up clock fence shows the time (hours and minutes, Windows' region
#      format) in its title row in place of its name, and the next minute without a click; open,
#      it shows its name and the clock face as before. Also with the title row shown on hover,
#      and with seconds switched on (the title has none).
#   U  feedback #130 (and #156): in a folder fence that shows a subfolder, a double-click on empty
#      space goes up one level, in the icons, list and details views; at the folder's root, in a
#      plain fence and on the details header it does nothing new.
#   R  feedback #120: list and details rows follow the fence's 「图标大小」: at 48 as before, taller
#      rows at 64 and 96 (the fitting height grows), the same rows at 32; the icons view as before.
#   A  recordings and pictures for the report (no checks).
# Default: the parts with checks. 1600x900 box at 100 %, Fluent, light, grey wallpaper.
if (-not (Get-Variable Part -ErrorAction SilentlyContinue) -or -not $Part) { $Part = 'CUR' }

Case 'setup'
CloseWindows
# A run cut short in part U may have left Explorer's single-click open on.
if ([Th]::SingleClick()) { [Th]::SetSingleClick($false) }
Set-Culture en-US
GreyWallpaper
. C:\pf\kit\rec.ps1
PfStart -Script (DumpScript 400 25 3000)
Start-Sleep -Seconds 2

# What a rolled-up clock shows in its title row (the dump's rolled_caption): the text, '' for
# None, $null when the build has no such field.
function Caption($w) {
  if ($w -and $w.raw -match ' rolled_caption=Some\("([^"]*)"\)') { return $Matches[1] }
  if ($w -and $w.raw -match ' rolled_caption=None') { return '' }
  return $null
}
function DumpById($id) { foreach ($w in @(PfDump 6)) { if ((DumpVal $w 'active') -eq $id) { return $w } }; return $null }
# The words Windows' OCR reads in a fence's title row.
function TitleOcr($w, $name) {
  ParkMouse
  Start-Sleep -Milliseconds 500
  return (OcrText ($w.l + 4) $w.t ([math]::Min(300, $w.r - $w.l - 40)) 34 $name)
}
function NowHm() { $n = Get-Date; return @(($n.Hour % 12), $n.Minute) }
# "2:32 PM" -> @(2, 32) (hours on a 12 hour dial), $null when it is no time.
function Hm($t) { if ($t -match '(\d{1,2}):(\d{2})') { return @((([int]$Matches[1]) % 12), [int]$Matches[2]) }; return $null }
function SameMinute($t) {
  $hm = Hm $t; if (-not $hm) { return $false }
  $a = NowHm
  return ($hm[0] -eq $a[0] -and [math]::Abs($hm[1] - $a[1]) -le 1)
}
# The folder a folder fence shows: portal.current inside a subfolder, else its root (portal.path).
function FolderOf($id) { $f = FenceGet $id; if ($f.portal.current) { return "$($f.portal.current)" }; return "$($f.portal.path)" }
function EmptySpot($id) { $wr = (FenceGet $id).windowRect; return @([int]($wr.x + $wr.w - 40), [int]($wr.y + $wr.h - 40)) }
function TestTree($name) {
  $root = "$PfRun\$name"
  New-Item -ItemType Directory -Force "$root\a子文件夹" | Out-Null
  Set-Content -LiteralPath "$root\说明文件.txt" 'root file'
  Set-Content -LiteralPath "$root\a子文件夹\第三季度报告（最终版）.txt" 'x'
  Set-Content -LiteralPath "$root\a子文件夹\b.txt" 'y'
  return $root
}
function RowsFolder($name, $n) {
  $root = "$PfRun\$name"
  New-Item -ItemType Directory -Force $root | Out-Null
  $names = @('报告', '第三季度销售数据汇总表', 'readme', '会议纪要', '预算', '设计稿第二版')
  $ext = @('.txt', '.docx', '.png', '.md', '.xlsx', '.pdf')
  for ($i = 0; $i -lt $n; $i++) { Set-Content -LiteralPath "$root\$($names[$i])$($ext[$i])" ('x' * (($i + 1) * 300)) }
  return $root
}
function FitH($id) { $f = FenceGet $id; return [int]$f.fit.fittingHeight }

function PartC() {
  trap { PfTrap $_; continue }
  Case 'C rolled-up clock shows the time in its title row (feedback #175)'
  ClearFences | Out-Null
  $cid = "$((PfJson fence create --tool clock --title 时钟 --rect '300,200,320,220').fence.id)"
  Check 'C a clock was made' ($cid -ne '') "id=$cid"
  Start-Sleep -Seconds 2
  $w = DumpById $cid
  $c0 = Caption $w
  Check 'C open, the title row shows the name (no time in the title)' (-not $c0 -and $w.title -eq '时钟') "caption='$c0' title=$($w.title)"
  Pf fence roll $cid | Out-Null
  Start-Sleep -Seconds 2
  $w = DumpById $cid
  $c1 = Caption $w
  Check 'C rolled up, the title row shows the time now' ((DumpVal $w 'rolled') -eq 'true' -and (SameMinute $c1)) "rolled=$(DumpVal $w 'rolled') caption='$c1' now=$((Get-Date).ToString('HH:mm'))"
  Check 'C the time in the title is hours and minutes in the region format (en-US: h:mm AM/PM)' ($c1 -match '^\d{1,2}:\d{2} [AP]M$') "caption='$c1'"
  $ocr = TitleOcr $w 'C-rolled-title'
  Check 'C the rolled title row reads the time on screen' ((SameMinute $ocr) -and $ocr -notmatch '时钟') "ocr='$ocr'"
  Shot 'C-rolled'
  # The next minute comes without a click: wait for the minute to change.
  $m0 = (Get-Date).Minute
  for ($i = 0; $i -lt 70 -and (Get-Date).Minute -eq $m0; $i++) { Start-Sleep -Seconds 1 }
  Start-Sleep -Seconds 3
  $w = DumpById $cid
  $c2 = Caption $w
  $ocr2 = TitleOcr $w 'C-next-minute'
  Check 'C a minute later the title shows the new minute' ((SameMinute $c2) -and $c2 -ne $c1 -and (SameMinute $ocr2)) "before='$c1' after='$c2' ocr='$ocr2'"
  Pf fence unroll $cid | Out-Null
  Start-Sleep -Seconds 2
  $w = DumpById $cid
  $c3 = Caption $w
  $ocr3 = TitleOcr $w 'C-open-title'
  Check 'C opened again, the title row shows the name again (no time there)' (-not $c3 -and -not (Hm $ocr3)) "caption='$c3' ocr='$ocr3'"
  Check 'C open, the clock face shows the time as before' ((DumpVal $w 'clock_time') -ne $null) "clock_time=$(DumpVal $w 'clock_time') clock_text=$(DumpVal $w 'clock_text')"

  Case 'C with seconds switched on, the rolled title has hours and minutes only'
  Pf settings set tools.clockSeconds true | Out-Null
  Start-Sleep -Seconds 1
  Pf fence roll $cid | Out-Null
  Start-Sleep -Seconds 2
  $w = DumpById $cid
  $c4 = Caption $w
  Check 'C seconds on: the rolled title shows h:mm without seconds' ($c4 -match '^\d{1,2}:\d{2} [AP]M$') "caption='$c4'"
  Pf fence unroll $cid | Out-Null
  Pf settings set tools.clockSeconds false | Out-Null
  Start-Sleep -Seconds 1

  Case 'C with the title row shown only on hover'
  Pf fence set $cid titleOnHover true | Out-Null
  Start-Sleep -Seconds 1
  Pf fence roll $cid | Out-Null
  Start-Sleep -Seconds 2
  $w = DumpById $cid
  $ocr5 = TitleOcr $w 'C-hover-title'
  Check 'C title on hover: rolled up, the title row shows and reads the time' ((SameMinute $ocr5)) "ocr='$ocr5' caption='$(Caption $w)'"
  Pf fence unroll $cid | Out-Null
  Pf fence set $cid titleOnHover null | Out-Null

  Case 'C after a restart, a clock that was rolled up still shows the time'
  Pf fence roll $cid | Out-Null
  Start-Sleep -Seconds 2
  PfQuit
  PfStart -Script (DumpScript 400 25 3000)
  Start-Sleep -Seconds 3
  $w = DumpById $cid
  $c6 = Caption $w
  Check 'C restarted rolled up: the title shows the time' ((DumpVal $w 'rolled') -eq 'true' -and (SameMinute $c6)) "rolled=$(DumpVal $w 'rolled') caption='$c6'"
  Pf fence unroll $cid | Out-Null
  Start-Sleep -Seconds 1
}

function PartU() {
  trap { PfTrap $_; continue }
  Case 'U double-click on empty space in a subfolder goes up (feedback #130)'
  # These checks are about going up; at a folder's root and in a plain fence a double-click on
  # empty space zooms the fence now (PR #245, its own steps in pr245-zoom.ps1): off here.
  Pf settings set zoom.enabled false | Out-Null
  ClearFences | Out-Null
  CloseWindows
  $root = TestTree '上级文件夹'
  $id = "$((PfJson fence create --portal $root --rect '300,180,440,320').fence.id)"
  Check 'U a folder fence was made' ($id -ne '') "id=$id root=$root"
  Start-Sleep -Seconds 2
  $p = ItemAt $id 'a子文件夹'
  DoubleClick $p[0] $p[1]; Start-Sleep -Milliseconds 1500
  $in = FolderOf $id
  Check 'U double-click on the subfolder opens it in the fence' ($in -like '*a子文件夹') "folder=$in"
  $e = EmptySpot $id
  DoubleClick $e[0] $e[1]; Start-Sleep -Milliseconds 1500
  $up = FolderOf $id
  Check 'U icons view: double-click on empty space goes up to the parent folder' ($up -eq $root) "folder=$up"
  $exp0 = @((New-Object -ComObject Shell.Application).Windows()).Count
  $n0 = @(@(PfJson fence list) | Where-Object { $_ -and $_.id }).Count
  DoubleClick $e[0] $e[1]; Start-Sleep -Milliseconds 1500
  $root2 = FolderOf $id
  $exp1 = @((New-Object -ComObject Shell.Application).Windows()).Count
  $n1 = @(@(PfJson fence list) | Where-Object { $_ -and $_.id }).Count
  Check 'U at the root a double-click on empty space changes nothing' ($root2 -eq $root -and $exp1 -eq $exp0 -and $n1 -eq $n0 -and (DumpVal (DumpById $id) 'rolled') -eq 'false') "folder=$root2 explorer windows $exp0 -> $exp1 fences $n0 -> $n1"

  Pf fence forward $id | Out-Null; Start-Sleep -Milliseconds 800
  Pf fence set $id layout list | Out-Null; Start-Sleep -Milliseconds 1200
  Check 'U list view: back in the subfolder' ((FolderOf $id) -like '*a子文件夹') "folder=$(FolderOf $id)"
  DoubleClick $e[0] $e[1]; Start-Sleep -Milliseconds 1500
  Check 'U list view: double-click on empty space goes up' ((FolderOf $id) -eq $root) "folder=$(FolderOf $id)"

  Pf fence forward $id | Out-Null; Start-Sleep -Milliseconds 800
  Pf fence set $id layout details | Out-Null; Start-Sleep -Milliseconds 1200
  $wr = (FenceGet $id).windowRect
  # The details header: 36 px title row, then the 28 px header band (the 名称 column's right part).
  DoubleClick ([int]($wr.x + 150)) ([int]($wr.y + 36 + 14)); Start-Sleep -Milliseconds 1500
  Check 'U details view: a double-click on the header stays in the subfolder' ((FolderOf $id) -like '*a子文件夹') "folder=$(FolderOf $id)"
  DoubleClick $e[0] $e[1]; Start-Sleep -Milliseconds 1500
  Check 'U details view: double-click on empty space goes up' ((FolderOf $id) -eq $root) "folder=$(FolderOf $id)"
  Pf fence set $id layout icons | Out-Null

  # A double-click on a file still opens it, in the subfolder too.
  Pf fence forward $id | Out-Null; Start-Sleep -Milliseconds 1200
  $p = ItemAt $id 'b'
  if (-not $p) { $p = ItemAt $id 'b.txt' }
  DoubleClick $p[0] $p[1]; Start-Sleep -Milliseconds 2500
  $note = NotepadTitles
  Check 'U a double-click on a file in the subfolder opens it and stays there' ($note -match 'b' -and (FolderOf $id) -like '*a子文件夹') "notepad='$note' folder=$(FolderOf $id)"
  CloseWindows

  Case 'U single-click open: a double-click on a subfolder goes in and stays there'
  # The subfolder comes third and holds one file, so the second click of the double-click lands on
  # empty space of the folder the first click (single-click open) went into.
  $r2 = "$PfRun\单击打开"
  New-Item -ItemType Directory -Force "$r2\c子文件夹" | Out-Null
  Set-Content -LiteralPath "$r2\a1.txt" '1'
  Set-Content -LiteralPath "$r2\a2.txt" '2'
  Set-Content -LiteralPath "$r2\c子文件夹\唯一的文件.txt" '3'
  $sid = "$((PfJson fence create --portal $r2 --rect '300,560,440,240').fence.id)"
  Pf fence set $sid foldersFirst false | Out-Null
  Start-Sleep -Seconds 2
  [Th]::SetSingleClick($true)
  Start-Sleep -Seconds 2
  $seen = PfWaitLog 'click to open setting changed' 4
  $p = ItemAt $sid 'c子文件夹'
  DoubleClick $p[0] $p[1]; Start-Sleep -Milliseconds 2000
  $in2 = FolderOf $sid
  Check 'U single-click open: a double-click on a subfolder goes in and stays in it' ($in2 -like '*c子文件夹') "folder=$in2 setting seen=$([bool]$seen)"
  $e3 = EmptySpot $sid
  DoubleClick $e3[0] $e3[1]; Start-Sleep -Milliseconds 1500
  Check 'U single-click open: a double-click on empty space still goes up' ((FolderOf $sid) -eq $r2) "folder=$(FolderOf $sid)"
  [Th]::SetSingleClick($false)
  Start-Sleep -Seconds 1
  CloseWindows

  Case 'U a plain fence: double-click on empty space changes nothing'
  DeskFile '普通文件.txt' | Out-Null
  $pid2 = "$((PfJson fence create --title 普通栅栏 --rect '900,180,360,260').fence.id)"
  Start-Sleep -Seconds 1
  Pf item move '普通文件.txt' --to $pid2 | Out-Null
  Start-Sleep -Seconds 1
  $e2 = EmptySpot $pid2
  $n0 = @(@(PfJson fence list) | Where-Object { $_ -and $_.id }).Count
  $exp0 = @((New-Object -ComObject Shell.Application).Windows()).Count
  DoubleClick $e2[0] $e2[1]; Start-Sleep -Milliseconds 1500
  $n1 = @(@(PfJson fence list) | Where-Object { $_ -and $_.id }).Count
  $exp1 = @((New-Object -ComObject Shell.Application).Windows()).Count
  Check 'U plain fence: no window opened, fence still open' ($n1 -eq $n0 -and $exp1 -eq $exp0 -and (DumpVal (DumpById $pid2) 'rolled') -eq 'false') "fences $n0 -> $n1 explorer $exp0 -> $exp1"

  Case 'U a plain fence that opens folders inside goes back the same way'
  $dir = Join-Path $PfDesk 'a桌面文件夹'
  New-Item -ItemType Directory -Force $dir | Out-Null
  Set-Content -LiteralPath "$dir\里面的文件.txt" 'z'
  Start-Sleep -Seconds 2
  Pf item move 'a桌面文件夹' --to $pid2 | Out-Null
  Pf fence set $pid2 openFoldersInside true | Out-Null
  Start-Sleep -Seconds 1
  $p = ItemAt $pid2 'a桌面文件夹'
  DoubleClick $p[0] $p[1]; Start-Sleep -Milliseconds 1500
  $opened = "$((FenceGet $pid2).openedFolder)"
  Check 'U the folder opens inside the plain fence' ($opened -like '*a桌面文件夹') "openedFolder=$opened"
  DoubleClick $e2[0] $e2[1]; Start-Sleep -Milliseconds 1500
  $back = "$((FenceGet $pid2).openedFolder)"
  $exp2 = @((New-Object -ComObject Shell.Application).Windows()).Count
  Check 'U double-click on empty space goes back to the fence''s own icons' (-not $back -and $exp2 -eq $exp0) "openedFolder='$back' explorer $exp0 -> $exp2"
  Remove-Item -LiteralPath (Join-Path $PfDesk '普通文件.txt') -ErrorAction SilentlyContinue
  Remove-Item -LiteralPath $dir -Recurse -Force -ErrorAction SilentlyContinue
  Pf settings set zoom.enabled true | Out-Null
}

function PartR() {
  trap { PfTrap $_; continue }
  Case 'R list and details rows follow the icon size (feedback #120)'
  ClearFences | Out-Null
  $root = RowsFolder '行图标' 6
  $id = "$((PfJson fence create --portal $root --rect '300,120,420,300').fence.id)"
  Start-Sleep -Seconds 2
  foreach ($layout in @('list', 'details')) {
    Pf fence set $id layout $layout | Out-Null
    $h = @{}
    foreach ($size in @(48, 32, 64, 96)) {
      Pf fence set $id iconSize $size | Out-Null
      Start-Sleep -Milliseconds 1200
      $h[$size] = FitH $id
    }
    Log "$layout fitting heights: 32=$($h[32]) 48=$($h[48]) 64=$($h[64]) 96=$($h[96])"
    Check "R ${layout}: icon size 32 keeps the rows of 48" ($h[32] -eq $h[48] -and $h[48] -gt 0) "32=$($h[32]) 48=$($h[48])"
    # Six rows: 7 DIP taller each at 64, 20 at 96.
    Check "R ${layout}: icon size 64 makes the rows taller (6 rows x 7)" ($h[64] - $h[48] -ge 36 -and $h[64] - $h[48] -le 48) "48=$($h[48]) 64=$($h[64]) diff=$($h[64] - $h[48])"
    Check "R ${layout}: icon size 96 makes them taller still (6 rows x 20)" ($h[96] - $h[48] -ge 114 -and $h[96] - $h[48] -le 126) "48=$($h[48]) 96=$($h[96]) diff=$($h[96] - $h[48])"
    Pf fence fit $id | Out-Null; Start-Sleep -Milliseconds 1200
    $w = DumpById $id
    Crop "R-$layout-96" $w.l $w.t ($w.r - $w.l) ($w.b - $w.t) | Out-Null
    Pf fence set $id iconSize 48 | Out-Null; Start-Sleep -Milliseconds 800
    Pf fence fit $id | Out-Null; Start-Sleep -Milliseconds 1200
    $w = DumpById $id
    Crop "R-$layout-48" $w.l $w.t ($w.r - $w.l) ($w.b - $w.t) | Out-Null
  }
  # The Menu key opens the item menu under the name's box, inside the row: at icon size 96 the
  # box's rect ended 8 DIPs into the next row, and so did the menu's top.
  Pf fence set $id layout list | Out-Null
  Pf fence set $id iconSize 96 | Out-Null; Start-Sleep -Milliseconds 1200
  Pf fence fit $id | Out-Null; Start-Sleep -Milliseconds 1000
  $wr = (FenceGet $id).windowRect
  $rowTop = $wr.y + 36 + 4
  [G]::Click([int]($wr.x + 120), [int]($rowTop + 24)); Start-Sleep -Milliseconds 700
  [G]::Press(0x5D); Start-Sleep -Milliseconds 900
  $m = MenuWin 5
  $mt = if ($m -ne [IntPtr]::Zero) { (Rect $m).T } else { $null }
  Check 'R list at 96: the Menu key opens the menu inside the first row, not lower' ($mt -ne $null -and $mt -le $rowTop + 48) "menu top=$mt row top=$rowTop row bottom=$($rowTop + 48)"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  DeskClick; Start-Sleep -Milliseconds 400
  # Ctrl+wheel over the rows steps the icon size there too (it did nothing in these views).
  Pf fence set $id layout list | Out-Null
  Pf fence set $id iconSize 48 | Out-Null; Start-Sleep -Milliseconds 1000
  $wr = (FenceGet $id).windowRect
  [G]::KeyDown(0x11); Start-Sleep -Milliseconds 80
  Wheel ([int]($wr.x + $wr.w / 2)) ([int]($wr.y + 80)) 1
  Start-Sleep -Milliseconds 300
  [G]::KeyUp(0x11); Start-Sleep -Milliseconds 1200
  $z = (FenceGet $id).iconSize
  Check 'R list view: Ctrl+wheel up steps the icon size 48 -> 64' ($z -eq 64) "iconSize=$z"
  Pf fence set $id layout icons | Out-Null
  Pf fence set $id iconSize 48 | Out-Null; Start-Sleep -Milliseconds 800
  $i48 = FitH $id
  Pf fence set $id iconSize 96 | Out-Null; Start-Sleep -Milliseconds 1200
  $i96 = FitH $id
  Check 'R icons view: a larger icon size still makes bigger cells' ($i96 -gt $i48) "48=$i48 96=$i96"
}

function PartA() {
  trap { PfTrap $_; continue }
  [Anim]::Set($true)
  Case 'A recordings'
  ClearFences | Out-Null
  CloseWindows
  $cid = "$((PfJson fence create --tool clock --title 时钟 --rect '420,220,320,220').fence.id)"
  Start-Sleep -Seconds 2
  $w = DumpById $cid
  ParkMouse
  RecStart 'A-clock' -X 380 -Y 180 -W 400 -H 300 -Fps 15
  RecMark '展开的时钟栅栏'
  Start-Sleep -Milliseconds 1500
  RecMark '点标题栏右边的箭头，收起'
  Pf fence roll $cid | Out-Null
  Start-Sleep -Milliseconds 2500
  RecMark '收起后，看标题栏'
  Start-Sleep -Milliseconds 2500
  RecStop
  Pf fence unroll $cid | Out-Null
  Start-Sleep -Seconds 1

  ClearFences | Out-Null
  $root = TestTree '上级文件夹'
  $id = "$((PfJson fence create --portal $root --rect '420,200,440,320').fence.id)"
  Start-Sleep -Seconds 2
  $p = ItemAt $id 'a子文件夹'
  $e = EmptySpot $id
  RecStart 'A-up' -X 400 -Y 180 -W 480 -H 360 -Fps 15
  RecMark '双击子文件夹，进入'
  [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 600
  DoubleClick $p[0] $p[1]; Start-Sleep -Milliseconds 1800
  RecMark '在空白处双击'
  [G]::MoveTo($e[0], $e[1]); Start-Sleep -Milliseconds 600
  DoubleClick $e[0] $e[1]; Start-Sleep -Milliseconds 2200
  RecStop
  Log "A-up folder at the end: $(FolderOf $id)"
}

if ($Part -match 'C') { PartC }
if ($Part -match 'U') { PartU }
if ($Part -match 'R') { PartR }
if ($Part -match 'A') { PartA }
CloseWindows
