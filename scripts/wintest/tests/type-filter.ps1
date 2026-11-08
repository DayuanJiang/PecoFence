# The type filter (needs Everything: smart.ps1). The Peek search's row of type tabs (全部 文件夹
# 文档 图片 视频 音乐 压缩包 程序, Windows 11 Start search): a click, Ctrl+Tab and Ctrl+Shift+Tab
# pick one, the fences' items and Everything's results are only of that type, and
# 「把这次搜索放到桌面」 takes it along as the smart fence's 类型 condition. The 筛选 box's 类型 chip
# (Finder): a text naming a type offers it, Tab takes it, kind: words become one, Backspace in
# an empty box takes it back, the kept tag shows the type and the text apart, and
# 把筛选存为智能栅栏 keeps it as an extension condition. Chinese UI.
$Dl = '最近 7 天下载'; $Fold = '下载'
function Laid() { return @(PfLogAll | Where-Object { $_ -match 'search list laid out' -and $_ -notmatch 'omitted' }) | Select-Object -Last 1 }
function Results($line) { if ($line -match 'results=(\d+)') { return [int]$Matches[1] }; return -1 }
function SearchWin() { return [G]::FindWindowW('PecoFence.PeekSearch', [NullString]::Value) }
# Clicks type tab $i (0 全部) where the search window last drew it.
function ClickTab($i) {
  $line = @(PfLogAll | Where-Object { $_ -match 'search tabs drawn' -and $_ -notmatch 'omitted' }) | Select-Object -Last 1
  # Left and right of every tab, in a row (PowerShell would flatten pairs).
  $edges = @([regex]::Matches($line, '\((\d+), (\d+)\)') | ForEach-Object { [int]$_.Groups[1].Value; [int]$_.Groups[2].Value })
  $r = Rect (SearchWin)
  if ($edges.Count -lt 2 * $i + 2) { Log "no tab $i in '$line'"; return }
  [G]::Click([int]($r.L + ($edges[2 * $i] + $edges[2 * $i + 1]) / 2), [int]($r.T + 56 + 19)); Start-Sleep -Milliseconds 1500
}
function FenceCrop($t, $name, $m = 10) { $r = (FenceGet $t).windowRect; return (Crop $name ([int]$r.x - $m) ([int]$r.y - $m) ([int]$r.w + 2 * $m) ([int]$r.h + 2 * $m)) }
function WaitTab($name, $sec = 6) {
  for ($i = 0; $i -lt $sec * 4; $i++) { $l = Laid; if ($l -match "tab=$name") { return $l }; Start-Sleep -Milliseconds 250 }
  return (Laid)
}

Case 'setup'
CloseWindows
ClearDownloads
New-Item -ItemType Directory -Force $PfDl | Out-Null
AgedFile $PfDl '季度报告.pdf' 0.5 | Out-Null
AgedFile $PfDl '季度海报.png' 0.8 | Out-Null
AgedFile $PfDl 'notes.txt' 1 | Out-Null
AgedFile $PfDocs '季度计划.docx' 8 | Out-Null
Check 'Everything runs' (EvStart) ''
PfStart -Lang zh-CN -Script (DumpScript 500 15)
Pf settings set theme light | Out-Null
Pf settings set featureTips false | Out-Null
Pf settings set tools.everything true | Out-Null
Pf fence create --smart downloads --rect '24,40,420,300' | Out-Null
Pf fence create --portal $PfDl --title $Fold --rect '24,360,420,300' | Out-Null
Check 'the smart fence is filled' (WaitSmart $Dl { $s.shown -ge 3 } 30) (SmartLine $Dl)
[G]::MoveTo(1250, 780); Start-Sleep -Seconds 2

Case 'Peek type tabs: a click, Ctrl+Tab, Ctrl+Shift+Tab'
Pf peek start --search '季度' | Out-Null; Start-Sleep -Seconds 3
$all = WaitTab '全部'
Shot 't01-all'
Check '全部: the report and the poster in both fences, the plan from Everything (5)' ((Results $all) -eq 5) "$all"
ClickTab 2
$doc = WaitTab '文档'
Shot 't02-documents'
$pick = @(PfLogAll | Where-Object { $_ -match 'type tab picked' -and $_ -match 'by=click' }).Count
Check 'a click on 文档 picks it: the report twice and the plan, no poster (3)' ((Results $doc) -eq 3 -and $pick -ge 1) "picked=$pick $doc"
Chord 0x11 0x09; $img = WaitTab '图片'
Check 'Ctrl+Tab: 图片, the poster twice (2)' ((Results $img) -eq 2 -and (@(PfLogAll | Where-Object { $_ -match 'type tab picked' -and $_ -match 'by=Ctrl\+Tab' }).Count -ge 1)) "$img"
Shot 't03-images'
[G]::KeyDown(0x11); [G]::KeyDown(0x10); [G]::Press(0x09); [G]::KeyUp(0x10); [G]::KeyUp(0x11)
$back = WaitTab '文档'
Check 'Ctrl+Shift+Tab: back to 文档 (3)' ((Results $back) -eq 3 -and (@(PfLogAll | Where-Object { $_ -match 'by=Ctrl\+Shift\+Tab' }).Count -ge 1)) "$back"
$asked = @(PfLogAll | Where-Object { $_ -match 'Everything asked' -and $_ -match 'ext:' } | Select-Object -Last 1)
Check 'Everything is asked for the type too' ($asked.Count -ge 1) "$asked"

Case '把这次搜索放到桌面 takes the tab along'
$before = @(@(PfJson fence list) | Where-Object { $_ -and $_.smart }).Count
$r = Rect (SearchWin)
[G]::Click([int]($r.L + 200), [int]($r.B - 66)); Start-Sleep -Seconds 3
$made = @(@(PfJson fence list) | Where-Object { $_ -and $_.smart -and $_.title -match '季度' })
$spec = if ($made.Count) { (FenceGet $made[0].title).smart.spec } else { $null }
Check 'a smart fence 搜索“季度” with the 文档 condition' ($made.Count -eq 1 -and (@($spec.allOf) | Where-Object { $_.cond -eq 'type' -and @($_.value) -contains 'documents' })) ($spec | ConvertTo-Json -Compress -Depth 5)
if ($made.Count) { Pf fence delete $made[0].title | Out-Null }

Case 'the 类型 chip in the filter box'
FocusFence $Fold
Chord 0x11 0x46; Start-Sleep -Milliseconds 800
[G]::Type('pdf'); Start-Sleep -Milliseconds 900
$offer = @(PfLogAll | Where-Object { $_ -match 'the text names a type' -and $_ -match 'kind:pdf' }).Count
Check 'pdf offers 类型：PDF' ($offer -ge 1) "lines=$offer"
FenceCrop $Fold 'f01-offer' 10 | Out-Null
[G]::Press(0x09); Start-Sleep -Milliseconds 900
Check 'Tab takes it: only the PDF' ((FilterOf $Fold).text -eq 'kind:pdf' -and (WaitFilterShown $Fold 1)) ((FilterOf $Fold) | ConvertTo-Json -Compress)
FenceCrop $Fold 'f02-chip' 10 | Out-Null
[G]::Type('jd'); Start-Sleep -Milliseconds 900
Check 'the chip and pinyin initials together: 季度报告.pdf' ((FilterOf $Fold).text -eq 'kind:pdf jd' -and (WaitFilterShown $Fold 1)) ((FilterOf $Fold) | ConvertTo-Json -Compress)
[G]::Press(0x08); [G]::Press(0x08); Start-Sleep -Milliseconds 500; [G]::Press(0x08); Start-Sleep -Milliseconds 900
Check 'Backspace in the empty box takes the chip back: every file' ((FilterOf $Fold) -eq $null -or -not (FilterOf $Fold).text) ((FilterOf $Fold) | ConvertTo-Json -Compress)
[G]::Type('kind:png '); Start-Sleep -Milliseconds 900
Check 'kind:png becomes the chip: the poster' ((FilterOf $Fold).text -eq 'kind:png' -and (WaitFilterShown $Fold 1)) ((FilterOf $Fold) | ConvertTo-Json -Compress)
[G]::Press(0x0D); Start-Sleep -Milliseconds 900
[G]::MoveTo(1250, 780); Start-Sleep -Milliseconds 500
$w = DumpOf $Fold
Check 'the kept tag: the type on its own, no text, the count' ($w.raw -match 'filter_tag=Some\(\("类型：PNG", "", "1 / 3"\)\)') ([regex]::Match($w.raw, 'filter_tag=\S+ \S+ \S+ \S+ \S+').Value)
FenceCrop $Fold 'f03-kept' 10 | Out-Null

Case '把筛选存为智能栅栏 keeps the type'
$m = FenceMenuOf $Fold
MenuPick '^把筛选存为智能栅栏' $m | Out-Null; Start-Sleep -Seconds 3
$saved = @(@(PfJson fence list) | Where-Object { $_ -and $_.smart -and $_.title -match '类型' })
$spec = if ($saved.Count) { (FenceGet $saved[0].title).smart.spec } else { $null }
Check 'a smart fence 下载：类型：PNG over the folder with the .png condition' ($saved.Count -eq 1 -and (@($spec.allOf) | Where-Object { $_.cond -eq 'ext' -and @($_.value) -contains '.png' })) "$($saved.title) $($spec | ConvertTo-Json -Compress -Depth 5)"

Case 'cleanup'
PfQuit
Remove-Item -LiteralPath (Join-Path $PfDocs '季度计划.docx') -ErrorAction SilentlyContinue
ClearDownloads
