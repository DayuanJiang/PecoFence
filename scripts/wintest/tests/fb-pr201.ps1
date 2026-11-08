# PR #201 电脑状态 (PC status) example: the fence made from 添加到桌面 › 新建小工具 › 电脑状态 in
# Chinese and English, its rows (IP of the box, drive rows), its height, a click on the IP row
# (clipboard, the 已复制 tip beside the pointer, no focus taken, gone after about 2 s, a failed
# copy while another program holds the clipboard), an unedited 0.3.0 script replaced at the next
# start while an edited one stays, and many drive rows in an old (230) and a new (300) fence.
# Needs --extra with old/status.{zh,en}.ps1 (the 0.3.0 scripts) and --var Build=new|old.
Case 'setup'
CloseWindows
. C:\pf\kit\rec.ps1
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Windows.Forms
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class CB {
  [DllImport("user32.dll")] public static extern bool OpenClipboard(IntPtr h);
  [DllImport("user32.dll")] public static extern bool CloseClipboard();
  [DllImport("user32.dll")] public static extern int GetWindowLongW(IntPtr h, int i);
}
'@
# run.sh --var Build=new (the PR build) or Build=old (origin/main before it).
$New = ($Build -eq 'new')
Log "build has the PR: $New"
$BoxIp = (Get-NetIPConfiguration | Where-Object { $_.IPv4DefaultGateway } | Select-Object -First 1).IPv4Address.IPAddress
Log "box IP $BoxIp"
# Assign the result before piping it on (it comes back as one array).
function ToolFences($kind) { return ,@(@((PfJson fence list)) | Where-Object { $_ -and $_.tool -eq $kind }) }
function PlainFence() { return (@(@((PfJson fence list)) | Where-Object { $_ -and -not $_.tool }) | Select-Object -First 1) }
function FenceMenu($f) {
  $wr = $f.windowRect
  [G]::RClick([int]($wr.x + $wr.w / 2), [int]($wr.y + 14)); Start-Sleep -Milliseconds 700
  return (MenuWin 5)
}
function SubOf($h, $regex, $want) {
  $p = MenuPoint $h $regex
  if (-not $p) { return $null }
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
  foreach ($m in @([W]::Tops('#32768'))) { if ((MenuTexts $m) -match $want) { return $m } }
  return $null
}
# 添加到桌面 › 新建小工具 › 电脑状态 in the first plain fence's menu.
function AddStatus($zh) {
  $add = if ($zh) { '^添加到桌面' } else { '^Add to desktop' }
  $tool = if ($zh) { '^新建小工具' } else { '^New tool' }
  $status = if ($zh) { '^电脑状态$' } else { '^PC status$' }
  # A submenu is found by an item it holds (its first one is 新建栅栏 / the clock).
  $h = SubOf (FenceMenu (PlainFence)) $add $tool.TrimStart('^')
  $sub = if ($h) { SubOf $h $tool $status.Trim('^$') } else { $null }
  Log "menus: add=$h $(if ($h) { MenuTexts $h }) tool=$sub $(if ($sub) { MenuTexts $sub })"
  if ($sub) { MenuPick $status $sub | Out-Null } else { CloseMenus }
  Start-Sleep -Seconds 2
}
function ScriptOf($f) { $g = PfJson fence get $f.id; if ($g.fence) { return $g.fence.script }; return $g.script }
function WaitRows($f, $regex, $sec = 25) {
  for ($i = 0; $i -lt $sec; $i++) { $s = ScriptOf $f; if ("$($s.output)" -match $regex) { return $s }; Start-Sleep -Seconds 1 }
  return $s
}
# Windows OCR lines of a screen area with their screen rects (the crop is read at twice the size).
function OcrLines($x, $y, $w, $h, $name) {
  Add-Type -AssemblyName System.Runtime.WindowsRuntime
  $null = [Windows.Media.Ocr.OcrEngine, Windows.Foundation, ContentType = WindowsRuntime]
  $null = [Windows.Graphics.Imaging.BitmapDecoder, Windows.Graphics, ContentType = WindowsRuntime]
  $null = [Windows.Storage.StorageFile, Windows.Storage, ContentType = WindowsRuntime]
  $asTask = @([System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object { $_.Name -eq 'AsTask' -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' })[0]
  $await = { param($op, $type) $t = $asTask.MakeGenericMethod($type).Invoke($null, @($op)); $t.Wait(-1) | Out-Null; $t.Result }
  $src = [Px]::Grab($x, $y, $w, $h)
  $big = New-Object Drawing.Bitmap ($w * 2), ($h * 2)
  $g = [Drawing.Graphics]::FromImage($big); $g.InterpolationMode = 'HighQualityBicubic'; $g.DrawImage($src, 0, 0, $w * 2, $h * 2); $g.Dispose()
  $path = "$PfShots\$name.png"; $big.Save($path); $big.Dispose(); $src.Dispose()
  $eng = [Windows.Media.Ocr.OcrEngine]::TryCreateFromUserProfileLanguages()
  if (-not $eng) { return @() }
  $file = & $await ([Windows.Storage.StorageFile]::GetFileFromPathAsync($path)) ([Windows.Storage.StorageFile])
  $stream = & $await ($file.OpenAsync([Windows.Storage.FileAccessMode]::Read)) ([Windows.Storage.Streams.IRandomAccessStream])
  $dec = & $await ([Windows.Graphics.Imaging.BitmapDecoder]::CreateAsync($stream)) ([Windows.Graphics.Imaging.BitmapDecoder])
  $bmp = & $await ($dec.GetSoftwareBitmapAsync()) ([Windows.Graphics.Imaging.SoftwareBitmap])
  $res = & $await ($eng.RecognizeAsync($bmp)) ([Windows.Media.Ocr.OcrResult])
  $stream.Dispose()
  return @($res.Lines | ForEach-Object {
    $r = @($_.Words | ForEach-Object { $_.BoundingRect })
    $l = ($r | Measure-Object X -Minimum).Minimum; $t = ($r | Measure-Object Y -Minimum).Minimum
    $b = ($r | ForEach-Object { $_.Y + $_.Height } | Measure-Object -Maximum).Maximum
    [pscustomobject]@{ Text = $_.Text; X = [int]($x + $l / 2); Y = [int]($y + ($t + $b) / 4) }
  })
}
# The PecoFence infotip windows on screen, with their text (UI Automation reads a comctl32 tip).
function Tips() {
  return @(ProcTops 'tooltips_class32' 'pecofence' | ForEach-Object {
    $n = try { [Windows.Automation.AutomationElement]::FromHandle($_).Current.Name } catch { '?' }
    [pscustomobject]@{ H = $_; Text = $n; R = (Rect $_) }
  })
}
function TipText() { return ((Tips | ForEach-Object { $_.Text }) -join ' / ') }
function Clip() { try { return [Windows.Forms.Clipboard]::GetText() } catch { return "error: $_" } }
function ConfigScripts() { return "$PfApp\config\scripts" }
function WriteScript($path, $text) { [IO.File]::WriteAllText($path, ($text.TrimStart([char]0xFEFF) -replace "`r?`n", "`r`n"), (New-Object Text.UTF8Encoding($true))) }

# One language: the fence, its rows and height, and the copy click.
function StatusCase($zh) {
  trap { PfTrap $_; continue }
  $tag = if ($zh) { 'zh' } else { 'en' }
  AddStatus $zh
  $all = ToolFences 'script'
  $f = $all | Where-Object { $_.title -match '电脑状态|pc-status' } | Select-Object -First 1
  Check "$tag | 添加到桌面 › 新建小工具 › 电脑状态 makes the fence" ($null -ne $f) (($all | ForEach-Object { $_.title }) -join ', ')
  if (-not $f) { return }
  $drive = if ($zh) { 'C 盘剩余 \d+ GB（\d+%） \| open=C:\\ ' } else { 'C: free \d+ GB \(\d+%\) \| open=C:\\ ' }
  $s = WaitRows $f 'copy='
  $out = "$($s.output)"
  Add-Content "$PfShots\outputs.txt" "== $tag`r`n$out" -Encoding UTF8
  Check "$tag | the script keeps running and prints its rows" ($s.running -and $out -match 'CPU \d+%') "running=$($s.running) outcome=$($s.outcome)"
  Check "$tag | a row for drive C: that opens it" ($out -match $drive) (($out -split "`n" | Where-Object { $_ -match 'open=' }) -join ' / ')
  $ipRow = ($out -split "`n" | Where-Object { $_ -match 'copy=' }) -join ' / '
  Check "$tag | the IP row shows and copies the box's address $BoxIp" ($ipRow -match [regex]::Escape(" $BoxIp | copy=$BoxIp ")) $ipRow
  $f = PfFence $f.title
  $wr = $f.windowRect
  Log "$tag fence windowRect $($wr.x),$($wr.y) $($wr.w)x$($wr.h)"
  $wantH = if ($New) { 300 } else { 230 }
  Check "$tag | a new PC status fence is $wantH tall" ([math]::Abs($wr.h - $wantH) -le 2) "h=$($wr.h)"
  # Room for the tip below the fence when it sits low.
  Pf fence move $f.id --x 300 --y 120 | Out-Null; Start-Sleep -Seconds 1
  $f = PfFence $f.title; $wr = $f.windowRect
  [G]::MoveTo(80, 860); Start-Sleep -Milliseconds 500
  Crop "$tag-fence" ($wr.x - 10) ($wr.y - 10) ($wr.w + 200) ($wr.h + 60) | Out-Null
  $lines = OcrLines $wr.x $wr.y $wr.w $wr.h "$tag-ocr"
  Log "$tag ocr: $(($lines | ForEach-Object { "$($_.Text)@$($_.Y)" }) -join ' | ')"
  $ip = $lines | Where-Object { $_.Text -match '\d+\.\d+\.\d+\.\d+' } | Select-Object -First 1
  Check "$tag | OCR finds the IP row" ($null -ne $ip) (($lines | ForEach-Object { $_.Text }) -join ' | ')
  if (-not $ip) { return }
  $px = $wr.x + 90; $py = $ip.Y
  if ($zh) { RecStart "copy-$Build" -X ($wr.x - 20) -Y ($wr.y - 20) -W 520 -H ($wr.h + 90) -Fps 15; Start-Sleep -Milliseconds 600; RecMark '鼠标停在「本机 IP」这一行' }
  [G]::MoveTo($px, $py); Start-Sleep -Milliseconds 1800
  $hover = TipText
  Crop "$tag-hover-tip" ($wr.x - 10) ($wr.y - 10) ($wr.w + 200) ($wr.h + 60) | Out-Null
  $hoverWant = if ($zh) { '点一下复制' } else { 'Click to copy' }
  Check "$tag | resting on the IP row shows the script's tooltip" ($hover -match $hoverWant) "tip=$hover"
  [Windows.Forms.Clipboard]::SetText('before the click')
  $fg0 = [L]::GetForegroundWindow()
  if ($zh) { RecMark '点一下，复制 IP' }
  [G]::Click($px, $py); Start-Sleep -Milliseconds 350
  $t1 = @(Tips); $fg1 = [L]::GetForegroundWindow()
  Crop "$tag-copy-tip" ($wr.x - 10) ($wr.y - 10) ($wr.w + 200) ($wr.h + 60) | Out-Null
  $clip = Clip
  Check "$tag | a click copies the address" ($clip -eq $BoxIp) "clipboard='$clip'"
  $copiedWant = if ($zh) { '^已复制$' } else { '^Copied$' }
  $tipText = ($t1 | ForEach-Object { $_.Text }) -join ' / '
  if ($New) {
    Check "$tag | 已复制 shows beside the pointer" ($t1.Count -eq 1 -and $t1[0].Text -match $copiedWant -and [math]::Abs($t1[0].R.L - $px) -le 60 -and $t1[0].R.T -ge $py) "tips=$tipText at $(if ($t1.Count) { "$($t1[0].R.L),$($t1[0].R.T)" }) pointer=$px,$py"
    if (-not $zh) { $o = OcrText ($px - 20) ($py + 5) 260 50 "$tag-copy-tip-ocr"; Check "$tag | OCR reads Copied in the tip" ($o -match 'Copied') "ocr=$o" }
  } else {
    Check "$tag | before: nothing shows after the click" ($t1.Count -eq 0) "tips=$tipText"
  }
  $exs = if ($t1.Count) { [CB]::GetWindowLongW($t1[0].H, -20) } else { 0 }
  Check "$tag | the tip takes no focus (foreground stays the window the click gave it)" (($t1 | Where-Object { $_.H -eq $fg1 }).Count -eq 0) "fg before=$([L]::Cls($fg0)) after=$([L]::Cls($fg1)) tipExStyle=0x$('{0:X}' -f $exs)"
  Start-Sleep -Milliseconds 1200
  $t15 = TipText
  if ($New) { Check "$tag | still there 1.5 s after the click" ($t15 -match $copiedWant.Trim('^$')) "tips=$t15" }
  Start-Sleep -Milliseconds 1200
  $t27 = @(Tips)
  if ($zh) { RecMark '鼠标不动，等一会'; Start-Sleep -Milliseconds 800; RecStop }
  $fg2 = [L]::GetForegroundWindow()
  Check "$tag | gone 2.9 s after the click, the pointer still on the row" ($t27.Count -eq 0) "tips=$(($t27 | ForEach-Object { $_.Text }) -join ' / ')"
  Check "$tag | the foreground did not change while the tip showed" ($fg2 -eq $fg1) "fg=$([L]::Cls($fg1)) then $([L]::Cls($fg2))"
  # Off the row and back: the script's own tooltip again, not 已复制.
  [G]::MoveTo($wr.x + $wr.w + 60, $py); Start-Sleep -Milliseconds 600
  [G]::MoveTo($px + 4, $py); Start-Sleep -Milliseconds 1800
  $again = TipText
  Check "$tag | back on the row later: the script's tooltip again" ($again -match $hoverWant) "tip=$again"
  # Another program holds the clipboard: the copy fails and the tip says so.
  [G]::MoveTo($wr.x + $wr.w + 60, $py); Start-Sleep -Milliseconds 400
  [G]::MoveTo($px, $py); Start-Sleep -Milliseconds 300
  [Windows.Forms.Clipboard]::SetText('held')
  $held = [CB]::OpenClipboard([IntPtr]::Zero)
  [G]::Click($px, $py); Start-Sleep -Milliseconds 350
  $fail = TipText
  Crop "$tag-copy-failed" ($wr.x - 10) ($wr.y - 10) ($wr.w + 200) ($wr.h + 60) | Out-Null
  [CB]::CloseClipboard() | Out-Null
  $clip = Clip
  $failWant = if ($zh) { '复制失败，请重试' } else { 'Copy failed. Try again.' }
  if ($New) {
    Check "$tag | clipboard held by another program: the tip says the copy failed" ($held -and $fail -eq $failWant -and $clip -eq 'held') "held=$held tip=$fail clipboard='$clip'"
  } else {
    Check "$tag | before: a failed copy shows nothing" ($held -and $fail -eq '' -and $clip -eq 'held') "held=$held tip=$fail clipboard='$clip'"
  }
  Check "$tag | the log says what the clicks did" ((@(PfLogAll) -match 'script row: text copied').Count -ge 2) ((@(PfLogAll) -match 'text copied' | Select-Object -Last 2) -join ' / ')
  [G]::MoveTo(80, 860); Start-Sleep -Milliseconds 300
}

Case 'zh: 电脑状态 from the fence menu'
Set-Culture zh-CN
PfStart -Lang zh-CN
Start-Sleep -Seconds 3
StatusCase $true

# An unedited 0.3.0 script gets the new one at the next start; an edited copy stays.
function OldCase() {
  trap { PfTrap $_; continue }
  $file = Join-Path (ConfigScripts) '电脑状态.ps1'
  Check 'the example file is in the config scripts folder' (Test-Path $file) $file
  PfQuit
  WriteScript $file ([IO.File]::ReadAllText("$PfExtra\old\status.zh.ps1", [Text.Encoding]::UTF8))
  $oldLen = (Get-Item $file).Length
  PfStart -Lang zh-CN
  Start-Sleep -Seconds 8
  $now = [IO.File]::ReadAllText($file, [Text.Encoding]::UTF8)
  $bom = ([IO.File]::ReadAllBytes($file)[0..2] -join ',') -eq '239,187,191'
  # PfLogAll also reads the shareable copy of the log: lines can come twice.
  $replaced = @(@(PfLogAll) -match 'unedited example script of an earlier version replaced')
  if ($New) {
    Check 'an unedited 0.3.0 script is replaced at the next start' ($now -match 'Get-LocalIP' -and $bom -and $now -match "`r`n" -and $replaced.Count -ge 1) "len $oldLen -> $((Get-Item $file).Length) bom=$bom log=$($replaced[0])"
    Check 'the log names the fence that runs the updated example' (@(@(PfLogAll) -match 'script fence runs the updated example id=').Count -ge 1)
  } else {
    Check 'before: the 0.3.0 script stays as it was' ($now -notmatch 'Get-LocalIP' -and $now -match 'Get-NetIPAddress') "len $oldLen -> $((Get-Item $file).Length)"
  }
  $all = ToolFences 'script'
  $f = $all | Select-Object -First 1
  $s = WaitRows $f 'copy='
  Check 'the fence runs it and shows its rows' ("$($s.output)" -match 'CPU \d+%' -and "$($s.output)" -match 'copy=') (("$($s.output)" -split "`n") -join ' / ')
  PfQuit
  $count = $replaced.Count
  $edited = [IO.File]::ReadAllText("$PfExtra\old\status.zh.ps1", [Text.Encoding]::UTF8) + "`r`n# 我改过这一行`r`n"
  WriteScript $file $edited
  $before = [IO.File]::ReadAllBytes($file)
  PfStart -Lang zh-CN
  Start-Sleep -Seconds 8
  $after = [IO.File]::ReadAllBytes($file)
  Check 'an edited 0.3.0 script stays as it is' ([Convert]::ToBase64String($before) -eq [Convert]::ToBase64String($after)) "len $($before.Length) -> $($after.Length)"
  $again = @(@(PfLogAll) -match 'unedited example script of an earlier version replaced').Count
  Check 'and is not logged as replaced again' ($again -le $count) "log lines $count -> $again"
}
Case 'an old PC status fence after the update'
OldCase

# Many drives: the same rows from a script that prints 7 drive rows, in a fence as tall as an old
# PC status fence (230) and as a new one (300).
function ManyCase() {
  trap { PfTrap $_; continue }
  $fake = "$PfRun\many-drives.ps1"
  # The order of the PC status example: CPU, memory, the IP and Task Manager rows, then the drives.
  $rows = @('CPU 12% | color=green icon=🧠', 'Memory used 41% | color=green icon=📊', '---',
    'IP address 192.168.1.23 | copy=192.168.1.23 icon=📋 tooltip="Click to copy"', 'Open Task Manager | run=taskmgr.exe icon=📈', '---') +
    @('C', 'D', 'E', 'F', 'G', 'H', 'I' | ForEach-Object { "${_}: free 120 GB (40%) | open=${_}:\ color=green icon=💽" })
  WriteScript $fake (($rows | ForEach-Object { "'" + ($_ -replace "'", "''") + "'" }) -join "`r`n")
  Pf fence create --script $fake | Out-Null
  Start-Sleep -Seconds 6
  $all = ToolFences 'script'
  $f = $all | Where-Object { $_.title -match 'many' } | Select-Object -First 1
  Check 'the many-drives fence runs' ("$((ScriptOf $f).output)" -match 'I: free') "$((ScriptOf $f).outcome)"
  foreach ($h in 230, 300, 340) {
    Pf fence move $f.id --rect "760,100,280,$h" | Out-Null; Start-Sleep -Milliseconds 1500
    $g = PfFence $f.title; $wr = $g.windowRect
    [G]::MoveTo(80, 860); Start-Sleep -Milliseconds 300
    Crop "many-$h" ($wr.x - 10) ($wr.y - 10) ($wr.w + 20) ($wr.h + 20) | Out-Null
    $txt = (OcrLines $wr.x $wr.y $wr.w $wr.h "many-$h-ocr" | ForEach-Object { $_.Text }) -join ' | '
    Log "many $h : $txt"
    $last = [regex]::Matches($txt, '([C-I]): free') | Select-Object -Last 1 | ForEach-Object { $_.Groups[1].Value }
    Check "7 drives in a fence $h tall: the IP and Task Manager rows stay in view" ($txt -match '192\.168' -and $txt -match 'Task Manager') "last drive shown: $last | $txt"
  }
}
Case 'many drives'
Set-Culture en-US
ManyCase

Case 'en: PC status from the fence menu'
PfFresh
Start-Sleep -Seconds 3
StatusCase $false
