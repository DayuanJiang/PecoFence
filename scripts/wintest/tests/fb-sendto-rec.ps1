# Before/after clip for feedback #125: Send to › Desktop (create shortcut) on a folder, with rules
# that put programs and shortcuts in different fences. The same steps on both builds; the BEFORE
# run sets $PfBin = $PfExtra first (the 0.3.1 build shipped with --extra). Recording in
# shots/rec-sendto-<side>, composed on the dev box with mkanim.py.
. C:\pf\kit\rec.ps1
$Side = if ($PfBin -eq $PfExtra) { 'before' } else { 'after' }
$Clip = 'C:\pf\clip'
$Folder = '项目资料'
$LinkFile = "$Folder - Shortcut.lnk"
function CleanClip() {
  foreach ($n in $LinkFile, '记事本.lnk', '会议记录 - Shortcut.lnk', 'st-np.lnk', 'st-mn.lnk') { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
}

# Windows 11's menu entry Show more options, in any window of Explorer (File Explorer's menu
# sits in its own window's tree, which the kit's ShellMenuPoint leaves out).
function ShowMorePoint($sec = 4) {
  $any = [Windows.Automation.Condition]::TrueCondition
  $byName = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::NameProperty, 'Show more options')
  $ex = @(Get-Process explorer -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
  for ($i = 0; $i -lt $sec * 2; $i++) {
    foreach ($w in @([Windows.Automation.AutomationElement]::RootElement.FindAll([Windows.Automation.TreeScope]::Children, $any))) {
      if ($ex -notcontains $w.Current.ProcessId -or $w.Current.ClassName -match '^(Progman|WorkerW|Shell_TrayWnd)$') { continue }
      $e = $w.FindFirst([Windows.Automation.TreeScope]::Descendants, $byName)
      if ($e) { $r = $e.Current.BoundingRectangle; Log "Show more options in $($w.Current.ClassName) at $r"; return @([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2)) }
    }
    Start-Sleep -Milliseconds 500
  }
  return $null
}

Case 'setup'
CloseWindows
Set-Culture en-US
CleanClip
New-Item -ItemType Directory -Force "$Clip\$Folder" | Out-Null
Set-Content "$Clip\会议记录.txt" 'notes'
PfStart
foreach ($i in 1..3) { Pf rule remove 0 | Out-Null }
Pf fence create --title '程序' --rect '800,90,380,200' | Out-Null
Pf fence create --title '快捷方式' --rect '800,320,380,200' | Out-Null
Pf rule add --name '程序' --type programs --to '程序' | Out-Null
Pf rule add --name '快捷方式' --type shortcuts --to '快捷方式' --index 1 | Out-Null
Pf fence set '程序' sort name | Out-Null
Pf fence set '快捷方式' sort name | Out-Null
# WScript.Shell saves only ANSI names and targets: write them as st-*.lnk, then rename.
Set-Content "$PfRun\notes.txt" 'notes'
$s = (New-Object -ComObject WScript.Shell).CreateShortcut("$PfDesk\st-np.lnk"); $s.TargetPath = "$env:WINDIR\System32\notepad.exe"; $s.Save()
$s = (New-Object -ComObject WScript.Shell).CreateShortcut("$PfDesk\st-mn.lnk"); $s.TargetPath = "$PfRun\notes.txt"; $s.Save()
Start-Sleep -Seconds 1
Rename-Item "$PfDesk\st-np.lnk" '记事本.lnk'
Rename-Item "$PfDesk\st-mn.lnk" '会议记录 - Shortcut.lnk'
Start-Sleep -Seconds 2
Check 'the two fences hold their first shortcut' ((ItemFence '记事本.lnk') -eq '程序' -and (ItemFence '会议记录 - Shortcut.lnk') -eq '快捷方式') "$(ItemFence '记事本.lnk') / $(ItemFence '会议记录 - Shortcut.lnk'); rules: $((Rules | ForEach-Object { "$($_.name) -> $($_.targetTitle)" }) -join '; ')"
Start-Process explorer.exe "`"$Clip`""
$w = $null
foreach ($k in 1..40) { foreach ($x in @((New-Object -ComObject Shell.Application).Windows())) { $f = try { $x.Document.Folder.Self.Path } catch { $null }; if ($f -eq $Clip) { $w = $x } }; if ($w) { break }; Start-Sleep -Milliseconds 250 }
$h = [IntPtr]$w.HWND
[L]::MoveWindow($h, 40, 90, 720, 430, $true) | Out-Null
Start-Sleep -Seconds 2
TitleClick $h
$el = UiaFind $h $Folder
$r = $el.Current.BoundingRectangle
$ix = [int]($r.X + 30); $iy = [int]($r.Y + $r.Height / 2)
Log "folder row at $ix,$iy ($r)"
[G]::MoveTo(600, 480)

Case 'record'
RecStart "sendto-$Side" -X 20 -Y 70 -W 1200 -H 470 -Fps 12
Start-Sleep -Milliseconds 1200
RecMark '在资源管理器里右键文件夹「项目资料」'
[G]::MoveTo($ix, $iy); Start-Sleep -Milliseconds 500
[G]::RClick($ix, $iy); Start-Sleep -Milliseconds 1200
$p = ShowMorePoint
if ($p) { [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 600; [G]::Click($p[0], $p[1]) }
else { Log 'Show more options not found: W key'; [G]::Press(0x57) }
$m = MenuWin 5
Start-Sleep -Milliseconds 600
RecMark '选「发送到 › 桌面快捷方式」'
$sp = MenuPoint $m '^Send to'
[G]::MoveTo($sp[0], $sp[1]); Start-Sleep -Milliseconds 1200
$sub = [IntPtr]::Zero
foreach ($k in 1..20) { foreach ($t in @([W]::Tops('#32768'))) { if ((MenuTexts $t) -match 'Desktop \(create shortcut\)') { $sub = $t } }; if ($sub -ne [IntPtr]::Zero) { break }; Start-Sleep -Milliseconds 250 }
$dp = MenuPoint $sub '^Desktop \(create shortcut\)'
[G]::MoveTo($sp[0] + 60, $dp[1]); Start-Sleep -Milliseconds 300
[G]::MoveTo($dp[0], $dp[1]); Start-Sleep -Milliseconds 700
[G]::Click($dp[0], $dp[1])
RecMark '新的快捷方式进了哪个栅栏'
[G]::MoveTo(760, 560)
Start-Sleep -Seconds 4
RecStop
$got = ItemFence $LinkFile
Shot "sendto-$Side-end"
Check "the folder's shortcut lands in 快捷方式 ($Side)" ($got -eq '快捷方式') "in '$got'; $((@(PfLogAll | Where-Object { $_ -match 'desktop item routed' }) | Select-Object -Last 1))"

Case 'cleanup'
CloseWindows
PfQuit
CleanClip
Remove-Item -LiteralPath $Clip -Recurse -Force -ErrorAction SilentlyContinue
