# Owner round 2026-11-12, three small things users asked for. One part per call:
#   --var Part=HT   the hover tab switch waits as long as 「展开快慢」 says (linux.do 2998063 #146)
#   --var Rec=before|after with Part=HT records the switch with 快 for the before/after animation.
trap { PfTrap $_; continue }
if (-not $Part) { $Part = 'HT' }

# ---------------------------------------------------------------------------------------------
# HT: with 「鼠标悬停时切换标签页」 on, the tab under a resting pointer shows after the delay of
# 「展开快慢」: 快 150 ms, 默认 Windows' hover time (400 ms), 慢 1 s. It was the Windows hover time
# whatever the setting said. `fence get` reports `fit` only for the tab that is shown, which tells
# when the switch happened.
function HtX($d, $i) { return [int]($d.l + $d.ox + $d.rects[$i][0] + $d.rects[$i][1] / 2) }
function HtY($d) { return [int]($d.t + $d.oy + 18) }
function HtShown($t) { return ($null -ne (FenceGet $t).fit) }
# Rests the pointer on the tab $title and returns the ms until that tab shows (-1: not in $max ms).
function HtHoverMs($d, $title, $max = 3000) {
  $i = [array]::IndexOf(@($d.tabs), $title)
  $x = HtX $d $i; $y = HtY $d
  [G]::MoveTo($x - 6, $y + 2); Start-Sleep -Milliseconds 30
  $sw = [Diagnostics.Stopwatch]::StartNew()
  [G]::MoveTo($x, $y)
  while ($sw.ElapsedMilliseconds -lt $max) { if (HtShown $title) { return [int]$sw.ElapsedMilliseconds } }
  return -1
}
function PartHT {
  trap { PfTrap $_; continue }
  Case 'HT setup: one fence with two tabs'
  CloseWindows
  PfStart -Script (DumpScript 250 10)
  Start-Sleep -Seconds 3
  ClearFences
  Pf fence create --title '常用' --rect '300,200,460,260' | Out-Null
  Pf fence create --title '项目资料与归档' --rect '300,520,300,200' | Out-Null
  Start-Sleep -Seconds 1
  Pf fence merge '项目资料与归档' --into '常用' | Out-Null
  Pf settings set rollUp.switchTabOnHover true | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse 1560 820
  $d = DumpOf '常用'
  Log "dump: $($d.raw)"
  $one = '常用'; $two = '项目资料与归档'
  # How long one `fence get` takes here (the resolution of the times below).
  $sw = [Diagnostics.Stopwatch]::StartNew(); 1..5 | ForEach-Object { $null = HtShown $one }; $per = [int]($sw.ElapsedMilliseconds / 5)
  # Right after the merge both tabs still report a `fit`; one switch each way settles it.
  $null = HtHoverMs $d $two; ParkMouse 1560 820; Start-Sleep -Milliseconds 800
  $null = HtHoverMs $d $one; ParkMouse 1560 820; Start-Sleep -Milliseconds 800
  $null = HtHoverMs $d $two; ParkMouse 1560 820; Start-Sleep -Milliseconds 800
  Check 'HT two tabs, the second one shown after resting on it' ($d.ntabs -eq 2 -and (HtShown $two) -and -not (HtShown $one)) "tabs=$($d.tabs -join ' | '); one fence get takes about $per ms"

  Case 'HT the hover switch waits as long as 展开快慢 says'
  $times = @{}
  foreach ($speed in 'fast', 'normal', 'slow') {
    Pf settings set rollUp.hoverSpeed $speed | Out-Null
    Start-Sleep -Seconds 1
    $ms = @()
    foreach ($k in 1..3) {
      ParkMouse 1560 820; Start-Sleep -Milliseconds 800
      $ms += HtHoverMs $d $one
      ParkMouse 1560 820; Start-Sleep -Milliseconds 800
      $ms += HtHoverMs $d $two
    }
    $times[$speed] = $ms
    Log "hover switch with hoverSpeed=${speed}: $($ms -join ', ') ms"
  }
  $f = $times['fast']; $n = $times['normal']; $s = $times['slow']
  $all = "快: $($f -join ', ') ms; 默认: $($n -join ', ') ms; 慢: $($s -join ', ') ms (one fence get about $per ms)"
  Check 'HT 快: the tab shows 150 ms after the pointer rests on it (100 to 280 ms measured)' (@($f | Where-Object { $_ -lt 100 -or $_ -gt 280 }).Count -eq 0) $all
  Check 'HT 默认: the Windows hover time, 400 ms (300 to 520 ms measured)' (@($n | Where-Object { $_ -lt 300 -or $_ -gt 520 }).Count -eq 0) $all
  Check 'HT 慢: 1 s (900 to 1250 ms measured)' (@($s | Where-Object { $_ -lt 900 -or $_ -gt 1250 }).Count -eq 0) $all

  Case 'HT what must not change'
  # Passing over a tab without resting there switches nothing, also with 快.
  Pf settings set rollUp.hoverSpeed fast | Out-Null; Start-Sleep -Seconds 1
  ParkMouse 1560 820; Start-Sleep -Milliseconds 600
  $x = HtX $d 0; $y = HtY $d
  [G]::MoveTo($x - 6, $y + 2); Start-Sleep -Milliseconds 30; [G]::MoveTo($x, $y); Start-Sleep -Milliseconds 60
  ParkMouse 1560 820; Start-Sleep -Milliseconds 700
  Check 'HT 快: a pointer that leaves the tab after 60 ms switches nothing' ((HtShown $two) -and -not (HtShown $one)) "shown: one=$(HtShown $one) two=$(HtShown $two)"
  # With the switch off a resting pointer switches nothing.
  Pf settings set rollUp.switchTabOnHover false | Out-Null; Start-Sleep -Seconds 1
  $off = HtHoverMs $d $one 1500
  Check 'HT 「鼠标悬停时切换标签页」 off: resting on a tab for 1.5 s switches nothing' ($off -eq -1 -and (HtShown $two)) "ms=$off"
  ParkMouse 1560 820
  # A click still switches at once.
  [G]::Click((HtX $d 0), (HtY $d)); Start-Sleep -Milliseconds 600
  Check 'HT a click on the tab still switches' (HtShown $one) "shown: one=$(HtShown $one)"
  ParkMouse 1560 820

  if ($Rec) {
    Case "HT recording ($Rec)"
    . C:\pf\kit\rec.ps1
    $animWas = [Anim]::Get(); [Anim]::Set($true)
    Pf settings set rollUp.switchTabOnHover true | Out-Null
    Pf settings set rollUp.hoverSpeed fast | Out-Null; Start-Sleep -Seconds 1
    $null = HtHoverMs $d $two; ParkMouse 1560 820; Start-Sleep -Milliseconds 800
    RecStart "HT-$Rec" -X 260 -Y 170 -W 540 -H 330 -Fps 30
    RecMark '「展开快慢」选了「快」。鼠标停到另一个标签上'
    Start-Sleep -Milliseconds 400
    $ms = HtHoverMs $d $one
    RecMetric $ms
    RecMark "标签页切换过来了，等了 $ms 毫秒"
    Start-Sleep -Milliseconds 1200
    RecStop
    [Anim]::Set($animWas)
    Check "HT recorded the switch with 快 ($Rec)" ($ms -gt 0) "ms=$ms"
    ParkMouse 1560 820
  }
  Pf settings set rollUp.hoverSpeed normal | Out-Null
  Pf settings set rollUp.switchTabOnHover false | Out-Null
}
if ($Part -eq 'HT') { PartHT }
