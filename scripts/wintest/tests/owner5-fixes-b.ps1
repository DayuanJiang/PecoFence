# Owner round 2026-11-12, fifth batch, package ux5-fixes-b: what two readers of the merged
# branch found (notes/fix/ux5-read-b and ux5-read-c). Parts (run.sh --var Part=...):
#   S  a configuration from somebody else: its scripts start only after the user's own turn of
#      the 小工具 switch. Not by a new note or clock (page, command line), and not when a
#      snapshot brings the imported layout back later. The script is harmless: a .cmd that
#      writes a marker file into the job's run folder.
#   D  a snapshot deleted on the settings page and brought back with 撤销 keeps the note
#      texts and script paths of its fences
#   P  a note made from the settings page lands off the fence beside the window, also while
#      「桌面」 (arranged like the desktop) spans the whole screen
#   K  AltGr with a digit or a letter in the Peek search box opens no result and no menu
#      (Windows reports AltGr as Ctrl and Alt)
#   L  the tip of a shortcut whose link file is not on this disk (offline attribute, as a
#      cloud file that is only online has it) is made without reading the file
#   G  hover switching between two tabs of which one has 按时间分组 on: the second switch
#      stands at once, no cross-fade
#   T  settings page: after a click on a colour swatch the element under the pointer has no
#      `title` again (the browser would show its own tip)
#   C  a double-click on 「下一条」 of the first-start tips goes on by one tip, not by two
# S, D, P and T drive the settings page over CDP: run them on a debug build.
# Runs on a 1600x900 box at 100 %.
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'SDPKLGTC' }
trap { PfTrap $_; continue }

function O5Tools() { return (PfJson settings get).tools }
function O5WaitFile($path, $sec) { for ($i = 0; $i -lt $sec * 2 -and -not (Test-Path $path); $i++) { Start-Sleep -Milliseconds 500 }; return (Test-Path $path) }
function O5Fences($title) { return @(PfJson fence list | Where-Object { $_ -and $_.title -eq $title }).Count }
function O5Newest() { return @(Snaps | Sort-Object { [long]$_.ts } -Descending)[0] }
function O5SettingsWin() { return [G]::FindWindowW('PecoFence.Settings', [NullString]::Value) }
# The lines the app has logged since $stamp (UTC, yyyy-MM-ddTHH:mm:ss).
function O5LogSince($stamp) { return @(PfLogAll | Where-Object { $_.Length -gt 19 -and [string]::CompareOrdinal($_.Substring(0, 19), $stamp) -ge 0 }) }
function O5Now() { return (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ss') }
# The middle of tab $i of the strip of dump $d.
function O5TabPt($d, $i) { return @([int]($d.l + $d.ox + $d.rects[$i][0] + $d.rects[$i][1] / 2), [int]($d.t + $d.oy + 18)) }
# The tooltip that is on screen: its rectangle, or $null.
function O5Tip() {
  foreach ($h in @([W]::Tops('tooltips_class32'))) { $r = Rect $h; if ($r.R - $r.L -gt 20) { return $r } }
  return $null
}
$O5Files = @('测试报告一.txt', '测试报告二.txt', '测试报告三.txt')
function O5Clean() {
  foreach ($n in $O5Files) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
  Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue | Where-Object { $_.Name -match '^o5 ' } | Remove-Item -Force -ErrorAction SilentlyContinue
}
# A first start on "another computer", in Chinese, with a note of the user's own.
function O5Another() {
  PfFresh
  Start-Sleep -Seconds 3
  Pf settings set language zh-CN | Out-Null
  CloseNotice | Out-Null
  Pf fence create --tool note --title '我的便签' --text '自己的' --rect '420,140,300,200' | Out-Null
  Start-Sleep -Seconds 1
}

Case 'setup'
CloseWindows
O5Clean
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'

# ---- S: imported scripts wait for the user's own turn of the switch
function O5Import() {
  trap { PfTrap $_; continue }
  Case 'S0 somebody''s configuration with a script fence'
  $Mark = "$PfRun\o5-script-ran.txt"
  $S = "$PfRun\o5-script.cmd"
  Set-Content $S -Value "@echo ran>> `"$Mark`"`r`n@echo hello" -Encoding ASCII
  PfStart -Lang zh-CN
  Start-Sleep -Seconds 3
  Pf fence create --script $S --title 'Runner' --rect '420,440,300,160' | Out-Null
  Check 'the script fence runs its script on the computer it was made on' (O5WaitFile $Mark 15) ''
  $theirs = "$PfRun\o5-theirs.json"; Pf config export $theirs | Out-Null

  Case 'S1 imported on another computer, then a clock made by the command line'
  O5Another
  $rep = PfJson config import $theirs; Start-Sleep -Seconds 3
  Remove-Item $Mark -ErrorAction SilentlyContinue
  Check 'the import holds the script: 小工具 is off and the reply counts it' ((O5Tools).enabled -eq $false -and $rep.scriptsHeld -eq 1) "tools.enabled=$((O5Tools).enabled) reply=$($rep | ConvertTo-Json -Compress)"
  $out = "$(Pf fence create --tool clock --title '时钟' --rect '820,140,240,140')"
  Start-Sleep -Seconds 6
  Check 'S1 a clock made by the command line starts no imported script' (-not (Test-Path $Mark)) "script ran=$(Test-Path $Mark) tools.enabled=$((O5Tools).enabled)"
  Check 'S1 the command line says which script waits' ($out -match 'o5-script\.cmd' -and (O5Fences '时钟') -eq 0) $out.Trim()

  # A first start again: a script that ran a moment ago in this run is not due for its
  # interval, whoever turns the switch.
  Case 'S2 the same file, then 新建便签 and the clock''s 添加 on the settings page'
  O5Another
  Pf config import $theirs | Out-Null; Start-Sleep -Seconds 3
  Remove-Item $Mark -ErrorAction SilentlyContinue
  Check 'settings open over CDP' (PfSettings)
  PageClick 'nav a[data-page=tools]'; Start-Sleep -Milliseconds 600
  $held = "$(Js "(document.getElementById('toolsHeld') || { textContent: '' }).textContent")"
  Shot 's2-tools-page-held'
  Check 'S2 the card of the 小工具 switch lists the script that waits' ($held -match 'o5-script\.cmd') "card: [$held]"
  PageClick '#newNote'; Start-Sleep -Seconds 5
  $toast = PageToast
  Check 'S2 新建便签 starts no imported script' (-not (Test-Path $Mark)) "script ran=$(Test-Path $Mark) tools.enabled=$((O5Tools).enabled) notice=[$($toast.text)]"
  Check 'S2 the page says that the scripts wait for the switch' ($toast.text -match '脚本' -and (O5Fences '便签') -eq 0) "notice=[$($toast.text)] new notes=$(O5Fences '便签')"
  if (-not (Test-Path $Mark)) {
    PageClick '#newClock'; Start-Sleep -Seconds 5
    Check 'S2 the clock''s 添加 starts no imported script' (-not (Test-Path $Mark)) "script ran=$(Test-Path $Mark) tools.enabled=$((O5Tools).enabled)"
  }

  Case 'S3 the user turns 小工具 on himself'
  if (-not (O5Tools).enabled) { PageClick '[data-bind="tools.enabled"]' }
  Check 'S3 the script starts with the user''s own turn of the switch' (O5WaitFile $Mark 15) "tools.enabled=$((O5Tools).enabled)"
  $hidden = Js "(document.getElementById('toolsHeld') || { hidden: true }).hidden"
  Check 'S3 the card no longer lists a waiting script' ($hidden -eq $true) "hidden=$hidden"
  PageClick '#newNote'; Start-Sleep -Seconds 2
  [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  Check 'S3 新建便签 makes a note again' ((O5Fences '便签') -ge 1) "notes titled 便签: $(O5Fences '便签')"
  $json = [IO.File]::ReadAllText((PfJson status).configPath)
  Check 'S3 the saved configuration holds no script any more' ($json -notmatch '"held"') ''

  Case 'S4 the import undone, 小工具 on for the own note, then the snapshot of the imported layout restored'
  O5Another
  Pf config import $theirs | Out-Null; Start-Sleep -Seconds 3
  $undo = O5Newest
  Pf snapshot restore "$($undo.id)" | Out-Null; Start-Sleep -Seconds 3
  Remove-Item $Mark -ErrorAction SilentlyContinue
  Pf settings set tools.enabled true | Out-Null; Start-Sleep -Seconds 6
  Check 'S4 the user''s own layout is back and 小工具 on: the imported script does not run' ((O5Fences 'Runner') -eq 0 -and (O5Fences '我的便签') -eq 1 -and -not (Test-Path $Mark)) "Runner=$(O5Fences 'Runner') own note=$(O5Fences '我的便签') script ran=$(Test-Path $Mark)"
  $imp = O5Newest
  $rep = PfJson snapshot restore "$($imp.id)"; Start-Sleep -Seconds 2
  Check 'S4 the snapshot taken before that restore brings the imported script fence back' ((O5Fences 'Runner') -eq 1) "snapshot '$($imp.name)'"
  # The minute housekeeping starts every script that is due.
  $ran = O5WaitFile $Mark 80
  Check 'S4 the script that came back with the snapshot does not run' (-not $ran) "script ran=$ran within 80 s; tools.enabled=$((O5Tools).enabled)"
  Check 'S4 小工具 is off again and the reply counts the script' ((O5Tools).enabled -eq $false -and $rep.scriptsHeld -eq 1) "tools.enabled=$((O5Tools).enabled) reply=$($rep | ConvertTo-Json -Compress)"
  Pf settings set tools.enabled true | Out-Null
  Check 'S4 it starts with the user''s own turn of the switch' (O5WaitFile $Mark 15) ''
  PfQuit
  Remove-Item $Mark, $S, $theirs -ErrorAction SilentlyContinue
}

# ---- D: a deleted snapshot's undo
function O5Deleted() {
  trap { PfTrap $_; continue }
  Case 'D0 a note and a script fence that only one snapshot still has'
  $Hello = "$PfRun\o5-hello.cmd"; Set-Content $Hello -Value '@echo hello' -Encoding ASCII
  PfFresh
  Start-Sleep -Seconds 3
  Pf settings set language zh-CN | Out-Null
  CloseNotice | Out-Null
  Pf fence create --tool note --title '买菜' --text '[ ] 牛奶 和 鸡蛋' --rect '420,140,300,200' | Out-Null
  Pf fence create --script $Hello --title '问候' --rect '420,380,300,160' | Out-Null
  Start-Sleep -Seconds 2
  $mine = "$((PfJson snapshot save '有便签和脚本').snapshot.id)"
  Pf fence delete '买菜' | Out-Null; Pf fence delete '问候' | Out-Null
  # The command line's automatic snapshots have the two fences too: they go, so that the saved
  # snapshot is the only place the note and the script are still wanted.
  foreach ($s in @(Snaps | Where-Object { "$($_.id)" -ne $mine })) { Pf snapshot delete "$($s.id)" | Out-Null }
  Start-Sleep -Seconds 2
  Check 'one snapshot is left and the two fences are gone' (@(Snaps).Count -eq 1 -and (O5Fences '买菜') -eq 0 -and (O5Fences '问候') -eq 0) "snapshots=$(@(Snaps).Count)"
  Check 'settings open over CDP' (PfSettings)
  PageClick 'nav a[data-page=layout]'; Start-Sleep -Milliseconds 600

  Case 'D1 the snapshot deleted on the page, a second later 撤销'
  PageClick "#snapList .card[data-row-id=`"$mine`"] [data-act=del]"
  # The configuration is saved 0.8 s after the delete.
  Start-Sleep -Milliseconds 2500
  Check 'the snapshot is gone and the notice offers 撤销' (@(Snaps).Count -eq 0 -and (Js "!document.getElementById('toastUndo').hidden")) "snapshots=$(@(Snaps).Count) notice=[$((PageToast).text)]"
  PageClick '#toastUndo'; Start-Sleep -Seconds 1
  Check '撤销 puts the snapshot back in the list' (@(Snaps).Count -eq 1) "snapshots=$(@(Snaps).Count)"
  Pf snapshot restore $mine | Out-Null; Start-Sleep -Seconds 2
  $note = "$((FenceGet '买菜').noteText)"; $path = "$((FenceGet '问候').script.path)"
  Shot 'd1-restored'
  Check 'D1 the note restored from that snapshot has its text' ($note -match '牛奶') "note text: [$note]"
  Check 'D1 the script fence restored from that snapshot has its script' ($path -match 'o5-hello') "script: [$path]"
  PfQuit
  Remove-Item $Hello -ErrorAction SilentlyContinue
}

# ---- P: a new note from the settings page while 「桌面」 spans the screen
function O5Place() {
  trap { PfTrap $_; continue }
  Case 'P0 「桌面」 spans the screen, a fence stands beside the settings window'
  DeskFile 'o5 右下角.txt' | Out-Null
  PfFresh
  Start-Sleep -Seconds 3
  Pf settings set language zh-CN | Out-Null
  CloseNotice | Out-Null
  ClearFences | Out-Null
  Pf fence set inbox desktopLayout true | Out-Null
  Pf fence set inbox freePlacement true | Out-Null
  Start-Sleep -Seconds 2
  # An icon in the bottom right corner, as a Recycle Bin kept there: the window of 「桌面」
  # reaches from the top left of the work area to it.
  $ip = ItemPt 'inbox' 'o5 右下角'
  DragTo $ip[0] $ip[1] 1500 760
  Start-Sleep -Seconds 2
  $desk = (FenceGet 'inbox').windowRect
  Check 'the window of 「桌面」 spans most of the screen' ($desk.w -ge 1400 -and $desk.h -ge 700) "window $($desk.x),$($desk.y) $($desk.w)x$($desk.h)"
  Check 'settings open over CDP' (PfSettings)
  $sr = Rect (O5SettingsWin)
  $bx = if (1600 - $sr.R -ge $sr.L) { $sr.R + 12 } else { $sr.L - 12 - 300 }
  Pf fence create --title '挡路的' --rect "$bx,$($sr.T),300,400" | Out-Null
  Start-Sleep -Seconds 1
  PageClick 'nav a[data-page=tools]'; Start-Sleep -Milliseconds 600

  Case 'P1 新建便签 on the settings page'
  PageClick '#newNote'; Start-Sleep -Seconds 2
  [G]::Press(0x1B); Start-Sleep -Milliseconds 600
  $n = (FenceGet '便签').windowRect; $b = (FenceGet '挡路的').windowRect
  $ox = [math]::Max(0, [math]::Min($n.x + $n.w, $b.x + $b.w) - [math]::Max($n.x, $b.x))
  $oy = [math]::Max(0, [math]::Min($n.y + $n.h, $b.y + $b.h) - [math]::Max($n.y, $b.y))
  $line = @(PfLogAll | Where-Object { $_ -match 'new fence from the settings page' }) | Select-Object -Last 1
  Shot 'p1-new-note'
  Check 'P1 the new note does not lie on the fence beside the window' ($n -and $ox * $oy -eq 0) "note $($n.x),$($n.y) $($n.w)x$($n.h); fence $($b.x),$($b.y) $($b.w)x$($b.h); overlap ${ox}x${oy} px"
  Check 'P1 the log: a spot off every fence was found' ($line -notmatch 'no spot off the fences') ("$line" -replace '^\S+\s+\S+\s+', '')
  PfQuit
  O5Clean
}

# ---- K: AltGr in the Peek search box
function O5AltGr() {
  trap { PfTrap $_; continue }
  Case 'K AltGr with a digit or a letter in the Peek search box'
  foreach ($n in $O5Files) { Set-Content -LiteralPath (Join-Path $PfDesk $n) -Value 'sample' }
  PfFresh
  Start-Sleep -Seconds 3
  Pf settings set language zh-CN | Out-Null
  CloseNotice | Out-Null
  Start-Sleep -Seconds 1
  DeskClick 1500 820 | Out-Null
  PeekKey; Start-Sleep -Milliseconds 900
  [G]::Type('csbg'); Start-Sleep -Milliseconds 900
  Check 'the search is open with three results' ([M]::BoxText() -eq 'csbg') "box=[$([M]::BoxText())]"
  # What Windows reports for AltGr+2 (@ on a Spanish keyboard): Ctrl and Alt down, then 2.
  [G]::KeyDown(0x11); [G]::KeyDown(0x12); [G]::Press(0x32); [G]::KeyUp(0x12); [G]::KeyUp(0x11)
  Start-Sleep -Seconds 3
  $np = NotepadTitles; $box = [M]::BoxText()
  Shot 'k-after-altgr-2'
  Check 'K AltGr+2 opens no result and leaves the search open' (-not $np -and $box -eq 'csbg') "notepad=[$np] box=[$box]"
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force; Start-Sleep -Milliseconds 500
  if ($box -eq $null) {
    DeskClick 1500 820 | Out-Null
    PeekKey; Start-Sleep -Milliseconds 900
    [G]::Type('csbg'); Start-Sleep -Milliseconds 900
  }
  # AltGr+O (ó on a Polish keyboard) is no Ctrl+O: the actions of the row do not open.
  [G]::KeyDown(0x11); [G]::KeyDown(0x12); [G]::Press(0x4F); [G]::KeyUp(0x12); [G]::KeyUp(0x11)
  Start-Sleep -Milliseconds 900
  $menu = MenuWin 1
  Check 'K AltGr+O opens no action menu' ($menu -eq [IntPtr]::Zero) "menu window=$menu"
  CloseMenus
  # Ctrl+2 alone still opens the second row.
  if ([M]::BoxText() -eq $null) {
    DeskClick 1500 820 | Out-Null
    PeekKey; Start-Sleep -Milliseconds 900
    [G]::Type('csbg'); Start-Sleep -Milliseconds 900
  }
  Chord 0x11 0x32; Start-Sleep -Seconds 3
  $np2 = NotepadTitles
  Check 'K Ctrl+2 alone still opens the second row' ($np2 -like '*测试报告*') "notepad=[$np2]"
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  for ($i = 0; $i -lt 3 -and [M]::BoxText() -ne $null; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 400 }
  PfQuit
  O5Clean
}

# ---- L: the tip of a shortcut whose link file is not on this disk
function O5LinkTip() {
  trap { PfTrap $_; continue }
  Case 'L the tip of a shortcut whose link file is not on this disk'
  $ld = "$PfRun\o5-links"
  New-Item -ItemType Directory -Force $ld | Out-Null
  $wsh = New-Object -ComObject WScript.Shell
  $la = $wsh.CreateShortcut("$ld\o5 a here.lnk"); $la.TargetPath = "$env:WINDIR\notepad.exe"; $la.Description = 'Write short notes'; $la.Save()
  $lb = $wsh.CreateShortcut("$ld\o5 b cloud.lnk"); $lb.TargetPath = "$env:WINDIR\notepad.exe"; $lb.Description = 'Kept in the cloud'; $lb.Save()
  # The offline attribute, which a cloud file that is only online carries too.
  & attrib.exe +O "$ld\o5 b cloud.lnk" | Out-Null
  Log ("attributes: " + ((Get-ChildItem -LiteralPath $ld | ForEach-Object { "$($_.Name)=0x{0:x}" -f [int]$_.Attributes }) -join ' '))
  PfFresh
  Start-Sleep -Seconds 3
  Pf settings set language en | Out-Null
  CloseNotice | Out-Null
  Pf settings set featureTips false | Out-Null
  Pf fence create --portal $ld --title 'O5L' --rect '300,150,500,330' | Out-Null
  Pf fence set O5L sort name | Out-Null; Start-Sleep -Seconds 3
  $tips = @()
  foreach ($i in 0..1) {
    $pt = CellPt 'O5L' $i
    [G]::MoveTo($pt[0] - 4, $pt[1]); Start-Sleep -Milliseconds 60; [G]::MoveTo($pt[0], $pt[1]); Start-Sleep -Milliseconds 1800
    $t = O5Tip
    $text = if ($t) { OcrText ($t.L - 2) ($t.T - 2) ($t.R - $t.L + 4) ($t.B - $t.T + 4) "l-tip-ocr-$i" } else { '' }
    $tips += , "$text"
    ParkMouse; Start-Sleep -Milliseconds 700
  }
  $skipped = @(PfLogAll | Where-Object { $_ -match 'shortcut tip: the link file is not on this disk, not read' }).Count
  Check 'L a shortcut on this disk: the tip shows its comment' ($tips[0] -match 'Write short notes') "tip: [$($tips[0])]"
  Check 'L a shortcut that is not on this disk: the tip is made without reading it (type, size, date)' ($tips[1] -notmatch 'Kept in the cloud' -and $tips[1] -match 'Size|Type|Date') "tip: [$($tips[1])]"
  Check 'L the log says the link file was not read' ($skipped -ge 1) "lines: $skipped"
  Pf fence delete O5L | Out-Null
  PfQuit
  & attrib.exe -O "$ld\o5 b cloud.lnk" | Out-Null
  Remove-Item -Recurse -Force $ld -ErrorAction SilentlyContinue
}

# ---- G: hover switching between tabs that differ in 按时间分组
function O5HoverGroup() {
  trap { PfTrap $_; continue }
  Case 'G hover switching from a tab without 按时间分组 to one with it'
  $animWas = [Anim]::Get(); [Anim]::Set($true)
  # Fourteen files of fourteen different months, two more, and three pictures.
  foreach ($i in 1..14) {
    $f = DeskFile ("o5 报告 {0:D2}.pdf" -f $i)
    (Get-Item -LiteralPath $f).LastWriteTime = (Get-Date).AddDays(-31 * $i)
  }
  foreach ($n in 'o5 说明.txt', 'o5 记录.txt', 'o5 图 一.png', 'o5 图 二.png', 'o5 图 三.png') { DeskFile $n | Out-Null }
  PfStart -Lang zh-CN -Script (DumpScript 250 6)
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  ClearFences | Out-Null
  Pf fence create --title '文字' --rect '200,120,520,330' | Out-Null
  Pf item move --glob 'o5*.txt' --to '文字' | Out-Null
  Pf fence create --title '报告' --rect '760,120,400,240' | Out-Null
  Pf item move --glob 'o5*.pdf' --to '报告' | Out-Null
  Pf fence create --title '图片' --rect '1180,120,340,240' | Out-Null
  Pf item move --glob 'o5*.png' --to '图片' | Out-Null
  Pf fence set '图片' groupByDate true | Out-Null
  Pf fence merge '报告' --into '文字' | Out-Null
  Pf fence merge '图片' --into '文字' | Out-Null
  Pf settings set rollUp.switchTabOnHover true | Out-Null
  Start-Sleep -Seconds 2
  $d = DumpOf '文字' 4
  $iText = [array]::IndexOf(@($d.tabs), '文字'); $iRep = [array]::IndexOf(@($d.tabs), '报告'); $iPic = [array]::IndexOf(@($d.tabs), '图片')
  $pt = O5TabPt $d $iText; [G]::Click($pt[0], $pt[1]); Start-Sleep -Milliseconds 700
  ParkMouse 1300 800; Start-Sleep -Seconds 2
  # A rest on the tab of the reports (the first hover switch, with its swap), then on to the
  # pictures within a second: shown at once.
  $pt = O5TabPt $d $iRep; [G]::MoveTo($pt[0], $pt[1]); Start-Sleep -Milliseconds 1200
  $stamp = O5Now; Start-Sleep -Milliseconds 1100
  $pt = O5TabPt $d $iPic; [G]::MoveTo($pt[0], $pt[1]); Start-Sleep -Milliseconds 900
  $shown = (DumpOf '文字' 4).title
  ParkMouse 1300 800; Start-Sleep -Milliseconds 800
  $lines = O5LogSince $stamp
  $atOnce = @($lines | Where-Object { $_ -match 'tab hover switch' -and $_ -match 'at_once=true' }).Count
  # The grouping differs from the last tab's: its icons must neither cross-fade nor glide to
  # the new arrangement before the next tab's list replaces them.
  $fades = @($lines | Where-Object { $_ -match 'group by date changed: (the content cross-fades|icons glide)' } | ForEach-Object { $_ -replace '^\S+\s+\S+\s+', '' })
  Check 'G setup: the second switch was shown at once' ($atOnce -ge 1) "hover switches shown at once: $atOnce; tabs $($d.tabs -join ','); title $shown"
  Check 'G the page of the next tab stands at once: the last tab''s icons are not moved or faded for the grouping' ($fades.Count -eq 0) "$($fades -join ' | ')"
  Pf settings set rollUp.switchTabOnHover false | Out-Null
  [Anim]::Set($animWas)
  PfQuit
  O5Clean
}

# ---- T: the settings page's own tip after a click on a colour swatch
function O5SwatchTip() {
  trap { PfTrap $_; continue }
  Case 'T a click on a colour swatch, the pointer stays on it'
  PfFresh
  Start-Sleep -Seconds 3
  Pf settings set language zh-CN | Out-Null
  CloseNotice | Out-Null
  Check 'settings open over CDP' (PfSettings)
  $sel = '#iconTintSwatches .swatch:not(.custom):not(.none)'
  $page = Js "document.querySelector('#iconTintSwatches').closest('.page').id.replace('page-', '')"
  PageClick "nav a[data-page=$page]"; Start-Sleep -Milliseconds 600
  $r = PageRect $sel 1
  [G]::MoveTo($r.cx - 3, $r.cy); Start-Sleep -Milliseconds 200; [G]::MoveTo($r.cx, $r.cy); Start-Sleep -Milliseconds 700
  $before = Js "document.querySelectorAll('$sel')[1].hasAttribute('title')"
  [G]::Click($r.cx, $r.cy); Start-Sleep -Milliseconds 900
  [G]::MoveTo($r.cx + 2, $r.cy); Start-Sleep -Milliseconds 900
  $after = Js "document.querySelectorAll('$sel')[1].hasAttribute('title')"
  $chosen = "$((PfJson settings get).icons.tintRgb)"
  Shot 't-swatch-after-click'
  Check 'T while the pointer rests on the swatch it has no title (the page draws the tip)' ($before -eq $false) "title there: $before"
  Check 'T after the click the swatch under the pointer still has no title' ($after -eq $false -and "$chosen" -ne '') "title there: $after; icon colour now: [$chosen]"
  # Off the swatch the title is back for the next time.
  $rest = PageRect '.brand'; [G]::MoveTo($rest.cx, $rest.cy); Start-Sleep -Milliseconds 500
  $back = Js "document.querySelectorAll('$sel')[1].getAttribute('title')"
  Check 'T off the swatch its title is back' ("$back" -ne '') "title: [$back]"
  PfQuit
}

# ---- C: a double-click on a card's button
function O5CardDouble() {
  trap { PfTrap $_; continue }
  Case 'C a double-click on 下一条 of the first-start tips'
  # A real first start (no test script, no notices kept): the tips card shows.
  if (PfAlive) { PfQuit }
  Remove-Item -Recurse -Force "$PfApp\config", "$PfApp\notices.json" -ErrorAction SilentlyContinue
  if (-not (Test-Path "$PfApp\pecofence.exe")) { New-Item -ItemType Directory -Force $PfApp | Out-Null; Copy-Item "$PfBin\*" $PfApp -Force }
  $env:PECOFENCE_INSTANCE = $PfInst
  $env:RUST_LOG = 'pecofence=debug'
  [G]::MoveTo(700, 450)
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $PfApp
  WaitWin 'PecoFence.Control' 20 -Hidden | Out-Null
  $n = WaitWin 'PecoFence.Notice' 30
  Start-Sleep -Seconds 2
  Check 'the first tip shows' ($n -ne [IntPtr]::Zero) "notice=$n"
  $r = Rect $n
  $t0 = [L]::Text($n)
  # The number of the tip a card's title names ("Tip 2/4: ...").
  function TipNo($title) { if ("$title" -match '(\d+)\s*/\s*\d+') { return [int]$Matches[1] }; return 0 }
  # The pointer on the button, then two clicks 120 ms apart.
  [G]::MoveTo($r.R - 46, $r.B - 32); Start-Sleep -Milliseconds 500
  DoubleClick ($r.R - 46) ($r.B - 32)
  Start-Sleep -Milliseconds 1200
  $t1 = [L]::Text([G]::FindWindowW('PecoFence.Notice', [NullString]::Value))
  $by = (TipNo $t1) - (TipNo $t0)
  Shot 'c-after-double-click'
  Check 'C a double-click on 下一条 goes on by one tip' ($by -eq 1) "the double-click went on by $by ($t0 -> $t1)"
  # A click of its own after that goes on as before.
  [G]::Click($r.R - 46, $r.B - 32); Start-Sleep -Milliseconds 1200
  $t2 = [L]::Text([G]::FindWindowW('PecoFence.Notice', [NullString]::Value))
  Check 'C a click after it shows the next tip' ((TipNo $t2) - (TipNo $t1) -eq 1) "$t1 -> $t2"
  PfQuit
}

if ($Part -match 'S') { O5Import }
if ($Part -match 'D') { O5Deleted }
if ($Part -match 'P') { O5Place }
if ($Part -match 'K') { O5AltGr }
if ($Part -match 'L') { O5LinkTip }
if ($Part -match 'G') { O5HoverGroup }
if ($Part -match 'T') { O5SwatchTip }
if ($Part -match 'C') { O5CardDouble }

Case 'cleanup'
if (PfAlive) { PfQuit }
CloseWindows
O5Clean
