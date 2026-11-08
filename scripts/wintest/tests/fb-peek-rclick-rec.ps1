# Feedback #124 before/after recordings. Same steps on both builds: --var Rec=before (old
# build) or Rec=after. Clip=rclick (default): a right-click on a Peek search result, frames in
# shots/rec-rc-<Rec>. Clip=props: Ctrl+O, Properties (where its window opens), in
# shots/rec-props-<Rec>.
if (-not $Rec) { $Rec = 'after' }
if (-not $Clip) { $Clip = 'rclick' }
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
if ($Clip -eq 'props') { RecStart "props-$Rec" -X 320 -Y 160 -W 960 -H 700 -Fps 15 }
else { RecStart "rc-$Rec" -X 320 -Y 160 -W 960 -H 640 -Fps 15 }
Start-Sleep -Milliseconds 600
RecMark '速览时输入 xmjh'
PeekKey; Start-Sleep -Milliseconds 700
[G]::Type('xmjh'); Start-Sleep -Milliseconds 1500
$b = [G]::FindWindowW('PecoFence.PeekSearch', [NullString]::Value)
$r = Rect $b
Log "box $($r.L),$($r.T),$($r.R),$($r.B)"
if ($Clip -eq 'props') {
  RecMark '按 Ctrl+O，在菜单里选 Properties（属性）'
  Chord 0x11 0x4F; Start-Sleep -Milliseconds 1200
  $p = MenuPoint (MenuWin 3) '^Properties$'
  if ($p) { Glide ($r.L + 40) ($r.T + 120) $p[0] $p[1] 14 30; Start-Sleep -Milliseconds 300; [G]::Click($p[0], $p[1]) }
  RecMark '看属性窗口'
  Start-Sleep -Milliseconds 3500
  $pw = @([W]::Tops('#32770') | Where-Object { [W]::Title($_) -like '*Properties' }) | Select-Object -First 1
  Check "Properties window ($Rec)" ([bool]$pw) "class at its middle=$(if ($pw) { $q = Rect $pw; [W]::RootClassAt([int](($q.L + $q.R) / 2), [int](($q.T + $q.B) / 2)) })"
  RecStop
  if ($pw) { [L]::PostMessageW($pw, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; Start-Sleep -Milliseconds 600 }
  for ($i = 0; $i -lt 3 -and [M]::BoxText() -ne $null; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 400 }
} else {
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
}

Case 'cleanup'
PfQuit
CloseWindows
Remove-Item -LiteralPath $f -ErrorAction SilentlyContinue
