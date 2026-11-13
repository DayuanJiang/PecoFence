# Owner round 2026-11-12, second batch: tabs and folder fences (package hunt-tabs).
# One part per fix; `run.sh --var Part=A` runs one part, no Part runs them all.
#   A  Ctrl+Z takes back a merge by title drag and a tab dragged out
#   B  a folder fence whose folder goes away: the folder in the title, and the folder coming back
#   C  a tab split out lands off the window it left while 「桌面」 is arranged like the desktop
#   D  a tab dragged along the strip hides the names it passes over
#   E  the tips of the title's folder and up arrow name the folder shown
#   F  into a subfolder: the title fades in with the icons
#   G  a folder renamed away is noticed when the pointer comes onto its fence
#   H  Ctrl+Z after the first tab was dragged out, and after a tab was dragged into another window
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'ABCDEFGH' }

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
  # The tab menu's 合并到…: one step as well.
  $s = DumpOf '独立'
  [G]::RClick((Sx $s 150), (TitleY $s)); $m = MenuWin 4
  $q = MenuPoint $m '标签页'; [G]::MoveTo($q[0], $q[1]); Start-Sleep -Milliseconds 800
  $sub = @([W]::Tops('#32768')) | Where-Object { $_ -ne $m } | Select-Object -First 1
  $q = MenuPoint $sub '合并到'; [G]::MoveTo($q[0], $q[1]); Start-Sleep -Milliseconds 800
  $third = @([W]::Tops('#32768')) | Where-Object { $_ -ne $m -and $_ -ne $sub } | Select-Object -First 1
  MenuPick '^工作$' $third | Out-Null; Start-Sleep -Milliseconds 1000
  Check 'the tab menu''s 合并到… merges 独立 into 工作' ((HostOf '独立') -eq $work) "host of 独立=$(HostOf '独立')"
  Chord 0x11 0x5A; Start-Sleep -Milliseconds 1200
  $f = FenceGet '独立'
  Check 'Ctrl+Z takes that merge back too, to where 独立 stood' (-not $f.tabHost -and (RectText $f) -eq '700,60,300,200') "host=$($f.tabHost) rect=$(RectText $f); $(LastLog 'undo: undone|nothing to undo')"
  CloseMenus
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

function PartF() {
  trap { PfTrap $_; continue }
  Case 'F into a subfolder: the title fades in with the icons'
  . C:\pf\kit\rec.ps1
  $root = "$PfRun\标题文件夹"
  New-Item -ItemType Directory -Force "$root\子文件夹甲乙丙丁" | Out-Null
  Set-Content -LiteralPath "$root\计划书.txt" 'x'; Set-Content -LiteralPath "$root\子文件夹甲乙丙丁\里面的文件.txt" 'x'
  $animWas = [Anim]::Get(); [Anim]::Set($true)
  PfStart -Script (DumpScript) -Lang 'zh-CN'
  Pf settings set theme light | Out-Null
  ClearFences | Out-Null; SmallInbox
  $fid = (PfJson fence create --portal $root --rect '60,60,480,330').fence.id
  Pf fence rename $fid 资料 | Out-Null
  Start-Sleep -Seconds 2
  DeskClick 1100 760; Start-Sleep -Milliseconds 300
  $p = ItemAt $fid '子文件夹甲乙丙丁'
  RecStart 'F-title' -X 60 -Y 60 -W 260 -H 36 -Fps 30
  Start-Sleep -Milliseconds 300
  DblClick $p[0] $p[1]; Start-Sleep -Milliseconds 900
  RecStop
  [Anim]::Set($animWas)
  # The ink of the title in every frame, in the columns that only the longer new name reaches.
  $profs = @()
  foreach ($f in (Get-ChildItem "$PfShots\rec-F-title" -Filter *.jpg | Sort-Object Name)) {
    $b = [Drawing.Bitmap]::FromFile($f.FullName); $profs += , ([Px]::Ink($b, 6, 30)); $b.Dispose()
  }
  $endOf = { param($pr) $e = 0; for ($i = 36; $i -lt 260; $i++) { if ($pr[$i] -ge 60) { $e = $i } }; $e }
  $xa = & $endOf $profs[0]; $x1 = & $endOf $profs[-1]
  $vals = @($profs | ForEach-Object { $s = 0; for ($i = $xa + 4; $i -le $x1; $i++) { $s += $_[$i] }; $s })
  $full = $vals[-1]
  $partial = @($vals | Where-Object { $_ -gt 0.15 * $full -and $_ -lt 0.85 * $full }).Count
  Check 'the fence went into the subfolder and its title is the longer name' ($x1 -gt $xa + 20 -and $full -gt 500) "old name ends at $xa px, new at $x1 px, ink of the new part $full"
  Check 'the new name fades in (frames that show it part-way)' ($partial -ge 1 -and $partial -le 6) "frames part-way: $partial of $($vals.Count); ink of the new part per frame: $(($vals | ForEach-Object { [int](100 * $_ / [math]::Max(1, $full)) }) -join ' ') %"
  PfQuit
}

function PartG() {
  trap { PfTrap $_; continue }
  Case 'G a folder renamed away is noticed when the pointer comes onto its fence'
  $root = "$PfRun\指针文件夹"
  New-Item -ItemType Directory -Force $root | Out-Null
  foreach ($n in '计划书.txt', '预算表.txt') { Set-Content -LiteralPath "$root\$n" 'x' }
  PfStart -Script (DumpScript) -Lang 'zh-CN'
  ClearFences | Out-Null; SmallInbox
  $fid = (PfJson fence create --portal $root --rect '60,60,480,330').fence.id
  Pf fence create --title 普通 --rect '620,60,300,220' | Out-Null
  Pf item move "$PfDesk\年度报告.txt" --to 普通 | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse 1100 760; Start-Sleep -Milliseconds 300
  Rename-Item -LiteralPath $root '指针文件夹-改名'; Start-Sleep -Milliseconds 1200
  $before = Count $fid
  [G]::MoveTo(300, 250); Start-Sleep -Milliseconds 150; [G]::MoveTo(310, 256); Start-Sleep -Milliseconds 900
  $after = Count $fid
  Check 'the pointer on the fence: it says that the folder is gone' ($before -eq 2 -and $after -eq 0) "icons before the pointer came: $before, after: $after; $(LastLog 'no longer at its path')"
  Shot 'G1-pointer-in'
  # It takes no drop now: a file dragged onto it stays, and no move is tried.
  $fails = @(PfLogAll | Select-String -Pattern 'move into folder failed').Count
  $p = ItemAt 普通 '年度报告'
  DragTo $p[0] $p[1] 300 250; Start-Sleep -Milliseconds 1200
  $fails2 = @(PfLogAll | Select-String -Pattern 'move into folder failed').Count
  Check 'a file dragged onto the fence that says so is refused (no move is tried)' ((Test-Path -LiteralPath "$PfDesk\年度报告.txt") -and $fails2 -eq $fails) "on the desktop: $(Test-Path -LiteralPath "$PfDesk\年度报告.txt"); failed moves in the log before $fails, after $fails2"
  Rename-Item -LiteralPath "$PfRun\指针文件夹-改名" '指针文件夹'
  $ms = WaitCount $fid 2 8
  Check 'the folder is shown again within 3 s of its return' ($ms -is [int] -and $ms -le 3000) "shown after $ms ms"
  CloseWindows
  PfQuit
}

# Every fence but 「桌面」 with its window, tabs, items and place, as one line to compare.
function TabState($tag) {
  $rows = @()
  foreach ($f in @(PfJson fence list)) {
    if (-not $f -or -not $f.id -or $f.kind -eq 'inbox') { continue }
    $h = if ($f.tabHost) { $IdTitle["$($f.tabHost)"] } else { '-' }
    $rows += "$($f.title)[host=$h tabs=$(@($f.tabs).Count) n=$($f.itemCount) rect=$(RectText $f)]"
  }
  Log "STATE $tag :: $($rows -join ' ; ')"
  return ($rows -join ' ; ')
}
function PartH() {
  trap { PfTrap $_; continue }
  Case 'H Ctrl+Z after the first tab was dragged out, and after a tab went into another window'
  PfStart -Script (DumpScript) -Lang 'zh-CN'
  Pf settings set rollUp.switchTabOnHover false | Out-Null
  ClearFences | Out-Null; SmallInbox
  Pf fence create --title 工作 --rect '60,60,420,260' | Out-Null
  Pf fence create --title 图片素材 --rect '60,400,300,200' | Out-Null
  Pf fence create --title 第三个 --rect '60,400,300,200' | Out-Null
  Pf fence create --title 独立 --rect '700,60,300,200' | Out-Null
  Pf item move "$PfDesk\年度报告.txt" --to 工作 | Out-Null
  Pf item move "$PfDesk\设计稿-最终版.txt" --to 图片素材 | Out-Null
  Pf item move "$PfDesk\独立的文件.txt" --to 独立 | Out-Null
  Pf fence merge 图片素材 --into 工作 | Out-Null
  Pf fence merge 第三个 --into 工作 | Out-Null
  Start-Sleep -Seconds 2
  FenceIds
  DeskClick 1100 760; Start-Sleep -Milliseconds 400
  $s0 = TabState 'start'
  # The first tab is the fence that owns the window: another tab takes the window over.
  $d = DumpOf '工作'
  $x0 = TabX $d 0; $y0 = TitleY $d
  [G]::MoveTo($x0, $y0); Start-Sleep -Milliseconds 250; MDown; Start-Sleep -Milliseconds 120
  Glide $x0 $y0 ($x0 + 420) ($y0 + 420) 16 30; Start-Sleep -Milliseconds 500; MUp; Start-Sleep -Milliseconds 900
  $s1 = TabState 'first tab out'
  Chord 0x11 0x5A; Start-Sleep -Milliseconds 1200
  $s2 = TabState 'after Ctrl+Z'
  Check 'the first tab dragged out is a fence of its own' ($s1 -ne $s0 -and $s1 -match '工作\[host=- tabs=0') $s1
  Check 'Ctrl+Z puts every fence back as it was' ($s2 -eq $s0) "now: $s2 | at the start: $s0 | $(LastLog 'undo: undone|nothing to undo')"
  $wins = @(PfDump).Count
  Check 'three windows again (工作 with its tabs, 独立, 桌面)' ($wins -eq 3) "windows=$wins"
  # One drag takes a tab out of 工作 and drops it on the title of 独立.
  $d = DumpOf '工作'; $t = DumpOf '独立'
  $i = [array]::IndexOf(@($d.tabs), '第三个')
  $x0 = TabX $d $i; $y0 = TitleY $d
  [G]::MoveTo($x0, $y0); Start-Sleep -Milliseconds 250; MDown; Start-Sleep -Milliseconds 120
  Glide $x0 $y0 ($x0 + 60) ($y0 + 80) 8 30; Start-Sleep -Milliseconds 300
  Glide ($x0 + 60) ($y0 + 80) (Sx $t 200) (TitleY $t) 16 30; Start-Sleep -Milliseconds 600; MUp; Start-Sleep -Milliseconds 900
  $s3 = TabState 'tab moved to 独立'
  Chord 0x11 0x5A; Start-Sleep -Milliseconds 1200
  $s4 = TabState 'after Ctrl+Z'
  Check 'a tab dragged onto another fence''s title is a tab there' ($s3 -match '第三个\[host=独立') $s3
  Check 'Ctrl+Z brings it back to its first window, all as before' ($s4 -eq $s0) "now: $s4 | at the start: $s0 | $(LastLog 'undo: undone|nothing to undo')"
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
if ($Part -match 'F') { PartF }
if ($Part -match 'G') { PartG }
if ($Part -match 'H') { PartH }

Case 'end'
if (PfAlive) { PfQuit }
foreach ($f in $made) { Remove-Item -LiteralPath $f -ErrorAction SilentlyContinue }
