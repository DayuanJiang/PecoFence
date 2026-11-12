# PR #223, the Clipboard History script fence: made from the fence menu, what it keeps (texts,
# file lists, pictures), what password managers mark as private, texts with quotes and colons
# beside the row's buttons, the blur, pin and restart, Delete all and the purge timer, what stays
# on the disk, large clipboards and the cost at rest.
#   run.sh --var Part=A   basics, private formats, row texts, blur     (default: A)
#   run.sh --var Part=B   pin, restart, Delete all, purge, files left on the disk
#   run.sh --var Part=C   a 51 MB picture, 100000 files, a 200000 character text, cost at rest
#   run.sh --var Part=D   in Chinese: the fence's title and its rows
if (-not $Part) { $Part = 'A' }
# The menu texts, the fence's title and two of its rows in the language of the part.
$L = if ($Part -eq 'D') {
  @{ lang = 'zh-CN'; add = '^添加到桌面'; fence = '新建栅栏'; tool = '^新建小工具'; weather = '天气'; pick = '^剪贴板历史$'; title = '剪贴板历史'; show = '显示条目'; empty = '还没有复制任何内容' }
} else {
  @{ lang = 'en'; add = '^Add to desktop'; fence = '^New fence'; tool = '^New tool'; weather = 'Weather'; pick = '^Clipboard history$'; title = 'Clipboard History'; show = 'Show entries'; empty = 'Nothing copied yet' }
}
Add-Type -AssemblyName System.Windows.Forms, System.Drawing

# Hit rects of a dump field (script_rows) as objects: i (row * 100 + cell), l, t, r, b in content DIPs.
function DumpRects($w, $name) {
  $v = DumpVal $w $name
  if (-not $v -or $v -eq '-') { return @() }
  return @($v -split ';' | ForEach-Object {
    $pair = $_ -split ':'; $n = $pair[1] -split ','
    [pscustomobject]@{ i = [int]$pair[0]; l = [double]$n[0]; t = [double]$n[1]; r = [double]$n[2]; b = [double]$n[3] }
  })
}
# Screen point of content DIPs (x, y) of a dumped window.
function ContentPt($w, $x, $y) {
  $sc = [double](DumpVal $w 'content_y') / 36.0
  return @([int]($w.l + [double](DumpVal $w 'content_x') + $x * $sc), [int]($w.t + [double](DumpVal $w 'content_y') + $y * $sc))
}
function ChOut() { $s = (FenceGet $script:ChId).script; return "$($s.output)" }
function ChLines() { return @((ChOut) -split "`n") }
function ChWait($pattern, $sec = 10) {
  for ($i = 0; $i -lt $sec * 4; $i++) { if ((ChOut) -match $pattern) { return $true }; Start-Sleep -Milliseconds 250 }
  return $false
}
function ChWaitGone($pattern, $sec = 10) {
  for ($i = 0; $i -lt $sec * 4; $i++) { if ((ChOut) -notmatch $pattern) { return $true }; Start-Sleep -Milliseconds 250 }
  return $false
}
# The output line (its index) that matches.
function ChLine($pattern) { $l = ChLines; for ($i = 0; $i -lt $l.Count; $i++) { if ($l[$i] -match $pattern) { return $i } }; return -1 }
# The clickable cells drawn for an output line: their cell numbers.
function ChCells($line) {
  $w = DumpOf $script:ChTitle 5
  return @(DumpRects $w 'script_rows' | Where-Object { [math]::Floor($_.i / 100) -eq $line } | ForEach-Object { $_.i % 100 })
}
function ChClick($line, $cell) {
  $w = DumpOf $script:ChTitle 5
  $r = @(DumpRects $w 'script_rows' | Where-Object { $_.i -eq $line * 100 + $cell }) | Select-Object -First 1
  if (-not $r) { Log "no cell $line/$cell in $(DumpVal $w 'script_rows')"; return $false }
  $p = ContentPt $w (($r.l + $r.r) / 2) (($r.t + $r.b) / 2)
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900
  ParkMouse
  return $true
}
# The script's own PowerShell process.
function ChProc() {
  $p = Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" | Where-Object { $_.CommandLine -match 'clipboard-history' } | Select-Object -First 1
  if ($p) { return (Get-Process -Id $p.ProcessId -ErrorAction SilentlyContinue) }
  return $null
}
function ChData() { return (Get-ChildItem $PfApp -Recurse -Directory -Filter 'scripts-data' -ErrorAction SilentlyContinue | Select-Object -First 1 | ForEach-Object { Join-Path $_.FullName $script:ChId }) }
# A text on the clipboard with extra formats (name -> bytes), as a password manager sets them.
function ClipWith($text, $formats = @{}) {
  $d = New-Object Windows.Forms.DataObject
  $d.SetData([Windows.Forms.DataFormats]::UnicodeText, $text)
  foreach ($k in $formats.Keys) { $d.SetData($k, (New-Object IO.MemoryStream (, [byte[]]$formats[$k]))) }
  [Windows.Forms.Clipboard]::SetDataObject($d, $true)
  Start-Sleep -Milliseconds 1200
}
function ClipFormats() { return (@([Windows.Forms.Clipboard]::GetDataObject().GetFormats($false)) -join ',') }
function ClipPicture($w, $h, [switch]$Noise) {
  $b = New-Object Drawing.Bitmap $w, $h
  if ($Noise) {
    $bits = $b.LockBits((New-Object Drawing.Rectangle 0, 0, $w, $h), 'WriteOnly', 'Format32bppArgb')
    $bytes = New-Object byte[] ($bits.Stride * $h); (New-Object Random 7).NextBytes($bytes)
    [Runtime.InteropServices.Marshal]::Copy($bytes, 0, $bits.Scan0, $bytes.Length); $b.UnlockBits($bits)
  } else {
    $g = [Drawing.Graphics]::FromImage($b); $g.Clear([Drawing.Color]::FromArgb(30, 120, 200))
    $g.DrawString('PICTURE 4711', (New-Object Drawing.Font 'Segoe UI', 28), [Drawing.Brushes]::White, 10, 10); $g.Dispose()
  }
  [Windows.Forms.Clipboard]::SetImage($b); $b.Dispose()
}
function Cpu($proc, $sec) {
  if (-not $proc) { return -1 }
  $proc.Refresh(); $a = $proc.TotalProcessorTime.TotalMilliseconds
  Start-Sleep -Seconds $sec
  $proc.Refresh(); return [math]::Round(($proc.TotalProcessorTime.TotalMilliseconds - $a) / ($sec * 10.0), 2)
}
# Opens 添加到桌面 > 新建小工具 of a plain fence's menu and picks an entry.
function AddTool() {
  $f = @(@((PfJson fence list)) | Where-Object { $_ -and -not $_.tool }) | Select-Object -First 1
  $wr = $f.windowRect
  [G]::RClick([int]($wr.x + $wr.w / 2), [int]($wr.y + 14)); Start-Sleep -Milliseconds 700
  $h = MenuWin 5
  $p = MenuPoint $h $L.add
  if (-not $p) { return $false }
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
  $add = $null
  foreach ($m in @([W]::Tops('#32768'))) { if ((MenuTexts $m) -match $L.fence) { $add = $m } }
  if (-not $add) { return $false }
  $p = MenuPoint $add $L.tool
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
  $sub = $null
  foreach ($m in @([W]::Tops('#32768'))) { if ((MenuTexts $m) -match $L.weather) { $sub = $m } }
  if (-not $sub) { return $false }
  return [bool](MenuPick $L.pick $sub)
}

Case 'setup'
CloseWindows
Set-Culture en-US
[Windows.Forms.Clipboard]::Clear()
PfStart -Lang $L.lang -Script (DumpScript 250 12)
Start-Sleep -Seconds 2
$picked = AddTool
Start-Sleep -Seconds 6
$ch = @(@((PfJson fence list)) | Where-Object { $_ -and $_.tool -eq 'script' })
Check 'New tool > Clipboard history makes a script fence with the name the menu shows' ($picked -and $ch.Count -eq 1 -and $ch[0].title -eq $L.title) "picked=$picked fences=$((@((PfJson fence list)) | ForEach-Object { $_.title }) -join ', ')"
$script:ChId = "$($ch[0].id)"; $script:ChTitle = "$($ch[0].title)"
Pf fence move $script:ChId --x 900 --y 80 | Out-Null
Pf fence resize $script:ChId --w 420 --h 600 | Out-Null
Check 'the script keeps running and shows its boxes' ((ChWait $L.show 20) -and (FenceGet $script:ChId).script.running) ((ChLines)[0])
Check 'what was on the clipboard before is not listed' ((ChWait $L.empty 15)) ((ChLines) -join ' / ')
ParkMouse

if ($Part -eq 'A') {
  Case 'A1 what is kept'
  ClipWith 'HELLO WORLD 12345'
  Check 'a copied text is listed' (ChWait 'HELLO WORLD 12345') ((ChLines) -join ' / ')
  $list = New-Object Collections.Specialized.StringCollection
  Set-Content C:\pf\run\one.txt 1; Set-Content C:\pf\run\two.txt 2
  [void]$list.Add('C:\pf\run\one.txt'); [void]$list.Add('C:\pf\run\two.txt')
  [Windows.Forms.Clipboard]::SetFileDropList($list)
  Check 'a copied list of files is listed' (ChWait 'one\.txt; C:\\pf\\run\\two\.txt') ((ChLines) -join ' / ')
  ClipPicture 300 200
  Check 'a copied picture is listed as a picture cell' (ChWait 'image="[^"]+\.t\.png"') ((ChLines) -join ' / ')
  $data = ChData
  $pics = @(Get-ChildItem "$data\images" -ErrorAction SilentlyContinue | ForEach-Object { $_.Name })
  Check 'the picture is kept as files in the fence data folder (full size, small, blurred)' (($pics -match '^\d+\.png$') -and ($pics -match '\.t\.png$') -and ($pics -match '\.b\.png$')) "data=$data files=$($pics -join ',')"
  Shot 'a1-kept'

  Case 'A2 what password managers mark as private'
  ClipWith 'SECRETALPHA' @{ 'ExcludeClipboardContentFromMonitorProcessing' = [byte[]](1, 0, 0, 0) }
  $fa = ClipFormats
  Start-Sleep -Seconds 2
  Check 'ExcludeClipboardContentFromMonitorProcessing: the text is left out' ($fa -match 'ExcludeClipboardContentFromMonitorProcessing' -and (ChOut) -notmatch 'SECRETALPHA') "formats=$fa"
  ClipWith 'SECRETBRAVO' @{ 'CanIncludeInClipboardHistory' = [byte[]](0, 0, 0, 0) }
  $fb = ClipFormats
  Start-Sleep -Seconds 2
  Check 'CanIncludeInClipboardHistory = 0: the text is left out' ($fb -match 'CanIncludeInClipboardHistory' -and (ChOut) -notmatch 'SECRETBRAVO') "formats=$fb"
  ClipWith 'ALLOWEDCHARLIE' @{ 'CanIncludeInClipboardHistory' = [byte[]](1, 0, 0, 0) }
  Check 'CanIncludeInClipboardHistory = 1: the text is listed' (ChWait 'ALLOWEDCHARLIE') ((ChLines) -join ' / ')
  $data = ChData
  $onDisk = @(Get-ChildItem $data -Recurse -File -ErrorAction SilentlyContinue | Where-Object { $_.Extension -ne '.png' } | Select-String -Pattern 'SECRETALPHA|SECRETBRAVO' -List)
  Check 'neither private text is in a file of the data folder' ($onDisk.Count -eq 0) "$($onDisk | ForEach-Object { $_.Path })"
  $inLog = @(PfLogAll | Select-String -Pattern 'SECRETALPHA|SECRETBRAVO|HELLO WORLD 12345|ALLOWEDCHARLIE').Count
  Check 'no copied text reaches the PecoFence log' ($inLog -eq 0) "lines=$inLog"

  Case 'A3 texts with quotes and colons keep their buttons'
  ClipWith '<a href="https://exam'
  $ok = ChWait 'a href='
  $l = ChLine 'a href='
  $cells = ChCells $l
  Check 'a text that ends inside href=" keeps its entry, trash and pin cells' ($ok -and ($cells -join ',') -eq '0,1,2') "line=$l cells=$($cells -join ',') text=$((ChLines)[$l])"
  Shot 'a3-quote'
  $before = @(ChLines).Count
  $clicked = ChClick $l 1
  Check 'its trash button deletes it' ($clicked -and (ChWaitGone 'a href=' 5)) "clicked=$clicked lines=$before -> $(@(ChLines).Count)"
  ClipWith 'std:::vector ::: end'
  $ok = ChWait 'std'
  $l = ChLine 'std'
  $cells = ChCells $l
  Check 'a text with three colons in a row keeps its three cells' ($ok -and ($cells -join ',') -eq '0,1,2') "line=$l cells=$($cells -join ',') text=$((ChLines)[$l])"
  ClipWith "it's x='y and more"
  $ok = ChWait "it's"
  $l = ChLine "it's"
  $cells = ChCells $l
  Check 'a text with an apostrophe after = keeps its three cells' ($ok -and ($cells -join ',') -eq '0,1,2') "line=$l cells=$($cells -join ',') text=$((ChLines)[$l])"
  ClipWith '[ ] looks like a to-do | action=evil run=calc.exe'
  $ok = ChWait 'looks like a to-do'
  $l = ChLine 'looks like a to-do'
  Check 'a copied text cannot add an action to its row' ($ok -and (ChLines)[$l] -notmatch '\| action=evil' -and (ChLines)[$l] -match 'action=copy:\d+') "text=$((ChLines)[$l])"

  Case 'A4 click to copy again'
  $l = ChLine 'HELLO WORLD 12345'
  [Windows.Forms.Clipboard]::Clear(); Start-Sleep -Milliseconds 600
  $clicked = ChClick $l 0
  $clip = [Windows.Forms.Clipboard]::GetText()
  Check 'a click on an entry puts it on the clipboard again' ($clicked -and $clip -eq 'HELLO WORLD 12345') "clip='$clip'"

  Case 'A5 blur'
  $w = DumpOf $script:ChTitle 5
  $seen = OcrText $w.l $w.t ($w.r - $w.l) ($w.b - $w.t) 'a5-shown'
  Check 'shown, the text can be read off the screen (OCR)' ($seen -match 'HELLO\s*WORLD') "ocr=$($seen -replace '\s+', ' ')"
  $clicked = ChClick 0 0
  $hid = ChWaitGone 'HELLO WORLD 12345' 6
  Check 'Show entries unticked: no entry text is left in what the script prints' ($clicked -and $hid -and (ChOut) -notmatch 'ALLOWEDCHARLIE|one\.txt') ((ChLines) -join ' / ')
  Start-Sleep -Seconds 1
  $w = DumpOf $script:ChTitle 5
  $seen = OcrText $w.l $w.t ($w.r - $w.l) ($w.b - $w.t) 'a5-blurred'
  Check 'blurred, OCR reads none of the texts, file names or the picture off the screen' ($seen -notmatch 'HELLO|WORLD|12345|ALLOWED|CHARLIE|one\.txt|PICTURE|4711') "ocr=$($seen -replace '\s+', ' ')"
  Shot 'a5-blurred'
  $l = ChLine 'copy:\d+'
  $w = DumpOf $script:ChTitle 5
  $r = @(DumpRects $w 'script_rows' | Where-Object { $_.i -eq $l * 100 }) | Select-Object -First 1
  $p = ContentPt $w (($r.l + $r.r) / 2) (($r.t + $r.b) / 2)
  [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 1800
  $tip = OcrText ($w.l - 200) $w.t ($w.r - $w.l + 400) ($w.b - $w.t) 'a5-tip'
  Check 'the tooltip of a blurred entry does not show its text' ($tip -notmatch 'HELLO|WORLD|ALLOWED|CHARLIE|std|looks') "ocr=$($tip -replace '\s+', ' ')"
  ParkMouse
  $clicked = ChClick 0 0
  Check 'Show entries ticked again: the texts are back' ($clicked -and (ChWait 'HELLO WORLD 12345' 6)) ((ChLines)[0])
  $clicked = ChClick 0 1
  $hid = ChWaitGone 'HELLO WORLD 12345' 6
  Check 'Reveal on click ticked: the entries are blurred' ($clicked -and $hid) ((ChLines)[0])
  $l = ChLine 'reveal:\d+'
  $clicked = ChClick $l 0
  $shown = ChWait 'action=copy:\d+ tooltip' 4
  Check 'a click shows that entry' ($clicked -and $shown) ((ChLines)[$l])
  Start-Sleep -Seconds 6
  Check 'and it is blurred again 5 seconds later' ((ChOut) -notmatch 'fill=true action=copy:\d+') ((ChLines)[$l])
  ChClick 0 1 | Out-Null

  Case 'A6 nothing leaves the computer'
  $sp = ChProc
  $tcp = @(Get-NetTCPConnection -OwningProcess $sp.Id -ErrorAction SilentlyContinue).Count
  $udp = @(Get-NetUDPEndpoint -OwningProcess $sp.Id -ErrorAction SilentlyContinue).Count
  $pf = Get-Process pecofence | Select-Object -First 1
  $tcpPf = @(Get-NetTCPConnection -OwningProcess $pf.Id -ErrorAction SilentlyContinue).Count
  Check 'the script and PecoFence hold no network connection' ($sp -and $tcp -eq 0 -and $udp -eq 0 -and $tcpPf -eq 0) "script pid=$($sp.Id) tcp=$tcp udp=$udp pecofence tcp=$tcpPf"
}

if ($Part -eq 'B') {
  Case 'B1 pin and restart'
  ClipWith 'KEEPME pinned text'
  ChWait 'KEEPME' | Out-Null
  ClipPicture 300 200
  ChWait 'image=' | Out-Null
  ClipWith 'LOSEME plain text'
  ChWait 'LOSEME' | Out-Null
  $l = ChLine 'KEEPME'
  $clicked = ChClick $l 2
  $pinnedTop = ChWait 'KEEPME[^\n]*checked=true' 5
  Check 'the pin button pins an entry and moves it to the top' ($clicked -and $pinnedTop -and (ChLine 'KEEPME') -lt (ChLine 'LOSEME')) ((ChLines) -join ' / ')
  $data = ChData
  $pinned = "$data\pinned.json"
  Check 'the pinned text is written to pinned.json in the data folder' ((Test-Path $pinned) -and ((Get-Content $pinned -Raw) -match 'KEEPME')) "$pinned"
  Shot 'b1-pinned'
  PfQuit
  Start-Sleep -Seconds 2
  $left = @(Get-ChildItem "$data\images" -ErrorAction SilentlyContinue | ForEach-Object { $_.Name })
  $full = @($left | Where-Object { $_ -match '^\d+\.png$' })
  Check 'after PecoFence quits no picture that was not pinned stays on the disk' ($full.Count -eq 0) "left in images: $($left -join ',')"
  $alive = [bool](ChProc)
  Check 'the script ends with PecoFence' (-not $alive) "alive=$alive"
  PfStart -Lang $L.lang -Script (DumpScript 250 10)
  Start-Sleep -Seconds 4
  Check 'after a restart the pinned entry is back' (ChWait 'KEEPME[^\n]*checked=true' 25) ((ChLines) -join ' / ')
  Check 'and the others are gone' ((ChOut) -notmatch 'LOSEME|image=') ((ChLines) -join ' / ')

  Case 'B2 Delete all and the purge'
  ClipWith 'GOAWAY one'; ChWait 'GOAWAY one' | Out-Null
  ClipWith 'GOAWAY two'; ChWait 'GOAWAY two' | Out-Null
  $clicked = ChClick 1 0
  Check 'Delete all removes what is not pinned and keeps the pinned entry' ($clicked -and (ChWaitGone 'GOAWAY' 5) -and (ChOut) -match 'KEEPME') ((ChLines) -join ' / ')
  # Purge in 5 seconds: < until the delay says 5 s, then tick the box.
  for ($i = 0; $i -lt 14 -and (ChLines)[1] -notmatch ':: 5 s \|'; $i++) { ChClick 1 2 | Out-Null }
  Check 'the < button steps the delay down to 5 s' ((ChLines)[1] -match ':: 5 s \|') ((ChLines)[1])
  ChClick 1 1 | Out-Null
  ClipWith 'PURGEME text'
  $came = ChWait 'PURGEME' 5
  $count = ChWait '00:0\d \| color=gray' 4
  Start-Sleep -Seconds 7
  Check 'with the purge ticked a new entry counts down and is deleted after the delay' ($came -and $count -and (ChOut) -notmatch 'PURGEME' -and (ChOut) -match 'KEEPME') ((ChLines) -join ' / ')
  ChClick 1 1 | Out-Null
  Shot 'b2-purged'

  Case 'B3 the fence is deleted'
  $data = ChData
  ClipPicture 300 200; ChWait 'image=' | Out-Null
  Pf fence delete $script:ChId | Out-Null
  Start-Sleep -Seconds 3
  $alive = [bool](ChProc)
  $left = @(Get-ChildItem $data -Recurse -File -ErrorAction SilentlyContinue | ForEach-Object { $_.Name })
  Check 'deleting the fence ends its script' (-not $alive) "alive=$alive"
  Check 'what the deleted fence left in its data folder' $true "files: $($left -join ',')"
}

if ($Part -eq 'C') {
  Case 'C1 cost at rest'
  ClipWith 'one entry to show'
  ChWait 'one entry' | Out-Null
  Start-Sleep -Seconds 20
  $sp = ChProc
  $pf = Get-Process pecofence | Select-Object -First 1
  $cpuS = Cpu $sp 30
  $cpuP = Cpu $pf 30
  $sp.Refresh(); $pf.Refresh()
  Check 'at rest: CPU of the script and of PecoFence (percent of one core), memory, threads' ($cpuS -lt 3 -and $cpuP -lt 3) "script cpu=$cpuS% private=$([int]($sp.PrivateMemorySize64 / 1MB)) MB ws=$([int]($sp.WorkingSet64 / 1MB)) MB threads=$($sp.Threads.Count) handles=$($sp.HandleCount); pecofence cpu=$cpuP% private=$([int]($pf.PrivateMemorySize64 / 1MB)) MB threads=$($pf.Threads.Count) handles=$($pf.HandleCount)"

  Case 'C2 a 51 MB picture'
  $t = Get-Date
  ClipPicture 4000 3200 -Noise
  $came = ChWait 'Image 4000 x 3200' 90
  $took = [math]::Round(((Get-Date) - $t).TotalSeconds, 1)
  $data = ChData
  $size = @(Get-ChildItem "$data\images" -ErrorAction SilentlyContinue | ForEach-Object { "$($_.Name)=$([int]($_.Length / 1KB))KB" }) -join ','
  $sp = ChProc; $pf = Get-Process pecofence | Select-Object -First 1
  Check 'a 4000 x 3200 picture is listed' $came "after ${took}s; files: $size; script private=$([int]($sp.PrivateMemorySize64 / 1MB)) MB; pecofence private=$([int]($pf.PrivateMemorySize64 / 1MB)) MB"
  Check 'PecoFence still answers while the script works on it' ((PfJson fence list) -ne $null)
  Shot 'c2-big-picture'

  Case 'C3 100000 files'
  $list = New-Object Collections.Specialized.StringCollection
  $paths = New-Object string[] 100000
  for ($i = 0; $i -lt 100000; $i++) { $paths[$i] = "C:\pf\run\many\file-$i.txt" }
  $list.AddRange($paths)
  $t = Get-Date
  [Windows.Forms.Clipboard]::SetFileDropList($list)
  $set = [math]::Round(((Get-Date) - $t).TotalSeconds, 1)
  $came = ChWait 'file-0\.txt' 120
  $took = [math]::Round(((Get-Date) - $t).TotalSeconds, 1)
  $line = (ChLines)[(ChLine 'file-0\.txt')]
  Check '100000 copied files are listed as one entry' $came "after ${took}s (${set}s of it to put them on the clipboard); line length $($line.Length)"
  $sp = ChProc
  $idle = Cpu $sp 10
  # With the purge ticked the script builds its screen four times a second.
  ChClick 1 1 | Out-Null
  $busy = Cpu $sp 10
  ChClick 1 1 | Out-Null
  $sp.Refresh()
  Check 'CPU of the script with that entry in the list: purge off, purge counting down' ($idle -lt 5 -and $busy -lt 10) "purge off=$idle% purge on=$busy% private=$([int]($sp.PrivateMemorySize64 / 1MB)) MB"

  Case 'C4 a 200000 character text'
  ClipWith ('LONGTEXT ' + ('lorem ipsum dolor sit amet ' * 8000))
  $came = ChWait 'LONGTEXT' 30
  $sp = ChProc
  ChClick 1 1 | Out-Null
  $busy = Cpu $sp 10
  ChClick 1 1 | Out-Null
  Check 'a 200000 character text is listed, cut to one line' ($came -and (ChLines)[(ChLine 'LONGTEXT')].Length -lt 2100) "line length $((ChLines)[(ChLine 'LONGTEXT')].Length); script cpu with the purge counting down=$busy%"
  $frames = @(PfLogAll | Select-String -Pattern 'animation run' | Select-Object -Last 3 | ForEach-Object { $_.Line })
  Log "frame runs: $($frames -join ' // ')"
}

if ($Part -eq 'D') {
  Case 'D1 in Chinese'
  Check 'the rows are in Chinese' ((ChLines)[1] -match '全部删除' -and (ChLines)[1] -match '清除条目') ((ChLines)[1])
  ClipWith '中文条目 12345'
  Check 'a copied Chinese text is listed' (ChWait '中文条目 12345') ((ChLines) -join ' / ')
  Shot 'd1-chinese'
}

Case 'cleanup'
[Windows.Forms.Clipboard]::Clear()
PfQuit
