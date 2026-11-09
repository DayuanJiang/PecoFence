# The type filter (needs Everything: smart.ps1). The Peek search's row of type tabs (全部 文件夹
# 文档 图片 视频 音乐 压缩包 程序, Windows 11 Start search): a click, Ctrl+Tab and Ctrl+Shift+Tab
# pick one, the fences' items and Everything's results are only of that type, and
# 「把这次搜索放到桌面」 takes it along as the smart fence's 类型 condition. The 筛选 box's 类型 chip
# (Finder): a text naming a type offers it, Tab takes it, kind: words become one, Backspace in
# an empty box takes it back, the kept tag shows the type and the text apart, and
# 把筛选存为智能栅栏 keeps it as an extension condition. Chinese UI.
# Review fixes: 「N 个结果」 counts every result, 「在 Everything 里查看全部结果」 keeps the tab, a
# shortcut is a program by its target, kind:docx is not cut at kind:doc, Tab leaves no hidden
# character, a rolled filtered fence shows the tag only, a tab strip names the type, and the
# Ctrl+Tab hint is translated and stays in view in long languages. Re-check: a narrow fence's tag
# gives way part by part and its infotip names the filter; shortcut targets are read once.
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
# The infotip PecoFence shows now (OCR of its window), '' when none is up.
function TipText($name) {
  $tip = @([W]::Tops('tooltips_class32') | Where-Object { [W]::ProcOf($_) -eq 'pecofence' }) | Select-Object -First 1
  if (-not $tip) { return '' }
  $r = Rect $tip
  return (OcrText $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) $name)
}
function HoverTip($x, $y, $name) {
  [G]::MoveTo(1250, 780); Start-Sleep -Milliseconds 400
  [G]::MoveTo($x - 3, $y); Start-Sleep -Milliseconds 120; [G]::MoveTo($x, $y)
  for ($i = 0; $i -lt 10; $i++) { Start-Sleep -Milliseconds 300; $t = TipText $name; if ($t) { return $t } }
  return ''
}
function Reads() { return @(PfLogAll | Where-Object { $_ -match 'type filter: shortcut target read' }).Count }
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

Case 'the total, and 在 Everything 里查看全部结果 keeps the tab'
$Tot = Join-Path $PfDocs 'pf-total'
1..12 | ForEach-Object { AgedFile $Tot ('季度附件{0:D2}.txt' -f $_) 9 | Out-Null }
Start-Sleep -Seconds 2
Pf peek start --search '季度附件' | Out-Null; Start-Sleep -Seconds 3
$l = WaitTab '全部'
$total = if ($l -match 'total=(\d+)') { [int]$Matches[1] } else { -1 }
Check '「N 个结果」 counts all 12 files, more than the rows shown' ($total -eq 12 -and (Results $l) -lt 12) "$l"
Shot 't04-total'
Pf peek end | Out-Null; Start-Sleep -Seconds 1
Pf peek start --search '季度' | Out-Null; Start-Sleep -Seconds 3
WaitTab '全部' | Out-Null
ClickTab 2; WaitTab '文档' | Out-Null
$r = Rect (SearchWin)
[G]::Click([int]($r.L + 200), [int]($r.B - 110)); Start-Sleep -Seconds 3
$shown = @(PfLogAll | Where-Object { $_ -match 'search: shown in Everything' -and $_ -notmatch 'omitted' }) | Select-Object -Last 1
Check '在 Everything 里查看全部结果 on 文档: Everything gets the type words too' ("$shown" -match 'ext:' -and "$shown" -match 'types=1') "$shown"
Get-Process Everything -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero } | ForEach-Object { [L]::PostMessageW($_.MainWindowHandle, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null }
Start-Sleep -Seconds 1

Case 'a shortcut is a program by its target'
$sh = New-Object -ComObject WScript.Shell
# ASCII names: WScript.Shell cannot save a shortcut with Chinese in its path on this box.
$a = $sh.CreateShortcut((Join-Path $PfDl 'pfkj-notes.lnk')); $a.TargetPath = "$env:WINDIR\win.ini"; $a.Save()
$b = $sh.CreateShortcut((Join-Path $PfDl 'pfkj-tool.lnk')); $b.TargetPath = "$env:WINDIR\notepad.exe"; $b.Save()
Start-Sleep -Seconds 3
Pf peek start --search 'pfkj' | Out-Null; Start-Sleep -Seconds 3
$all = WaitTab '全部'
ClickTab 7
$prog = WaitTab '程序'
Check '程序 in the Peek: the shortcut to notepad only, half of 全部' ((Results $all) -ge 2 -and (Results $prog) * 2 -eq (Results $all)) "all=$all prog=$prog"
Pf peek end | Out-Null; Start-Sleep -Seconds 1
FocusFence $Fold; [G]::Press(0x1B); Start-Sleep -Milliseconds 700
Chord 0x11 0x46; Start-Sleep -Milliseconds 800
[G]::Type('kind:programs '); Start-Sleep -Milliseconds 900
Check '程序 in the filter box: the shortcut to notepad only' ((FilterOf $Fold).text -eq 'kind:programs' -and (WaitFilterShown $Fold 1)) ((FilterOf $Fold) | ConvertTo-Json -Compress)
$r0 = Reads
[G]::Type('pfkj'); Start-Sleep -Milliseconds 1200
$r1 = Reads
Check 'typing a name after 程序 reads no shortcut again (cached by path and time)' ($r0 -ge 1 -and $r1 -eq $r0 -and (WaitFilterShown $Fold 1)) "reads before=$r0 after=$r1"
[G]::Press(0x1B); Start-Sleep -Milliseconds 700

Case 'kind:docx is not cut at kind:doc; Tab leaves no character behind'
FocusFence $Fold; Chord 0x11 0x46; Start-Sleep -Milliseconds 800
$n0 = @(PfLogAll | Where-Object { $_ -match 'a typed type word became a chip' -and $_ -notmatch 'omitted' }).Count
[G]::Type('kind:docx'); Start-Sleep -Milliseconds 900
$early = @(PfLogAll | Where-Object { $_ -match 'a typed type word became a chip' -and $_ -notmatch 'omitted' }).Count - $n0
[G]::Type(' '); Start-Sleep -Milliseconds 900
$chip = @(PfLogAll | Where-Object { $_ -match 'a typed type word became a chip' -and $_ -notmatch 'omitted' }) | Select-Object -Last 1
Check 'no chip while typing kind:docx; the space makes it one' ($early -eq 0 -and "$chip" -match 'token=kind:docx') "early=$early $chip"
FenceCrop $Fold 'f04-docx' 10 | Out-Null
[G]::Press(0x08); Start-Sleep -Milliseconds 700
[G]::Type('kind:xlsx'); Start-Sleep -Milliseconds 700; [G]::Press(0x09); Start-Sleep -Milliseconds 800
$fin = @(PfLogAll | Where-Object { $_ -match 'Tab finishes the kind: word' -and $_ -notmatch 'omitted' }).Count
Check 'Tab finishes kind:xlsx into a chip' ($fin -ge 1 -and (FilterOf $Fold).text -eq 'kind:xlsx') "fin=$fin $((FilterOf $Fold) | ConvertTo-Json -Compress)"
[G]::Press(0x08); Start-Sleep -Milliseconds 700
[G]::Type('pdf'); Start-Sleep -Milliseconds 900; [G]::Press(0x09); Start-Sleep -Milliseconds 800
$took = (FilterOf $Fold).text
[G]::Press(0x08); Start-Sleep -Milliseconds 900
$now = FilterOf $Fold
Check 'after Tab took 类型：PDF, one Backspace takes it back' ($took -eq 'kind:pdf' -and ($now -eq $null -or -not $now.text)) "took=$took now=$($now | ConvertTo-Json -Compress)"
[G]::Press(0x1B); Start-Sleep -Milliseconds 700

Case 'a rolled filtered fence shows the tag only'
FocusFence $Fold; Chord 0x11 0x46; Start-Sleep -Milliseconds 800
[G]::Type('kind:png '); Start-Sleep -Milliseconds 700; [G]::Press(0x0D); Start-Sleep -Milliseconds 800
Pf fence roll $Fold | Out-Null; [G]::MoveTo(1250, 780); Start-Sleep -Seconds 2
$w = DumpOf $Fold
Check 'rolled with a filter: the tag, no 「N 项」 beside it' ($w.raw -match 'rolled=true' -and $w.raw -match 'filter_tag=Some' -and $w.raw -match 'rolled_count=false') ([regex]::Match($w.raw, 'rolled=\S+').Value + ' ' + [regex]::Match($w.raw, 'filter_tag=\S+ \S+ \S+').Value + ' ' + [regex]::Match($w.raw, 'rolled_count=\S+').Value)
FenceCrop $Fold 'f05-rolled' 10 | Out-Null
Pf fence unroll $Fold | Out-Null; Start-Sleep -Seconds 1

Case 'a narrow fence: the tag gives way, its infotip names the filter'
Pf fence move $Fold --rect '24,360,150,300' | Out-Null; [G]::MoveTo(1250, 780); Start-Sleep -Seconds 2
$w = DumpOf $Fold
$fit = [regex]::Match("$($w.raw)", 'tag_fit=Some\(TagFit \{ rect: Rect \{ left: ([\d.]+), top: ([\d.]+), right: ([\d.]+), bottom: ([\d.]+) \}, text: "([^"]*)", count: (\w+), kind: (\w+) \}\)')
Check '150 wide: the magnifier and × only, right of the title padding' ($fit.Success -and [double]$fit.Groups[1].Value -ge 16 -and $fit.Groups[6].Value -eq 'false' -and $fit.Groups[7].Value -eq 'false') ([regex]::Match("$($w.raw)", 'tag_fit=.*$').Value)
FenceCrop $Fold 'f06-narrow' 10 | Out-Null
if ($fit.Success) {
  $x = [int]($w.l + $w.ox + [double]$fit.Groups[1].Value + 10)
  $y = [int]($w.t + $w.oy + ([double]$fit.Groups[2].Value + [double]$fit.Groups[4].Value) / 2)
  $tip = HoverTip $x $y 'tag-tip'
  Check 'its infotip names the whole filter' ($tip -match 'PNG') "tip='$tip'"
  Shot 't06-tag-tip'
}
Pf fence move $Fold --rect '24,360,420,300' | Out-Null; [G]::MoveTo(1250, 780); Start-Sleep -Seconds 1
$w = DumpOf $Fold
Check '420 wide again: the whole tag' ("$($w.raw)" -match 'tag_fit=Some\(TagFit \{ rect: Rect \{ left: [\d.]+, top: [\d.]+, right: [\d.]+, bottom: [\d.]+ \}, text: "", count: true, kind: true') ([regex]::Match("$($w.raw)", 'tag_fit=.*$').Value)

Case 'a tab strip names the type'
Pf fence create --portal $PfDocs --title '文档' --rect '460,360,420,300' | Out-Null; Start-Sleep -Seconds 2
# 下载 hosts the strip and stays the shown tab: its title carries the filter.
Pf fence merge '文档' --into $Fold | Out-Null; Start-Sleep -Seconds 2
# The merged tab is shown: a click on the 下载 pill shows the filtered host again.
$wr = (FenceGet $Fold).windowRect; [G]::Click([int]$wr.x + 40, [int]$wr.y + 17); Start-Sleep -Milliseconds 900
[G]::MoveTo(1250, 780); Start-Sleep -Seconds 1
$w = DumpOf $Fold
Check 'the strip: 下载 · 筛选：类型：PNG' ($w -and $w.raw -match 'title="下载 · 筛选：类型：PNG"') ([regex]::Match("$($w.raw)", 'title="[^"]*"').Value + ' ' + [regex]::Match("$($w.raw)", 'tab_fonts=\[[^\]]*\]').Value)
FenceCrop $Fold 'f07-strip' 10 | Out-Null

Case 'the Ctrl+Tab hint stays in view in long languages'
foreach ($lng in 'es', 'it', 'pt-BR', 'de') {
  PfStart -Lang $lng -Script (DumpScript 500 15)
  Pf peek start --search 'zz' | Out-Null; Start-Sleep -Seconds 3
  $t = @(PfLogAll | Where-Object { $_ -match 'search tabs drawn' }) | Select-Object -Last 1
  $room = if ("$t" -match 'hint_room=(-?\d+)') { [int]$Matches[1] } else { -999 }
  $want = if ($lng -eq 'de') { 'hint=Strg\+Tab' } else { 'hint=Ctrl\+Tab' }
  Check "${lng}: the hint is in view ($want)" ($room -ge 0 -and "$t" -match $want) "$t"
  Shot "t05-tabs-$lng"
  Pf peek end | Out-Null; Start-Sleep -Seconds 1
}

Case 'cleanup'
PfQuit
Remove-Item -LiteralPath (Join-Path $PfDocs 'pf-total') -Recurse -ErrorAction SilentlyContinue
Remove-Item -LiteralPath (Join-Path $PfDocs '季度计划.docx') -ErrorAction SilentlyContinue
ClearDownloads
