# Feedback #124 before/after recording: a right-click on a Peek search result. Same steps on
# both builds: --var Rec=before (old build) or Rec=after; frames in shots/rec-rc-<Rec>.
if (-not $Rec) { $Rec = 'after' }
. C:\pf\kit\rec.ps1
Case "record $Rec"
CloseWindows
Set-Culture zh-CN
$f = Join-Path $PfDesk '项目计划.txt'
Set-Content -LiteralPath $f 'plan'
PfStart -Lang zh-CN
Pf settings set theme light | Out-Null
Start-Sleep -Seconds 2
[G]::Click(900, 800); Start-Sleep -Milliseconds 400
[G]::MoveTo(1000, 700)
RecStart "rc-$Rec" -X 320 -Y 160 -W 960 -H 640 -Fps 15
Start-Sleep -Milliseconds 600
RecMark '速览时输入 xmjh'
PeekKey; Start-Sleep -Milliseconds 700
[G]::Type('xmjh'); Start-Sleep -Milliseconds 1500
$b = [G]::FindWindowW('PecoFence.PeekSearch', [NullString]::Value)
$r = Rect $b
Log "box $($r.L),$($r.T),$($r.R),$($r.B)"
RecMark '在找到的文件上点右键'
Glide 1000 700 ($r.L + 200) ($r.T + 82) 14 30
Start-Sleep -Milliseconds 300
[G]::RClick($r.L + 200, $r.T + 82); Start-Sleep -Milliseconds 300
RecMark '看看出现了什么'
Start-Sleep -Milliseconds 2600
$m = MenuWin 1
Check "menu after the right-click ($Rec)" $true "menu=$m items=$(MenuTexts $m)"
RecStop
CloseMenus
for ($i = 0; $i -lt 3 -and [M]::BoxText() -ne $null; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 400 }

Case 'cleanup'
PfQuit
CloseWindows
Remove-Item -LiteralPath $f -ErrorAction SilentlyContinue
