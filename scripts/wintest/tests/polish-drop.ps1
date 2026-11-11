# Fence polish round 2026-11-12, package "drop": what happens when icons are let go.
#   MC1  a dropped icon arrives from the pointer (167 ms) instead of sliding from its old slot or
#        fading in at its cell (reorder, another fence, several icons, the shell's stack picture)
#   MC6  a new icon whose cell another icon is still sliding out of fades in 83 ms later
#   UX7  icons of another fence dropped between the icons of a 手动 fence show the insertion
#        line and land there (icons, list, details); Ctrl+Z puts them back
#   UX6  a drag that starts in a fence shows the target's text beside the badge ("Move to Bravo")
# Parts (run.sh --var Part=...): L landing and the late fade-in, I insertion, T badge text,
# R the recordings for a before / after clip (not part of the suite). Default: LIT.
# 1600x900 at 100 %; fences have no left tab column (window x + content x are the same).
if (-not $Part) { $Part = 'LIT' }
trap { PfTrap $_; continue }

$PdTxt = @('a1 报告', 'a2 预算表', 'a3 会议纪要 11月', 'a4 照片说明', 'a5 待办', 'b1 发票', 'b2 行程安排', 'b3 清单')
$PdMd = @('01 甲', '02 乙', '04 丁', '05 戊', '06 己', '07 庚')
function PdClean() {
  foreach ($n in $PdTxt) { Remove-Item -LiteralPath (Join-Path $PfDesk "$n.txt") -Force -ErrorAction SilentlyContinue }
  foreach ($n in $PdMd + @('03 丙', '00 新文件')) { Remove-Item -LiteralPath (Join-Path $PfDesk "$n.md") -Force -ErrorAction SilentlyContinue }
  Remove-Item -LiteralPath (Join-Path $PfDesk 'pd Archive') -Recurse -Force -ErrorAction SilentlyContinue
  Remove-Item -LiteralPath (Join-Path $PfDesk '08 末尾.md') -Force -ErrorAction SilentlyContinue
}
function Order($t) { return ((@(PfJson item list --fence $t) | Where-Object { $_ -and $_.name } | ForEach-Object { $_.name }) -join ' | ') }
function FR($t) { return (FenceGet $t).windowRect }
# Centre of an icon by its name, in the order the fence shows (CellPt counts cells).
function ItemPtIn($t, $name) { $i = [array]::IndexOf(@((Order $t) -split ' \| '), $name); if ($i -lt 0) { Log "ItemPtIn: '$name' not in '$t'"; return $null }; return (CellPt $t $i) }
function TabPt($d, $i) { $rc = $d.rects[$i]; return @([int]($d.l + $rc[0] + $rc[1] / 2), [int]($d.t + 18)) }
function P($n) { return (Join-Path $PfDesk "$n.txt") }
# The app log from a position on (the log is open for writing).
function LogPos() { $p = PfLogPath; if (-not (Test-Path $p)) { return 0 }; $fs = [IO.File]::Open($p, 'Open', 'Read', 'ReadWrite'); $n = $fs.Length; $fs.Close(); return $n }
function LogSince($pos, $regex) {
  $p = PfLogPath; if (-not (Test-Path $p)) { return @() }
  $fs = [IO.File]::Open($p, 'Open', 'Read', 'ReadWrite'); $fs.Position = $pos
  $sr = New-Object IO.StreamReader($fs, [Text.Encoding]::UTF8); $buf = $sr.ReadToEnd(); $sr.Close()
  return @($buf -split "`n" | Where-Object { $_ -match $regex })
}
# The icons that arrived from a drop since $pos: name, where each started and where it went
# (screen px; the fence's content starts 36 px under its top at 100 %).
function Landed($pos, $fence) {
  $r = FR $fence
  $out = @()
  foreach ($l in (LogSince $pos 'dropped icon glides')) {
    if ($l -match 'name=(.+?) from_x=(-?[\d.]+) from_y=(-?[\d.]+) to_x=(-?[\d.]+) to_y=(-?[\d.]+)') {
      $out += [pscustomobject]@{ name = $Matches[1]; fx = $r.x + [double]$Matches[2]; fy = $r.y + 36 + [double]$Matches[3]; tx = $r.x + [double]$Matches[4]; ty = $r.y + 36 + [double]$Matches[5] }
    }
  }
  return $out
}
function LandText($l) { return (($l | ForEach-Object { "$($_.name): ($([int]$_.fx),$([int]$_.fy)) -> ($([int]$_.tx),$([int]$_.ty))" }) -join '; ') }
function Near($ax, $ay, $bx, $by, $tol) { return ([math]::Abs($ax - $bx) -le $tol -and [math]::Abs($ay - $by) -le $tol) }
# Press at one point, glide to another and rest there with the button down.
function Hold($x0, $y0, $x1, $y1, $ms = 700) {
  [G]::MoveTo($x0, $y0); Start-Sleep -Milliseconds 200
  MDown; Start-Sleep -Milliseconds 250
  Glide $x0 $y0 $x1 $y1 16 30
  Start-Sleep -Milliseconds $ms
}
function Drop() { MUp; Start-Sleep -Milliseconds 1000 }
function FocusIn($t) { $r = FR $t; [G]::Click($r.x + $r.w - 14, $r.y + $r.h - 14); Start-Sleep -Milliseconds 300 }
# How much the darkest-to-lightest column of a strip differs: an insertion line is a 2 px
# column of its own colour on the plain plate; a wash or nothing gives a flat strip.
function ColumnContrast($x, $y, $w, $h, $name) {
  $b = [Px]::Grab($x, $y, $w, $h); $b.Save("$PfShots\$name.png")
  $cols = @()
  for ($i = 0; $i -lt $w; $i++) { $s = 0; for ($j = 0; $j -lt $h; $j += 3) { $c = $b.GetPixel($i, $j); $s += $c.R + $c.G + $c.B }; $cols += $s / [math]::Ceiling($h / 3) }
  $b.Dispose()
  $m = ($cols | Measure-Object -Minimum -Maximum)
  return [int]($m.Maximum - $m.Minimum)
}
function Layout() {
  ClearFences | Out-Null
  Pf fence create --title Alpha --rect '200,120,340,260' | Out-Null
  Pf fence create --title Bravo --rect '620,120,420,300' | Out-Null
  Pf item move (P 'a1 报告') (P 'a2 预算表') (P 'a3 会议纪要 11月') (P 'a4 照片说明') (P 'a5 待办') --to Alpha | Out-Null
  Pf item move (P 'b1 发票') (P 'b2 行程安排') (P 'b3 清单') --to Bravo | Out-Null
  Pf fence set Alpha sort manual | Out-Null
  Pf fence set Bravo sort manual | Out-Null
  Pf fence set Bravo layout icons | Out-Null
  Start-Sleep -Milliseconds 1500
  ParkMouse 1300 700
  Log "layout: Alpha [$(Order 'Alpha')] Bravo [$(Order 'Bravo')]"
}

Case 'setup'
CloseWindows
PdClean
foreach ($n in $PdTxt) { Set-Content -LiteralPath (P $n) "polish-drop $n" -Encoding UTF8 }
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
PfStart -Script (DumpScript 300 14) -Lang 'zh-CN'
Start-Sleep -Seconds 3
Layout
Check 'setup: five icons in Alpha, three in Bravo' ((Order 'Alpha') -eq 'a1 报告 | a2 预算表 | a3 会议纪要 11月 | a4 照片说明 | a5 待办' -and (Order 'Bravo') -eq 'b1 发票 | b2 行程安排 | b3 清单') "Alpha [$(Order 'Alpha')] Bravo [$(Order 'Bravo')]"

if ($Part -match 'L') {
  Case 'MC1 reorder inside a fence: the icon arrives from the pointer'
  $from = CellPt 'Alpha' 0; $to = CellPt 'Alpha' 5
  $pos = LogPos
  Hold $from[0] $from[1] $to[0] $to[1]
  Drop
  $l = @(Landed $pos 'Alpha'); $end = CellPt 'Alpha' 4
  Check 'MC1 reorder: the icon is the last one now' ((Order 'Alpha') -eq 'a2 预算表 | a3 会议纪要 11月 | a4 照片说明 | a5 待办 | a1 报告') (Order 'Alpha')
  Check 'MC1 reorder: it starts where it was let go' ($l.Count -eq 1 -and (Near $l[0].fx $l[0].fy $to[0] $to[1] 24)) "let go at $($to -join ','); $(LandText $l)"
  Check 'MC1 reorder: and glides to its new cell' ($l.Count -eq 1 -and (Near $l[0].tx $l[0].ty $end[0] $end[1] 24)) "cell at $($end -join ','); $(LandText $l)"

  Case 'MC1 another fence: the icon arrives from the pointer'
  $from = CellPt 'Alpha' 0; $b = FR 'Bravo'; $rx = $b.x + 330; $ry = $b.y + 240
  $pos = LogPos
  Hold $from[0] $from[1] $rx $ry
  Drop
  $l = @(Landed $pos 'Bravo'); $end = CellPt 'Bravo' 3
  Check 'MC1 another fence: the icon is in Bravo, after the others' ((Order 'Bravo') -eq 'b1 发票 | b2 行程安排 | b3 清单 | a2 预算表') (Order 'Bravo')
  Check 'MC1 another fence: it starts where it was let go' ($l.Count -eq 1 -and (Near $l[0].fx $l[0].fy $rx $ry 24)) "let go at $rx,$ry; $(LandText $l)"
  Check 'MC1 another fence: and glides to its cell, over 100 px away' ($l.Count -eq 1 -and (Near $l[0].tx $l[0].ty $end[0] $end[1] 24) -and [math]::Abs($l[0].fy - $l[0].ty) -gt 100) "cell at $($end -join ','); $(LandText $l)"

  Case 'MC1 three icons together keep their places to each other'
  $p0 = CellPt 'Alpha' 0; $p1 = CellPt 'Alpha' 1; $p2 = CellPt 'Alpha' 2
  [G]::Click($p0[0], $p0[1]); Start-Sleep -Milliseconds 200
  [G]::KeyDown(0x11); [G]::Click($p1[0], $p1[1]); Start-Sleep -Milliseconds 150; [G]::Click($p2[0], $p2[1]); [G]::KeyUp(0x11)
  Start-Sleep -Milliseconds 300
  $rx = $b.x + 210; $ry = $b.y + 240
  $pos = LogPos
  Hold $p1[0] $p1[1] $rx $ry
  Drop
  $l = @(Landed $pos 'Bravo' | Sort-Object fx)
  $pitch = $p1[0] - $p0[0]
  Check 'MC1 three icons: all three are in Bravo' ((Order 'Bravo') -eq 'b1 发票 | b2 行程安排 | b3 清单 | a2 预算表 | a3 会议纪要 11月 | a4 照片说明 | a5 待办') (Order 'Bravo')
  Check 'MC1 three icons: each starts under its own picture, a cell apart' ($l.Count -eq 3 -and (Near $l[1].fx $l[1].fy $rx $ry 24) -and [math]::Abs(($l[1].fx - $l[0].fx) - $pitch) -le 3 -and [math]::Abs(($l[2].fx - $l[1].fx) - $pitch) -le 3) "let go at $rx,$ry, pitch $pitch; $(LandText $l)"

  Case 'MC1 seven icons (the stack picture) all start under the pointer'
  FocusIn 'Bravo'
  Chord 0x11 0x41; Start-Sleep -Milliseconds 300
  $p = CellPt 'Bravo' 0; $a = FR 'Alpha'; $rx = $a.x + 170; $ry = $a.y + 200
  $pos = LogPos
  Hold $p[0] $p[1] $rx $ry
  Drop
  $l = @(Landed $pos 'Alpha')
  $sum = @(LogSince $pos 'layout motion: icons arriving from a drop') | Select-Object -Last 1
  $same = @($l | Where-Object { Near $_.fx $_.fy $rx $ry 4 }).Count
  Check 'MC1 seven icons: all are in Alpha' (@((Order 'Alpha') -split ' \| ').Count -eq 8) (Order 'Alpha')
  Check 'MC1 seven icons: they glide although more than half of the fence is new' ("$sum" -match 'landed=7') "$sum"
  Check 'MC1 seven icons: every one starts under the pointer' ($l.Count -ge 4 -and $same -eq $l.Count) "let go at $rx,$ry; $(LandText $l)"

  Case 'MC1 animations off: the icon is there at once'
  [Anim]::Set($false); Start-Sleep -Milliseconds 800
  $p = CellPt 'Alpha' 0; $rx = $b.x + 330; $ry = $b.y + 240
  $pos = LogPos
  Hold $p[0] $p[1] $rx $ry
  Drop
  $l = @(Landed $pos 'Bravo')
  $first = @((Order 'Bravo') -split ' \| ')[0]
  Check 'MC1 animations off: the icon moved and nothing glides' ($first -and $l.Count -eq 0) "Bravo [$(Order 'Bravo')]; landed lines: $($l.Count)"
  [Anim]::Set($true); Start-Sleep -Milliseconds 800

  Case 'MC6 a new icon waits for the icon leaving its cell'
  foreach ($n in $PdMd) { Set-Content -LiteralPath (Join-Path $PfDesk "$n.md") "polish-drop $n" -Encoding UTF8 }
  Pf fence create --title Names --rect '200,440,420,300' | Out-Null
  Start-Sleep -Seconds 2
  Pf item move --glob '*.md' --to Names | Out-Null
  Pf fence set Names sort name | Out-Null
  Pf rule add --name 'md to Names' --ext md --to Names --index 0 | Out-Null
  Start-Sleep -Seconds 2
  $pos = LogPos
  Set-Content -LiteralPath (Join-Path $PfDesk '03 丙.md') 'new' -Encoding UTF8
  $in = $false
  foreach ($k in 1..40) { if ((Order 'Names') -match '03 丙') { $in = $true; break }; Start-Sleep -Milliseconds 200 }
  Start-Sleep -Milliseconds 600
  $sum = @(LogSince $pos 'layout motion: icons arriving from a drop') | Select-Object -Last 1
  Check 'MC6 the new file shows between its neighbours' ($in -and (Order 'Names') -match '02 乙\.md \| 03 丙\.md \| 04 丁\.md') (Order 'Names')
  Check 'MC6 its fade-in starts 83 ms later, the icon before it is still leaving the cell' ("$sum" -match 'waited=1') "$sum"
  $pos = LogPos
  Set-Content -LiteralPath (Join-Path $PfDesk '00 新文件.md') 'new' -Encoding UTF8
  foreach ($k in 1..40) { if ((Order 'Names') -match '00 新文件') { break }; Start-Sleep -Milliseconds 200 }
  Start-Sleep -Milliseconds 600
  $late = @(LogSince $pos 'waited=[1-9]')
  $names = @((Order 'Names') -split ' \| ')
  Check 'MC6 a new first icon waits too (the old first icon leaves its cell)' ($names[0] -eq '00 新文件.md' -and $late.Count -ge 1) "Names [$(Order 'Names')]; $($late | Select-Object -Last 1)"
  Remove-Item -LiteralPath (Join-Path $PfDesk '00 新文件.md'), (Join-Path $PfDesk '03 丙.md') -Force -ErrorAction SilentlyContinue
  Start-Sleep -Seconds 1
  $pos = LogPos
  Set-Content -LiteralPath (Join-Path $PfDesk '08 末尾.md') 'new' -Encoding UTF8
  foreach ($k in 1..40) { if ((Order 'Names') -match '08 末尾') { break }; Start-Sleep -Milliseconds 200 }
  Start-Sleep -Milliseconds 600
  $late = @(LogSince $pos 'waited=[1-9]')
  Check 'MC6 a new last icon, whose cell was empty, fades in at once' ((Order 'Names') -match '08 末尾\.md$' -and $late.Count -eq 0) "Names [$(Order 'Names')]; late lines: $($late.Count)"
  Remove-Item -LiteralPath (Join-Path $PfDesk '08 末尾.md') -Force -ErrorAction SilentlyContinue
}

if ($Part -match 'I') {
  Case 'UX7 setup'
  Pf fence delete Names 2>$null | Out-Null
  Layout
  $b = FR 'Bravo'

  Case 'UX7 icons of another fence show the insertion line'
  $from = CellPt 'Alpha' 0; $c3 = CellPt 'Bravo' 3; $c4 = CellPt 'Bravo' 4
  $pitch = $c4[0] - $c3[0]
  # The line stands on the left edge of the fourth cell (the place after the three icons); the
  # pointer rests a cell further right, so the drag picture does not cover it.
  $lineX = [int]($c3[0] - $pitch / 2)
  $flat = ColumnContrast ($lineX - 10) ($b.y + 60) 20 60 'i-line-before'
  Hold $from[0] $from[1] $c4[0] $c4[1] 900
  $line = ColumnContrast ($lineX - 10) ($b.y + 60) 20 60 'i-line-hold'
  Shot 'i-hold-end'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300; MUp; Start-Sleep -Milliseconds 600
  Check 'UX7 insertion line where the icon will land (a column of its own colour)' ($line -ge 40 -and $flat -lt 20) "column contrast before the drag $flat, while holding $line"
  Check 'UX7 Esc cancels: nothing moved' ((Order 'Bravo') -eq 'b1 发票 | b2 行程安排 | b3 清单') (Order 'Bravo')

  Case 'UX7 dropped between two icons'
  $from = CellPt 'Alpha' 0; $q = CellPt 'Bravo' 1
  Hold $from[0] $from[1] ($q[0] - 30) $q[1]
  Shot 'i-hold-between'
  Drop
  Check 'UX7 the icon lands between b1 and b2' ((Order 'Bravo') -eq 'b1 发票 | a1 报告 | b2 行程安排 | b3 清单') (Order 'Bravo')
  FocusIn 'Bravo'
  Chord 0x11 0x5A; Start-Sleep -Milliseconds 1200
  Check 'UX7 Ctrl+Z puts it back, the others keep their order' ((Order 'Alpha') -match 'a1 报告' -and (Order 'Bravo') -eq 'b1 发票 | b2 行程安排 | b3 清单') "Alpha [$(Order 'Alpha')] Bravo [$(Order 'Bravo')]"

  Case 'UX7 dropped on the middle of an icon'
  $from = ItemPtIn 'Alpha' 'a2 预算表'; $q = CellPt 'Bravo' 1
  Hold $from[0] $from[1] ($q[0] + 6) $q[1]
  Drop
  $names = @((Order 'Bravo') -split ' \| ')
  $at = [array]::IndexOf($names, 'a2 预算表')
  Check 'UX7 the icon lands beside that icon, not at the end' ($names.Count -eq 4 -and ($at -eq 1 -or $at -eq 2)) (Order 'Bravo')

  Case 'UX7 list layout'
  Pf fence set Bravo layout list | Out-Null; Start-Sleep -Milliseconds 800
  # Rows are 28 px high and start 4 px under the title row: let go over the top of the second row.
  $from = ItemPtIn 'Alpha' 'a3 会议纪要 11月'
  Hold $from[0] $from[1] ($b.x + 120) ($b.y + 36 + 4 + 28 + 5)
  Shot 'i-list-hold'
  Drop
  $names = @((Order 'Bravo') -split ' \| ')
  Check 'UX7 list: the icon lands before the second row' ($names.Count -eq 5 -and $names[1] -eq 'a3 会议纪要 11月') (Order 'Bravo')

  Case 'UX7 details layout'
  Pf fence set Bravo layout details | Out-Null; Start-Sleep -Milliseconds 800
  # The header row takes the first 28 px.
  $from = ItemPtIn 'Alpha' 'a4 照片说明'
  Hold $from[0] $from[1] ($b.x + 120) ($b.y + 36 + 28 + 4 + 28 + 5)
  Shot 'i-details-hold'
  Drop
  $names = @((Order 'Bravo') -split ' \| ')
  Check 'UX7 details: the icon lands before the second row' ($names.Count -eq 6 -and $names[1] -eq 'a4 照片说明') (Order 'Bravo')
  Pf fence set Bravo layout icons | Out-Null; Start-Sleep -Milliseconds 800

  Case 'UX7 a sorted fence still sorts, a file arriving by a rule goes to the end'
  Pf fence set Alpha sort name | Out-Null; Start-Sleep -Milliseconds 600
  $p = CellPt 'Bravo' 0; $a = FR 'Alpha'
  $first = @((Order 'Bravo') -split ' \| ')[0]
  Hold $p[0] $p[1] ($a.x + 250) ($a.y + 60)
  Drop
  $sorted = @((Order 'Alpha') -split ' \| ')
  Check 'UX7 a fence sorted by name puts the dropped icon in name order' (($sorted -join '|') -eq ((@($sorted | Sort-Object)) -join '|') -and $sorted -contains $first) "Alpha [$(Order 'Alpha')], dropped $first"
  Pf rule add --name 'md to Bravo' --ext md --to Bravo --index 0 | Out-Null
  Start-Sleep -Seconds 1
  Set-Content -LiteralPath (Join-Path $PfDesk '00 新文件.md') 'new' -Encoding UTF8
  foreach ($k in 1..40) { if ((Order 'Bravo') -match '00 新文件') { break }; Start-Sleep -Milliseconds 200 }
  Check 'UX7 a new file sent by a rule goes after the others' ((Order 'Bravo') -match '\| 00 新文件\.md$') (Order 'Bravo')
  Remove-Item -LiteralPath (Join-Path $PfDesk '00 新文件.md') -Force -ErrorAction SilentlyContinue
}

if ($Part -match 'T') {
  Case 'UX6 setup'
  Pf settings set language en | Out-Null
  Layout
  New-Item -ItemType Directory -Force (Join-Path $PfDesk 'pd Archive') | Out-Null
  Start-Sleep -Seconds 2
  Pf item move (Join-Path $PfDesk 'pd Archive') --to Bravo | Out-Null
  Start-Sleep -Milliseconds 1200
  $b = FR 'Bravo'
  Log "Bravo [$(Order 'Bravo')] window $($b.x),$($b.y),$($b.w),$($b.h)"
  # The text of the drag picture's badge, read beside the pointer.
  function BadgeText($x, $y, $name) { return (OcrText ($x - 30) ($y - 10) 330 70 $name) }

  Case 'UX6 over a fence'
  $from = CellPt 'Alpha' 0
  Hold $from[0] $from[1] ($b.x + 330) ($b.y + 240) 1200
  $t1 = BadgeText ($b.x + 330) ($b.y + 240) 't-fence'
  Shot 't-over-fence'
  Check 'UX6 over a fence the picture says where the icon goes' ($t1 -match 'Move to\s+Bravo') "read: '$t1'"

  Case 'UX6 over a folder icon'
  $f = CellPt 'Bravo' 3
  Glide ($b.x + 330) ($b.y + 240) $f[0] $f[1] 12 30; Start-Sleep -Milliseconds 1200
  $t2 = BadgeText $f[0] $f[1] 't-folder'
  Shot 't-over-folder'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300; MUp; Start-Sleep -Milliseconds 600
  Check 'UX6 over a folder icon it names the folder' ($t2 -match 'Move to\s+pd Archive') "read: '$t2'"

  Case 'UX6 over a tab'
  Pf fence create --title Charlie --rect '1080,120,300,200' | Out-Null
  Start-Sleep -Milliseconds 800
  Pf fence merge Charlie --into Bravo | Out-Null
  Start-Sleep -Milliseconds 1500
  ParkMouse 1300 700
  $d = DumpOf 'Bravo' 4
  # The pill of the tab that is not shown.
  $other = if ("$($d.active)" -eq "$((FenceGet 'Charlie').id)") { 'Bravo' } else { 'Charlie' }
  $i = [array]::IndexOf(@($d.tabs), $other)
  $tab = TabPt $d $i
  Log "tabs: $($d.tabs -join ', ') active $($d.active), pill $i at $($tab -join ',')"
  $from = CellPt 'Alpha' 0
  Hold $from[0] $from[1] $tab[0] $tab[1] 1200
  $t3 = BadgeText $tab[0] $tab[1] 't-tab'
  Shot 't-over-tab'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300; MUp; Start-Sleep -Milliseconds 600
  Check 'UX6 over a tab it names the tab' ($t3 -match "Move to\s+$($d.tabs[$i])") "pill '$($d.tabs[$i])', read: '$t3'"

  Case 'UX6 the stack picture of five icons'
  FocusIn 'Alpha'
  Chord 0x11 0x41; Start-Sleep -Milliseconds 300
  $from = CellPt 'Alpha' 0
  Hold $from[0] $from[1] ($b.x + 330) ($b.y + 240) 1200
  $t4 = BadgeText ($b.x + 330) ($b.y + 240) 't-stack'
  Shot 't-stack'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300; MUp; Start-Sleep -Milliseconds 600
  Check 'UX6 the stack picture of five icons says it too' ($t4 -match 'Move to\s+Bravo') "read: '$t4'"
  Pf settings set language zh-CN | Out-Null
}

if ($Part -match 'R') {
  # Recordings for the before / after clips: the same steps on any build.
  . C:\pf\kit\rec.ps1
  Case 'recordings'
  Pf fence delete Names 2>$null | Out-Null
  Layout
  $b = FR 'Bravo'
  $from = CellPt 'Alpha' 0; $rx = $b.x + 330; $ry = $b.y + 240
  RecStart 'far' -X 190 -Y 110 -W 870 -H 330 -Fps 60
  Start-Sleep -Milliseconds 300
  RecMark 'press'
  [G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 200
  MDown; Start-Sleep -Milliseconds 250
  RecMark 'drag'
  Glide $from[0] $from[1] $rx $ry 24 25
  RecMark 'hold'
  Start-Sleep -Milliseconds 500
  RecMark 'drop'
  MUp
  Start-Sleep -Milliseconds 1200
  RecStop
  $from = CellPt 'Alpha' 0; $to = CellPt 'Alpha' 4
  RecStart 'reorder' -X 190 -Y 110 -W 360 -H 280 -Fps 60
  Start-Sleep -Milliseconds 300
  RecMark 'press'
  [G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 200
  MDown; Start-Sleep -Milliseconds 250
  RecMark 'drag'
  Glide $from[0] $from[1] $to[0] ($to[1] + 20) 24 25
  RecMark 'hold'
  Start-Sleep -Milliseconds 500
  RecMark 'drop'
  MUp
  Start-Sleep -Milliseconds 1200
  RecStop
  # Between two icons of another fence.
  $from = CellPt 'Alpha' 0; $q = CellPt 'Bravo' 1
  RecStart 'between' -X 190 -Y 110 -W 870 -H 330 -Fps 60
  Start-Sleep -Milliseconds 300
  RecMark 'press'
  [G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 200
  MDown; Start-Sleep -Milliseconds 250
  RecMark 'drag'
  Glide $from[0] $from[1] ($q[0] - 30) $q[1] 24 25
  RecMark 'hold'
  Start-Sleep -Milliseconds 900
  RecMark 'drop'
  MUp
  Start-Sleep -Milliseconds 1500
  RecStop
  # A new file in a fence sorted by name.
  foreach ($n in $PdMd) { Set-Content -LiteralPath (Join-Path $PfDesk "$n.md") "polish-drop $n" -Encoding UTF8 }
  Pf fence create --title Names --rect '200,440,420,300' | Out-Null
  Start-Sleep -Seconds 2
  Pf item move --glob '*.md' --to Names | Out-Null
  Pf fence set Names sort name | Out-Null
  Pf rule add --name 'md to Names' --ext md --to Names --index 0 | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse 1300 700
  RecStart 'newfile' -X 190 -Y 430 -W 440 -H 250 -Fps 60
  Start-Sleep -Milliseconds 300
  RecMark 'create'
  Set-Content -LiteralPath (Join-Path $PfDesk '03 丙.md') 'new' -Encoding UTF8
  Start-Sleep -Milliseconds 2500
  RecStop
  Check 'recordings made' ((Test-Path "$PfShots\rec-far\frames.tsv") -and (Test-Path "$PfShots\rec-newfile\frames.tsv")) (Get-Content "$PfShots\rec-far\stats.txt")
}

Case 'cleanup'
[Anim]::Set($AnimWas)
PfQuit
PdClean
$left = @(Get-ChildItem -LiteralPath $PfDesk | Where-Object { $_.Name -match '^(a\d |b\d |0\d |pd )' })
Check 'cleanup: no test file left on the desktop' ($left.Count -eq 0) (($left | ForEach-Object { $_.Name }) -join ', ')
