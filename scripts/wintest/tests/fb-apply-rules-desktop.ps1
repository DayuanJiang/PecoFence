# 「立即应用」 sorts what lies on the desktop, also an icon dragged out of a fence to be sorted again;
# an icon dragged into another fence stays; the notice's Undo puts the moved icon back. Both builds
# run one after the other in one job: BEFORE = --extra (main without the change), AFTER = --bin.
# On the BEFORE side the dragged shortcut stays on the desktop and no notice comes (those FAIL rows
# are the finding). Recordings in shots/rec-AR-before and shots/rec-AR-after for mkanim.py.
. C:\pf\kit\rec.ps1
$PfAllowLogErrors = $true
$Lnk = 'ar-notepad.lnk'
$Doc = 'ar-report.docx'

function NoticeWin() {
  $n = [G]::FindWindowW('PecoFence.Notice', [NullString]::Value)
  if ($n -ne [IntPtr]::Zero -and [G]::IsWindowVisible($n)) { return $n }
  return [IntPtr]::Zero
}

function Side($side) {
  trap { PfTrap $_; continue }
  if (PfAlive) { PfQuit }
  $script:PfBin = if ($side -eq 'before') { $PfExtra } else { 'C:\pf\bin' }
  $script:PfApp = "$PfRun\app-$side"
  New-Item -ItemType Directory -Force $PfApp | Out-Null
  foreach ($n in $Lnk, $Doc) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
  DeskLink $Lnk | Out-Null
  DeskFile $Doc | Out-Null
  PfStart
  Start-Sleep -Seconds 3
  # Room below the icons of 程序, so a drop there lands on no program (#198 opens files with it).
  Pf fence move Programs --rect '980,80,330,420' | Out-Null
  Pf fence move 'Files and documents' --rect '980,540,330,260' | Out-Null
  Start-Sleep -Seconds 2
  $l0 = WaitItemFence $Lnk 'Programs'
  $d0 = WaitItemFence $Doc 'Files and documents'
  Check "${side}: the rules put the shortcut in Programs, the document in Files and documents" ($l0 -eq 'Programs' -and $d0 -eq 'Files and documents') "$l0 / $d0"
  CloseNotice | Out-Null
  [G]::MoveTo(700, 450); Start-Sleep -Milliseconds 500

  RecStart "AR-$side" -Fps 10
  RecMark '把放错的快捷方式拖到桌面上'
  $p = ItemPt 'Programs' 'ar-notepad'
  $to = FreeSpot 640 380
  DragTo $p[0] $p[1] $to[0] $to[1] 18
  Start-Sleep -Milliseconds 900
  RecMark '把文档拖进「程序」（故意放这里）'
  $q = ItemPt 'Files and documents' 'ar-report'
  if (-not $q) { $q = ItemPt 'Files and documents' $Doc }
  $w = (PfFence 'Programs').windowRect
  DragTo $q[0] $q[1] ([int]($w.x + $w.w / 2)) ([int]($w.y + $w.h - 50)) 18
  Start-Sleep -Milliseconds 900
  $l1 = ItemFence $Lnk; $d1 = ItemFence $Doc
  Check "${side}: after the drags the shortcut is on the Desktop, the document in Programs" ($l1 -eq 'Desktop' -and $d1 -eq 'Programs') "$l1 / $d1"
  RecMark '托盘菜单：立即应用整理规则'
  $picked = TrayPick 'Apply sorting rules now'
  Start-Sleep -Seconds 2
  $l2 = ItemFence $Lnk; $d2 = ItemFence $Doc
  $n = NoticeWin
  Check "${side}: Apply now puts the shortcut dragged to the desktop back into Programs" ($l2 -eq 'Programs') "picked '$picked'; shortcut in $l2"
  Check "${side}: Apply now leaves the document put into Programs by hand" ($d2 -eq 'Programs') "document in $d2"
  Check "${side}: a notice offers Undo" ($n -ne [IntPtr]::Zero) "notice=$n $(if ($n -ne [IntPtr]::Zero) { [W]::Info($n) })"
  Start-Sleep -Seconds 2
  RecMark '点提示里的「撤销」'
  if ($n -ne [IntPtr]::Zero) { $r = Rect $n; [G]::Click($r.R - 50, $r.B - 30) } else { Start-Sleep -Milliseconds 800 }
  Start-Sleep -Seconds 2
  [G]::MoveTo(700, 450)
  Start-Sleep -Seconds 1
  RecStop
  $l3 = ItemFence $Lnk; $d3 = ItemFence $Doc
  Check "${side}: Undo puts the shortcut back on the Desktop and keeps the document in Programs" ($l3 -eq 'Desktop' -and $d3 -eq 'Programs') "$l3 / $d3"
  Shot "AR-$side-end"
  PfQuit
  foreach ($n in $Lnk, $Doc) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
}

Case 'before (main)'
Side 'before'
Case 'after (this change)'
Side 'after'
