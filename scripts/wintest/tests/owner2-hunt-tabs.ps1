# Owner round 2026-11-12, second batch: tabs and folder fences (package hunt-tabs).
# One part per fix; `run.sh --var Part=A` runs one part, no Part runs them all.
#   A  Ctrl+Z takes back a merge by title drag and a tab dragged out
#   B  a folder fence whose folder goes away: the folder in the title, and the folder coming back
#   C  a tab split out lands off the window it left while 「桌面」 is arranged like the desktop
#   D  a tab dragged along the strip hides the names it passes over
#   E  the tips of the title's folder and up arrow name the folder shown
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'ABCDE' }

function Sx($d, $x) { return [int]($d.l + $d.ox + $x) }
function TitleY($d) { return [int]($d.t + $d.oy + 18) }
function TabX($d, $i) { return (Sx $d ($d.rects[$i][0] + $d.rects[$i][1] / 2)) }
function RectText($f) { $r = $f.rect; return "$($r.x),$($r.y),$($r.w),$($r.h)" }
function HostOf($title) { $f = FenceGet $title; if ($f.tabHost) { return "$($f.tabHost)" }; return '' }
function SmallInbox() {
  Pf fence set inbox desktopLayout false | Out-Null
  Pf fence move inbox --x 1300 --y 500 | Out-Null
  Pf fence resize inbox --w 280 --h 300 | Out-Null
}
function LastLog($pattern) { return ((PfLogAll | Select-String -Pattern $pattern | Select-Object -Last 1) -replace '^\S+\s+', '') }

function PartA() {
  trap { PfTrap $_; continue }
  Case 'A Ctrl+Z after a merge by drag and after a tab dragged out'
  PfStart -Script (DumpScript) -Lang 'zh-CN'
  Pf settings set rollUp.switchTabOnHover false | Out-Null
  ClearFences | Out-Null; SmallInbox
  Pf fence create --title 工作 --rect '60,60,420,260' | Out-Null
  Pf fence create --title 图片素材 --rect '60,400,300,200' | Out-Null
  Pf fence create --title 独立 --rect '700,60,300,200' | Out-Null
  Pf item move "$PfDesk\年度报告.txt" --to 工作 | Out-Null
  Pf item move "$PfDesk\设计稿-最终版.txt" --to 图片素材 | Out-Null
  Pf item move "$PfDesk\独立的文件.txt" --to 独立 | Out-Null
  Pf fence merge 图片素材 --into 工作 | Out-Null
  Start-Sleep -Seconds 2
  FenceIds; $work = $Id['工作']
  DeskClick 1100 760; Start-Sleep -Milliseconds 400
  $d = DumpOf '工作'; $s = DumpOf '独立'
  [G]::MoveTo((Sx $s 150), (TitleY $s)); Start-Sleep -Milliseconds 250; MDown; Start-Sleep -Milliseconds 120
  Glide (Sx $s 150) (TitleY $s) (Sx $d 380) (TitleY $d) 16 30; Start-Sleep -Milliseconds 500; MUp; Start-Sleep -Milliseconds 900
  Check 'a title dropped on the strip merges 独立 into 工作' ((HostOf '独立') -eq $work) "host of 独立=$(HostOf '独立') 工作=$work"
  Chord 0x11 0x5A; Start-Sleep -Milliseconds 1200
  $f = FenceGet '独立'
  Check 'Ctrl+Z makes 独立 a fence of its own again' (-not $f.tabHost) "host=$($f.tabHost); $(LastLog 'undo: undone|nothing to undo')"
  Check 'where it stood before the drag' ((RectText $f) -eq '700,60,300,200') "rect=$(RectText $f)"
  Check '工作 keeps its two tabs' (@((FenceGet '工作').tabs).Count -eq 1 -and (HostOf '图片素材') -eq $work) "tabs of 工作=$(@((FenceGet '工作').tabs).Count) host of 图片素材=$(HostOf '图片素材')"
  Shot 'A1-merge-undone'

  $d = DumpOf '工作'; $before = $d.tabs -join ','
  $i = [array]::IndexOf(@($d.tabs), '图片素材')
  $x0 = TabX $d $i; $y0 = TitleY $d
  [G]::MoveTo($x0, $y0); Start-Sleep -Milliseconds 250; MDown; Start-Sleep -Milliseconds 120
  Glide $x0 $y0 ($x0 + 420) ($y0 + 420) 16 30; Start-Sleep -Milliseconds 500; MUp; Start-Sleep -Milliseconds 900
  $out = FenceGet '图片素材'
  Check 'a tab dragged out is a fence of its own' (-not $out.tabHost) "host=$($out.tabHost) rect=$(RectText $out)"
  Chord 0x11 0x5A; Start-Sleep -Milliseconds 1200
  $d2 = DumpOf '工作'
  Check 'Ctrl+Z puts the tab back into 工作' ((HostOf '图片素材') -eq $work) "host=$(HostOf '图片素材'); $(LastLog 'undo: undone|nothing to undo')"
  Check 'at its place in the strip' (($d2.tabs -join ',') -eq $before) "strip=$($d2.tabs -join ',') before=$before"
  $own = @(PfDump | Where-Object { $_.title -eq '图片素材' }).Count
  Check 'and no window of its own is left' ($own -eq 0) "windows titled 图片素材: $own"
  Chord 0x11 0x5A; Start-Sleep -Milliseconds 900
  Check 'one more Ctrl+Z finds nothing left of the two drags' ((LastLog 'undo: undone|nothing to undo') -match 'nothing to undo') (LastLog 'undo: undone|nothing to undo')
  Shot 'A2-split-undone'
  PfQuit
}

function Count($fence) { return @(@(PfJson item list --fence $fence) | Where-Object { $_ -and $_.name }).Count }
# Milliseconds until the fence lists $n items, or a text when it never does within $sec.
function WaitCount($fence, $n, $sec) {
  $t0 = Get-Date
  while (((Get-Date) - $t0).TotalSeconds -lt $sec) { $c = Count $fence; if ($c -eq $n) { return [int]((Get-Date) - $t0).TotalMilliseconds }; Start-Sleep -Milliseconds 200 }
  return "never in $sec s (count=$c, want $n)"
}

function PartB() {
  trap { PfTrap $_; continue }
  Case 'B a folder fence whose folder goes away and comes back'
  $root = "$PfRun\资料夹"
  New-Item -ItemType Directory -Force "$root\子文件夹甲" | Out-Null
  foreach ($n in '计划书.txt', '预算表.txt') { Set-Content -LiteralPath "$root\$n" 'x' }
  PfStart -Script (DumpScript) -Lang 'zh-CN'
  ClearFences | Out-Null; SmallInbox
  $fid = (PfJson fence create --portal $root --rect '60,60,480,330').fence.id
  Pf fence create --title 普通 --rect '620,60,300,220' | Out-Null
  Pf item move "$PfDesk\年度报告.txt" --to 普通 | Out-Null
  Start-Sleep -Seconds 2
  DeskClick 1100 760; Start-Sleep -Milliseconds 300
  Check 'the folder fence shows its three items' ((Count $fid) -eq 3) "count=$(Count $fid)"
  # The folder in the title, clicked while the folder is away and the fence still shows its icons.
  Rename-Item -LiteralPath $root '资料夹-改名'; Start-Sleep -Milliseconds 1000
  [G]::Click(80, 78); Start-Sleep -Milliseconds 2000
  $fg = Fg
  Check 'a click on the folder in the title opens no Windows error box' ($fg -notmatch '#32770') "foreground: $fg"
  Shot 'B1-glyph-click'
  if ($fg -match '#32770') { [G]::Press(0x0D); Start-Sleep -Milliseconds 800 }
  Check 'the fence says that the folder is gone' ((Count $fid) -eq 0) "count=$(Count $fid); $(LastLog 'the folder is gone')"
  Rename-Item -LiteralPath "$PfRun\资料夹-改名" '资料夹'
  $ms = WaitCount $fid 3 8
  Check 'back under its name, the folder is shown again within 3 s' ($ms -is [int] -and $ms -le 3000) "shown after $ms ms"
  # One of its icons, double-clicked while the folder is away.
  Rename-Item -LiteralPath $root '资料夹-改名'; Start-Sleep -Milliseconds 1000
  $p = ItemAt $fid '计划书'
  DblClick $p[0] $p[1]; Start-Sleep -Milliseconds 2000
  $fg = Fg
  Check 'a double-click on one of its icons opens no Windows error box' ($fg -notmatch '#32770') "foreground: $fg"
  Shot 'B2-icon-double-click'
  if ($fg -match '#32770') { [G]::Press(0x0D); Start-Sleep -Milliseconds 800 }
  Check 'and the fence says that the folder is gone' ((Count $fid) -eq 0) "count=$(Count $fid); $(LastLog 'the file is gone')"
  Rename-Item -LiteralPath "$PfRun\资料夹-改名" '资料夹'
  $ms = WaitCount $fid 3 8
  Check 'the folder is shown again within 3 s of its return' ($ms -is [int] -and $ms -le 3000) "shown after $ms ms"
  # A file dropped on the fence while the folder is away.
  Rename-Item -LiteralPath $root '资料夹-改名'; Start-Sleep -Milliseconds 800
  $p = ItemAt 普通 '年度报告'
  DragTo $p[0] $p[1] 400 300; Start-Sleep -Milliseconds 1500
  Check 'a file dropped on it while the folder is away stays on the desktop' (Test-Path -LiteralPath "$PfDesk\年度报告.txt") "on the desktop: $(Test-Path -LiteralPath "$PfDesk\年度报告.txt")"
  Check 'the fence is empty then (it says 找不到此文件夹。)' ((Count $fid) -eq 0) "count=$(Count $fid)"
  Rename-Item -LiteralPath "$PfRun\资料夹-改名" '资料夹'
  $ms = WaitCount $fid 3 8
  Check 'after the failed drop too, the folder is shown again within 3 s' ($ms -is [int] -and $ms -le 3000) "shown after $ms ms"
  Shot 'B3-back'
  CloseWindows
  PfQuit
}

function PartC() {
  trap { PfTrap $_; continue }
  Case 'C a tab split out while the loose icons spread over the screen'
  # A first start again: 「桌面」 arranged like the desktop, as a new install has it.
  PfFresh -Script (DumpScript)
  Pf settings set language zh-CN | Out-Null
  ClearFences | Out-Null
  Start-Sleep -Seconds 1
  # One loose icon to the far corner: 「桌面」 arranged like the desktop spans the screen then.
  $p = ItemPt 'inbox' '独立的文件'
  DragTo $p[0] $p[1] 1500 760; Start-Sleep -Milliseconds 1200
  $inbox = FenceGet 'inbox'
  Check '「桌面」 is arranged like the desktop and spans the screen' ($inbox.desktopLayout -and $inbox.windowRect.w -gt 1300) "desktopLayout=$($inbox.desktopLayout) windowRect=$($inbox.windowRect.x),$($inbox.windowRect.y),$($inbox.windowRect.w),$($inbox.windowRect.h)"
  Pf fence create --title 工作 --rect '300,200,420,260' | Out-Null
  Pf fence create --title 图片素材 --rect '300,520,300,200' | Out-Null
  Pf fence merge 图片素材 --into 工作 | Out-Null; Start-Sleep -Milliseconds 800
  Pf fence detach 图片素材 | Out-Null; Start-Sleep -Milliseconds 1000
  $w = (FenceGet '工作').rect; $s = (FenceGet '图片素材').rect
  $apart = ($s.x -ge $w.x + $w.w) -or ($s.x + $s.w -le $w.x) -or ($s.y -ge $w.y + $w.h) -or ($s.y + $s.h -le $w.y)
  Check 'the split-out fence does not lie on the window it left' $apart "工作 $($w.x),$($w.y),$($w.w),$($w.h); 图片素材 $($s.x),$($s.y),$($s.w),$($s.h); $(LastLog 'new fence placed')"
  Check 'it is back where it stood before the merge' ("$($s.x),$($s.y),$($s.w),$($s.h)" -eq '300,520,300,200') "图片素材 $($s.x),$($s.y),$($s.w),$($s.h)"
  # Its own place taken: it lands beside the window, a gap apart, tops level.
  Pf fence merge 图片素材 --into 工作 | Out-Null; Start-Sleep -Milliseconds 600
  Pf fence create --title 占位 --rect '300,520,300,200' | Out-Null; Start-Sleep -Milliseconds 600
  Pf fence detach 图片素材 | Out-Null; Start-Sleep -Milliseconds 1000
  $s = (FenceGet '图片素材').rect
  Check 'with that place taken it stands right of the window, tops level' ($s.x -ge $w.x + $w.w -and $s.x -le $w.x + $w.w + 16 -and $s.y -eq $w.y) "工作 $($w.x),$($w.y),$($w.w),$($w.h); 图片素材 $($s.x),$($s.y),$($s.w),$($s.h); $(LastLog 'new fence placed')"
  Shot 'C1-split-beside'
  PfQuit
}

function InkCols($b) { return @([Px]::Ink($b, 0, $b.Height - 1) | Where-Object { $_ -ge 25 }).Count }
function PartD() {
  trap { PfTrap $_; continue }
  Case 'D a tab dragged along the strip hides the names it passes over'
  PfStart -Script (DumpScript) -Lang 'zh-CN'
  Pf settings set theme light | Out-Null
  Pf settings set rollUp.switchTabOnHover false | Out-Null
  ClearFences | Out-Null; SmallInbox
  Pf fence create --title 工作 --rect '60,60,460,280' | Out-Null
  Pf fence create --title 图片与设计稿素材 --rect '60,420,300,200' | Out-Null
  Pf fence create --title 项目资料 --rect '560,420,300,200' | Out-Null
  Pf fence merge 图片与设计稿素材 --into 工作 | Out-Null
  Pf fence merge 项目资料 --into 工作 | Out-Null
  Start-Sleep -Seconds 2
  DeskClick 1100 760; Start-Sleep -Milliseconds 300
  $d = DumpOf '工作'
  [G]::Click((TabX $d 0), (TitleY $d)); Start-Sleep -Milliseconds 800
  ParkMouse 1100 760; Start-Sleep -Milliseconds 500
  $d = DumpOf '工作'
  $x = Sx $d $d.rects[0][0]; $w = [int]$d.rects[0][1]; $y = $d.t + $d.oy + 7
  $rest = Crop 'D0-rest' $x $y $w 22
  $x0 = TabX $d 0; $y0 = TitleY $d
  [G]::MoveTo($x0, $y0); Start-Sleep -Milliseconds 250; MDown; Start-Sleep -Milliseconds 150
  Glide $x0 $y0 ($x0 + 56) $y0 14 30; Start-Sleep -Milliseconds 600
  $held = DumpOf '工作'
  $over = Crop 'D1-dragged-over' ($x + 56) $y $w 22
  Shot 'D2-dragged-over'
  MUp; Start-Sleep -Milliseconds 800
  $a = InkCols $rest; $b = InkCols $over
  Check 'the tab is in the hand over its neighbour''s name' ($held.tabDrag -and ($held.tabs -join ',') -eq ($d.tabs -join ',')) "tab_drag=$($held.tabDrag) strip=$($held.tabs -join ',')"
  Check 'under the dragged tab only its own name is drawn' ($a -gt 8 -and $b -le $a + 4) "columns with ink: at rest $a, over the neighbour's name $b (of $w)"
  PfQuit
}

# Height in px of the white tip box shown for a pointer at ($cx, $cy): the tip opens 12 px right
# of and 20 px below the pointer, and its left padding holds no text. (Windows' OCR reads the
# tip's first line and drops the path under it.)
function TipRows($name, $cx, $cy) {
  $b = Crop $name ($cx + 14) ($cy + 10) 4 80
  $px = [Px]::Pixels($b); $rows = 0
  for ($y = 0; $y -lt 80; $y++) {
    $white = 0
    for ($x = 0; $x -lt 4; $x++) { $c = $px[$y * 4 + $x]; if ((($c -shr 16) -band 255) -gt 235 -and (($c -shr 8) -band 255) -gt 235 -and ($c -band 255) -gt 235) { $white++ } }
    if ($white -eq 4) { $rows++ }
  }
  return $rows
}
function PartE() {
  trap { PfTrap $_; continue }
  Case 'E the tips of the title''s folder and up arrow name the folder shown'
  # Latin names and the English interface: the tips are read with Windows' OCR.
  $root = "$PfRun\TipFolder"
  New-Item -ItemType Directory -Force "$root\Inner" | Out-Null
  Set-Content -LiteralPath "$root\plan.txt" 'x'
  PfStart -Script (DumpScript) -Lang 'en'
  Pf settings set theme light | Out-Null
  ClearFences | Out-Null; SmallInbox
  $fid = (PfJson fence create --portal $root --rect '60,200,480,300').fence.id
  Pf fence rename $fid 'My things' | Out-Null
  Start-Sleep -Seconds 2
  DeskClick 1100 760; Start-Sleep -Milliseconds 300
  [G]::MoveTo(84, 218); Start-Sleep -Milliseconds 1800
  $tips = OcrText 50 224 460 70 'E1-root-tip'; $rows = TipRows 'E1-root-tip-rows' 84 218
  Check 'at the root the tip of the folder in the title has a second line, the folder''s path' ($tips -match 'Open folder' -and $rows -ge 32) "tip box $rows px tall (one line: about 20, two: about 36); OCR reads: $tips"
  ParkMouse 1100 760; Start-Sleep -Milliseconds 400
  Pf fence open-folder $fid Inner | Out-Null; Start-Sleep -Milliseconds 900
  # (Another point than before: a move to the very point of the last one arms no tip.)
  [G]::MoveTo(80, 216); Start-Sleep -Milliseconds 300; [G]::MoveTo(86, 220); Start-Sleep -Milliseconds 1800
  $tips = OcrText 50 224 460 70 'E2-sub-tip'; $rows = TipRows 'E2-sub-tip-rows' 86 220
  Check 'in a subfolder the tip of the up arrow has it too' ($tips -match 'Up' -and $rows -ge 32) "tip box $rows px tall (one line: about 20, two: about 36); OCR reads: $tips"
  ParkMouse 1100 760
  PfQuit
}

Case 'setup'
CloseWindows
GreyWallpaper
$made = @()
foreach ($n in '年度报告.txt', '设计稿-最终版.txt', '独立的文件.txt') { $made += (DeskFile $n) }
if ($Part -match 'A') { PartA }
if ($Part -match 'B') { PartB }
if ($Part -match 'C') { PartC }
if ($Part -match 'D') { PartD }
if ($Part -match 'E') { PartE }

Case 'end'
if (PfAlive) { PfQuit }
foreach ($f in $made) { Remove-Item -LiteralPath $f -ErrorAction SilentlyContinue }
