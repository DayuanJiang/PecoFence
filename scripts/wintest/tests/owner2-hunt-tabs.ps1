# Owner round 2026-11-12, second batch: tabs and folder fences (package hunt-tabs).
# One part per fix; `run.sh --var Part=A` runs one part, no Part runs them all.
#   A  Ctrl+Z takes back a merge by title drag and a tab dragged out
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'A' }

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

Case 'setup'
CloseWindows
GreyWallpaper
$made = @()
foreach ($n in '年度报告.txt', '设计稿-最终版.txt', '独立的文件.txt') { $made += (DeskFile $n) }
if ($Part -match 'A') { PartA }

Case 'end'
if (PfAlive) { PfQuit }
foreach ($f in $made) { Remove-Item -LiteralPath $f -ErrorAction SilentlyContinue }
