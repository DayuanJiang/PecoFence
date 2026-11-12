# Smart fences and fences per virtual desktop: what the hunt of 2026-11-12 found (Everything is
# installed on the box, see smart.ps1). --var Part=A, B, C or D runs one part (default: all).
#   A  a smart fence that waits for Everything: started outside PecoFence, Everything is found
#      within seconds; a click on 「启动 Everything」 changes the fence at once
#   B  a smart fence whose folder is gone says that the folder cannot be found
#   C  a fence sent to another virtual desktop from its menu: the notice has 撤销
#   D  without Everything a new file shows in a smart fence within half a second
if (-not $Part) { $Part = 'ABCD' }
$HsDir = "$PfDocs\智能栅栏检查"
function HsFiles() {
  Remove-Item -Recurse -Force $HsDir -ErrorAction SilentlyContinue
  AgedFile $HsDir '会议纪要 11月.txt' 1 | Out-Null
  AgedFile $HsDir '年度报告 2025 董事会审阅版（含附录和财务报表）.pdf' 2 | Out-Null
  AgedFile $HsDir 'a.txt' 3 | Out-Null
}
# The words Windows' OCR reads in a fence (English interface: the box has no Chinese OCR).
function FenceWords($title, $name) { $r = (FenceGet $title).windowRect; return (OcrText $r.x $r.y $r.w $r.h $name) }

function PartA {
  trap { PfTrap $_; continue }
  Case 'A: setup'
  CloseWindows
  HsFiles
  EvStop | Out-Null
  PfStart -Lang en -Script (DumpScript 500 20)
  Pf settings set theme light | Out-Null
  ClearFences
  Pf fence create --title 'Whole PC' --everything '会议纪要' --rect '60,60,440,220' | Out-Null
  Check 'A the fence waits for Everything' (WaitSmart 'Whole PC' { $s.source -eq 'needsEverything' } 10) (SmartLine 'Whole PC')
  [G]::MoveTo(1200, 800)

  Case 'A: Everything started outside PecoFence'
  # Well before the first once-a-minute search, which found it on the build before.
  $t0 = Get-Date
  $started = EvStart
  $ok = WaitSmart 'Whole PC' { $s.source -eq 'everything' -and $s.shown -ge 1 } 12
  Check 'A the fence searches within 12 s of Everything starting' ($started -and $ok) "after $([math]::Round(((Get-Date) - $t0).TotalSeconds, 1)) s: $(SmartLine 'Whole PC')"

  Case 'A: a click on Start Everything'
  EvStop | Out-Null
  Pf fence refresh 'Whole PC' | Out-Null
  Check 'A the fence waits for Everything again' (WaitSmart 'Whole PC' { $s.source -eq 'needsEverything' } 10) (SmartLine 'Whole PC')
  Start-Sleep -Milliseconds 500
  $r = (FenceGet 'Whole PC').windowRect
  $row = OcrFind 'Start Everything' $r.x $r.y $r.w $r.h
  Check 'A the row Start Everything is under the sentence' ($null -ne $row) "row=$row words='$(FenceWords 'Whole PC' 'a-before')'"
  [G]::Click($row[0], $row[1]); Start-Sleep -Milliseconds 450
  $words = FenceWords 'Whole PC' 'a-after-click'
  Shot 'a-after-click'
  Check 'A half a second after the click the row is gone (Searching, or the results)' ($words -notmatch 'Start Everything') "words='$words'"
  Check 'A the fence searches once Everything runs' (WaitSmart 'Whole PC' { $s.source -eq 'everything' -and $s.shown -ge 1 } 15) (SmartLine 'Whole PC')
  PfQuit
  Remove-Item -Recurse -Force $HsDir -ErrorAction SilentlyContinue
}
if ($Part -match 'A') { PartA }
