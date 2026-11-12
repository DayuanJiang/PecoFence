# The desktop side of the app in every interface language: menus, the search box, the marquee's
# buttons, the name box, tool fences and empty states, captured per language for a contact sheet.
#   run.sh --var Part=C --var Langs=zh-CN,en,ja,zh-TW ...   captures of these languages (about
#                                                 100 s each; --var Only=menu: the fence menu alone)
#   run.sh --var Part=D ...   details headings, a rolled fence's count, the filter box and its tag,
#                             date group captions, the box that asks before a fence is deleted
#   run.sh --var Part=E ...   the search box with Everything's rows and the two action rows
#   run.sh --var Part=R ...   checks: a rolled fence's count beside a long name, the filter box's
#                             count and offered type, the search box when nothing matches
#   run.sh --var Part=L ...   checks: texts a window keeps follow a language switch; a long status
#                             line of a script fence shows whole
# The crops are named <surface>-<lang>.png; menu texts go to job.log as "TEXT <surface> <lang>: ...".

if (-not $Part) { $Part = 'C' }
if (-not $Langs) { $Langs = 'zh-CN,en,ja,zh-TW,ko,de,fr,es,pt-BR,ru,it' }
$LangList = @($Langs -split '[,; ]+' | Where-Object { $_ })

function HlMenuLine($h) {
  return ((@([W]::MenuItems($h) -split "`n") | Where-Object { $_ } | ForEach-Object { $p = $_ -split "`t"; if ($p[1]) { $p[1] } else { '-' } }) -join ' | ')
}
function HlCropWin($name, $h, $pad = 8) {
  $r = Rect $h
  if ($r.R -le $r.L) { Log "no rect for $name"; return }
  Crop $name ($r.L - $pad) ($r.T - $pad) ($r.R - $r.L + 2 * $pad) ($r.B - $r.T + 2 * $pad) | Out-Null
}
# The rectangle around every open popup menu.
function HlCropMenus($name) {
  $ms = @([W]::Tops('#32768'))
  if (-not $ms.Count) { Log "no menu for $name"; return }
  $l = 99999; $t = 99999; $r = 0; $b = 0
  foreach ($m in $ms) { $q = Rect $m; if ($q.L -lt $l) { $l = $q.L }; if ($q.T -lt $t) { $t = $q.T }; if ($q.R -gt $r) { $r = $q.R }; if ($q.B -gt $b) { $b = $q.B } }
  Crop $name ($l - 4) ($t - 4) ($r - $l + 8) ($b - $t + 8) | Out-Null
}
function HlTitleMenu($title) {
  $r = (FenceGet $title).windowRect
  [G]::RClick([int]($r.x + $r.w - 60), [int]($r.y + 14))
  return (MenuWin 6)
}
# Opens every submenu of the open menu $m in turn: a crop and the texts of each.
function HlSubmenus($m, $surface, $lang) {
  trap { PfTrap $_; continue }
  $rows = @(MenuRows $m)
  $n = 0
  foreach ($row in $rows) {
    if (-not $row.text) { continue }
    [G]::MoveTo([int](($row.L + $row.R) / 2) - 10, [int](($row.T + $row.B) / 2)); Start-Sleep -Milliseconds 120
    [G]::MoveTo([int](($row.L + $row.R) / 2), [int](($row.T + $row.B) / 2)); Start-Sleep -Milliseconds 650
    $subs = @([W]::Tops('#32768')) | Where-Object { $_ -ne $m }
    if (@($subs).Count) {
      $n++
      Log "TEXT $surface-sub$n $lang`: [$($row.text)] $(HlMenuLine @($subs)[0])"
      HlCropMenus "$surface-sub$n-$lang"
    }
  }
}
function HlBox() { $h = [G]::FindWindowW('PecoFence.PeekSearch', [NullString]::Value); if ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) { return $h }; return [IntPtr]::Zero }
function HlEndPeek() { for ($i = 0; $i -lt 4 -and (HlBox) -ne [IntPtr]::Zero; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 400 } }

function HlLanguage($lang) {
  trap { PfTrap $_; continue }
  Case "captures $lang"
  Pf settings set language $lang | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse 1500 840
  Shot "desk-$lang"

  # A fence's menu and its submenus.
  $m = HlTitleMenu 'Solo'
  Check "$lang`: the fence menu opens" ($m -ne [IntPtr]::Zero) ''
  Log "TEXT fencemenu $lang`: $(HlMenuLine $m)"
  HlCropMenus "fencemenu-$lang"
  HlSubmenus $m 'fencemenu' $lang
  CloseMenus
  if ($Only -eq 'menu') { return }
  # The tab fence, the folder fence, the note, the clock, the script and the smart fence: texts only,
  # and one crop each.
  foreach ($k in @(@('Work', 'tabmenu'), @('Shared docs', 'foldermenu'), @('Memo', 'notemenu'), @('Tick', 'clockmenu'), @('Fail', 'scriptmenu'), @('Psd', 'smartmenu'))) {
    $m = HlTitleMenu $k[0]
    if ($m -eq [IntPtr]::Zero) { Log "no menu for $($k[0])"; continue }
    Log "TEXT $($k[1]) $lang`: $(HlMenuLine $m)"
    HlCropMenus "$($k[1])-$lang"
    if ($k[1] -eq 'tabmenu' -or $k[1] -eq 'smartmenu' -or $k[1] -eq 'scriptmenu') { HlSubmenus $m $k[1] $lang }
    CloseMenus
  }
  # An icon's menu (PecoFence's entries are at the top).
  $m = RClickItem $HlInbox $HlItem
  if ($m -ne [IntPtr]::Zero) {
    Start-Sleep -Milliseconds 500
    Log "TEXT itemmenu $lang`: $(HlMenuLine $m)"
    HlCropMenus "itemmenu-$lang"
    HlSubmenus $m 'itemmenu' $lang
  }
  CloseMenus
  DeskClick 1500 840 | Out-Null

  # The tray menu.
  $m = TrayMenu
  if ($m -ne [IntPtr]::Zero) {
    Start-Sleep -Milliseconds 400
    Log "TEXT traymenu $lang`: $(HlMenuLine $m)"
    HlCropMenus "traymenu-$lang"
    HlSubmenus $m 'traymenu' $lang
  }
  CloseMenus
  DeskClick 1500 840 | Out-Null

  # The search box: empty, with results, with nothing found.
  PeekKey; Start-Sleep -Milliseconds 900
  [G]::Type('a'); Start-Sleep -Milliseconds 500
  [G]::Press(0x08); Start-Sleep -Milliseconds 800
  $b = HlBox
  Check "$lang`: the search box opens" ($b -ne [IntPtr]::Zero) ''
  HlCropWin "search-empty-$lang" $b 4
  [G]::Type('hl'); Start-Sleep -Milliseconds 1200
  HlCropWin "search-rows-$lang" (HlBox) 4
  # The row's action list (the right arrow opens it).
  [G]::Press(0x27); Start-Sleep -Milliseconds 700
  HlCropWin "search-actions-$lang" (HlBox) 4
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  if ((HlBox) -eq [IntPtr]::Zero) { PeekKey; Start-Sleep -Milliseconds 900 }
  [G]::Type('qqzzxx'); Start-Sleep -Milliseconds 1200
  HlCropWin "search-none-$lang" (HlBox) 4
  HlEndPeek
  [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  DeskClick 1500 840 | Out-Null

  # The marquee's buttons.
  $fr = FreeRect 260 150 60 560
  if ($fr) {
    DragTo ($fr[0] + 10) ($fr[1] + 10) ($fr[2] - 10) ($fr[3] - 30) 12
    $o = [IntPtr]::Zero
    for ($i = 0; $i -lt 30; $i++) { $o = [G]::FindWindowW('PecoFence.MarqueeOffer', [NullString]::Value); if ($o -ne [IntPtr]::Zero -and [G]::IsWindowVisible($o)) { break }; Start-Sleep -Milliseconds 100 }
    Start-Sleep -Milliseconds 500
    Check "$lang`: the marquee offers its buttons" ($o -ne [IntPtr]::Zero -and [G]::IsWindowVisible($o)) ''
    Log "TEXT marquee $lang`: $([L]::Text($o))"
    $r = Rect $o
    Crop "marquee-$lang" ($r.L - 240) ($r.T - 40) ($r.R - $r.L + 260) ($r.B - $r.T + 60) | Out-Null
    # 新建栅栏 (the right button): the name box with its hint.
    [G]::Click($r.R - 30, [int](($r.T + $r.B) / 2)); Start-Sleep -Milliseconds 1200
    $nb = [G]::FindWindowW('PecoFence.Rename', [NullString]::Value)
    if ($nb -ne [IntPtr]::Zero -and [G]::IsWindowVisible($nb)) {
      $q = Rect $nb
      Crop "namebox-$lang" ($q.L - 60) ($q.T - 30) 520 110 | Out-Null
    } else { Log "no name box in $lang" }
    [G]::Press(0x1B); Start-Sleep -Milliseconds 700
    # The fence the box made stays without a name: delete it.
    foreach ($f in @(PfJson fence list)) { if ($f -and $f.id -and $HlKeep -notcontains $f.id) { Pf fence delete $f.id | Out-Null } }
    Start-Sleep -Milliseconds 500
  }

  # Tooltips: the lock mark of a locked fence, an icon.
  $w = (FenceGet 'Locked').windowRect
  [G]::MoveTo($w.x + $w.w - 90, $w.y + 14); Start-Sleep -Milliseconds 300
  [G]::MoveTo($w.x + $w.w - 44, $w.y + 14); Start-Sleep -Milliseconds 300
  [G]::MoveTo($w.x + $w.w - 43, $w.y + 15); Start-Sleep -Milliseconds 1800
  Crop "tip-lock-$lang" ($w.x - 100) ($w.y - 10) 760 110 | Out-Null
  $p = ItemPt $HlInbox $HlItem
  if ($p) {
    [G]::MoveTo($p[0] + 30, $p[1] + 30); Start-Sleep -Milliseconds 300
    [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 300
    [G]::MoveTo($p[0] + 1, $p[1] + 1); Start-Sleep -Milliseconds 1800
    Crop "tip-item-$lang" ($p[0] - 60) ($p[1] - 40) 460 170 | Out-Null
  }
  ParkMouse 1500 840
}

Case 'setup'
CloseWindows
GreyWallpaper
Add-Type -AssemblyName System.Windows.Forms
. C:\pf\kit\user.ps1
$HlFile = 'hl 季度报告与资料汇总.txt'
$HlItem = 'hl 季度报告与资料汇总'
$HlFiles = @($HlFile, 'hl Geschwindigkeitsbegrenzung.txt', 'hl 图.txt')
foreach ($n in $HlFiles) { DeskFile $n | Out-Null }
New-Item -ItemType Directory -Force "$PfRun\Shared docs" | Out-Null
Set-Content "$PfRun\empty.ps1" '# prints nothing'
Set-Content "$PfRun\fail.ps1" 'Write-Error "boom"; exit 3'

if ($Part -match 'C') {
  PfStart -Lang en
  Pf settings set theme light | Out-Null
  ClearFences | Out-Null
  $HlInbox = (@(PfJson fence list) | Where-Object { $_ -and $_.kind -eq 'inbox' } | Select-Object -First 1).id
  Pf fence create --title Solo --rect '300,60,330,200' | Out-Null
  Pf fence create --title Work --rect '300,290,330,190' | Out-Null
  Pf fence create --title 'Geschwindigkeitsbegrenzungen 季度报告' --rect '300,500,300,200' | Out-Null
  Pf fence merge 'Geschwindigkeitsbegrenzungen 季度报告' --into Work | Out-Null
  Pf fence create --portal "$PfRun\Shared docs" --title 'Shared docs' --rect '660,60,330,200' | Out-Null
  Pf fence create --tool note --title Memo --rect '660,290,300,170' | Out-Null
  Pf fence create --tool clock --title Tick --rect '1020,60,260,150' | Out-Null
  Pf fence create --script "$PfRun\fail.ps1" --title Fail --rect '1020,240,300,170' | Out-Null
  Pf fence create --script "$PfRun\empty.ps1" --title Quiet --rect '1340,240,240,170' | Out-Null
  Pf fence create --smart psd --title Psd --rect '1020,440,300,190' | Out-Null
  Pf fence create --title Locked --rect '660,490,330,150' | Out-Null
  Pf fence set Locked locked true | Out-Null
  Start-Sleep -Seconds 4
  $HlKeep = @(@(PfJson fence list) | Where-Object { $_ -and $_.id } | ForEach-Object { $_.id })
  Log "fences: $((@(PfJson fence list) | Where-Object { $_ -and $_.title } | ForEach-Object { "$($_.title) [$($_.kind)]" }) -join ', '); inbox '$HlInbox'"
  foreach ($lang in $LangList) { HlLanguage $lang }
  PfQuit
}

if ($Part -match 'D') {
  # The details view's column headings, a rolled fence's count, a filter's tag in the title, the
  # date group captions and the box that asks before a fence is deleted.
  PfStart -Lang en
  Pf settings set theme light | Out-Null
  ClearFences | Out-Null
  foreach ($n in 'hd alpha.txt', 'hd beta.txt', 'hd gamma.txt') { Set-Content "$PfRun\Shared docs\$n" 'x' }
  Pf fence create --title Det --rect '300,60,620,200' | Out-Null
  Pf fence create --title Rolled --rect '300,300,300,200' | Out-Null
  Pf fence create --title Grp --rect '960,60,330,260' | Out-Null
  Pf fence create --portal "$PfRun\Shared docs" --title 'Shared docs' --rect '660,300,420,220' | Out-Null
  Pf item move (Join-Path $PfDesk $HlFiles[0]) --to Det | Out-Null
  Pf item move (Join-Path $PfDesk $HlFiles[1]) --to Rolled | Out-Null
  Pf item move (Join-Path $PfDesk $HlFiles[2]) --to Grp | Out-Null
  Pf fence set Det layout details | Out-Null
  Pf fence set Grp sort date | Out-Null
  Pf fence set Grp groupByDate true | Out-Null
  Pf fence roll Rolled | Out-Null
  Start-Sleep -Seconds 3
  foreach ($lang in $LangList) {
    Case "more captures $lang"
    Pf settings set language $lang | Out-Null
    Start-Sleep -Seconds 2
    ParkMouse 1500 840
    $d = (FenceGet 'Det').windowRect
    Crop "details-$lang" ($d.x - 4) ($d.y - 4) ($d.w + 8) 110 | Out-Null
    $r = (FenceGet 'Rolled').windowRect
    Crop "rolled-$lang" ($r.x - 4) ($r.y - 4) ($r.w + 8) 48 | Out-Null
    $g = (FenceGet 'Grp').windowRect
    Crop "groups-$lang" ($g.x - 4) ($g.y - 4) ($g.w + 8) 120 | Out-Null
    # A filter in the folder fence: its tag in the title.
    $s = (FenceGet 'Shared docs').windowRect
    [G]::Click($s.x + 200, $s.y + $s.h - 30); Start-Sleep -Milliseconds 300
    Chord 0x11 0x46; Start-Sleep -Milliseconds 700
    Crop "filterbox-$lang" ($s.x - 4) ($s.y - 4) ($s.w + 8) 110 | Out-Null
    [G]::Type('hd'); Start-Sleep -Milliseconds 700
    [G]::Press(0x0D); Start-Sleep -Milliseconds 600
    ParkMouse 1500 840
    Crop "filtertag-$lang" ($s.x - 4) ($s.y - 4) ($s.w + 8) 110 | Out-Null
    [G]::Click($s.x + 200, $s.y + $s.h - 30); Start-Sleep -Milliseconds 300
    [G]::Press(0x1B); Start-Sleep -Milliseconds 300; [G]::Press(0x1B); Start-Sleep -Milliseconds 400
    # 删除栅栏 (the last entry but one of the menu) on a fence with an icon: the box that asks.
    $m = HlTitleMenu 'Det'
    $rows = @(MenuRows $m | Where-Object { $_.text })
    $del = $rows[$rows.Count - 2]
    [G]::Click([int](($del.L + $del.R) / 2), [int](($del.T + $del.B) / 2))
    $box = WaitWin '#32770' 5
    Start-Sleep -Milliseconds 500
    Check "$lang`: 删除栅栏 asks first" ($box -ne [IntPtr]::Zero) "picked '$($del.text)'"
    # Enter takes the box's default button, No (a Yes / No box has no Esc).
    if ($box -ne [IntPtr]::Zero) { HlCropWin "confirm-$lang" $box 4; [G]::Press(0x0D); Start-Sleep -Milliseconds 600 }
    CloseMenus
  }
  PfQuit
}

if ($Part -match 'R') {
  # A rolled fence with a long name and twelve icons: the count after the name is one or two
  # digits in Chinese and a word with a colon in German, Russian, Spanish and French.
  Case 'rolled fence: name and count'
  $HrFiles = @(1..12 | ForEach-Object { 'hr {0:d2}.txt' -f $_ })
  foreach ($n in $HrFiles) { DeskFile $n | Out-Null }
  PfStart -Lang zh-CN
  Pf settings set theme light | Out-Null
  ClearFences | Out-Null
  $long = 'Geschwindigkeitsbegrenzungen 季度报告与资料汇总'
  Pf fence create --title $long --rect '700,200,300,220' | Out-Null
  foreach ($n in $HrFiles) { Pf item move (Join-Path $PfDesk $n) --to $long | Out-Null }
  Pf fence roll $long | Out-Null
  Start-Sleep -Seconds 2
  foreach ($lang in @('zh-CN', 'en', 'ja', 'ko', 'de', 'fr', 'es', 'pt-BR', 'ru', 'it', 'zh-TW')) {
    Pf settings set language $lang | Out-Null
    Start-Sleep -Milliseconds 1500
    ParkMouse 1500 840
    $r = (FenceGet $long).windowRect
    Crop "rolledlong-$lang" ($r.x - 4) ($r.y - 4) ($r.w + 8) 44 | Out-Null
    if ($lang -eq 'es') {
      $t = OcrText $r.x $r.y $r.w 36 'r-rolled-es'
      Check 'Spanish, a rolled fence with a long name: the count stands clear of the name' ($t -match '\sElementos:\s*12\b') "the title row reads '$t'"
    }
  }

  # The filter box of a folder fence: the count at its right end shows its number.
  Case 'filter box: the count'
  foreach ($n in 'hd alpha.txt', 'hd beta.txt', 'hd gamma.txt') { Set-Content "$PfRun\Shared docs\$n" 'x' }
  Pf fence create --portal "$PfRun\Shared docs" --title 'Shared docs' --rect '700,300,420,220' | Out-Null
  Pf settings set language de | Out-Null
  Start-Sleep -Seconds 2
  $s = (FenceGet 'Shared docs').windowRect
  [G]::Click($s.x + 200, $s.y + $s.h - 30); Start-Sleep -Milliseconds 300
  Chord 0x11 0x46; Start-Sleep -Milliseconds 800
  ParkMouse 1500 840
  Crop 'r-filterbox-de' ($s.x - 4) ($s.y - 4) ($s.w + 8) 60 | Out-Null
  $t = OcrText $s.x $s.y $s.w 40 'r-filterbox-ocr'
  Check 'German, the filter box of a fence with three files: the count shows its number' ($t -match 'Elemente:\s*3') "the box reads '$t'"
  # A word that names a type: the type is offered at the right end with its key, Tab.
  [G]::Type('installers'); Start-Sleep -Milliseconds 900
  Crop 'r-filtertype-de' ($s.x - 4) ($s.y - 4) ($s.w + 8) 60 | Out-Null
  $t = OcrText $s.x $s.y $s.w 40 'r-filtertype-ocr'
  Check 'German, "installers" typed in the filter box: the offered type shows its key, Tab' ($t -match 'Installationsprogramme\s+Tab') "the box reads '$t'"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400

  # The search box when nothing matches: the words at the right of the typed text show whole.
  Case 'search box: nothing found'
  Pf settings set language fr | Out-Null
  Start-Sleep -Seconds 2
  DeskClick 1500 840 | Out-Null
  PeekKey; Start-Sleep -Milliseconds 900
  [G]::Type('qqzzxx'); Start-Sleep -Milliseconds 1200
  $b = HlBox
  $q = Rect $b
  Crop 'r-searchnone-fr' ($q.L - 4) ($q.T - 4) ($q.R - $q.L + 8) 64 | Out-Null
  $t = OcrText $q.L $q.T ($q.R - $q.L) 56 'r-searchnone-ocr'
  Check 'French, a search that finds nothing: the box says so in whole words' ($b -ne [IntPtr]::Zero -and $t -match 'Aucun fichier correspondant') "the box reads '$t'"
  HlEndPeek
  [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  PfQuit
  foreach ($n in $HrFiles) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
}

if ($Part -match 'E') {
  # The search box with Everything's rows: the second group heading, the folder line under a name
  # and the two action rows at the end, in every language.
  Case 'search box with Everything'
  . C:\pf\kit\smart.ps1
  $deep = "$PfRun\Shared docs"
  Set-Content -LiteralPath (Join-Path $deep 'hlev 季度总结.txt') -Value 'x' -Encoding UTF8
  Set-Content -LiteralPath (Join-Path $deep 'hlev Geschwindigkeitsbegrenzung.txt') -Value 'x' -Encoding UTF8
  $evDesk = Join-Path $PfDesk 'hlev 桌面上的.txt'
  Set-Content -LiteralPath $evDesk -Value 'x' -Encoding UTF8
  $ev = EvStart
  PfStart -Lang en
  Pf settings set theme light | Out-Null
  Pf settings set tools.everything true | Out-Null
  Start-Sleep -Seconds 4
  Check 'Everything runs' $ev ''
  foreach ($lang in $LangList) {
    Pf settings set language $lang | Out-Null
    Start-Sleep -Milliseconds 1500
    DeskClick 1500 840 | Out-Null
    PeekKey; Start-Sleep -Milliseconds 900
    [G]::Type('hlev'); Start-Sleep -Milliseconds 2500
    HlCropWin "search-everything-$lang" (HlBox) 4
    # The last row, 「把这次搜索放到桌面」, with the line under it.
    [G]::Press(0x26); Start-Sleep -Milliseconds 500
    HlCropWin "search-everything-last-$lang" (HlBox) 4
    HlEndPeek
    [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  }
  PfQuit
  Remove-Item -LiteralPath $evDesk -Force -ErrorAction SilentlyContinue
}

if ($Part -match 'L') {
  # A script fence's status line and a smart fence's empty text are kept by the window as text:
  # a language switch has to give them again.
  Case 'kept texts follow a language switch'
  PfStart -Lang zh-CN
  Pf settings set theme light | Out-Null
  ClearFences | Out-Null
  Pf fence create --script "$PfRun\fail.ps1" --title Fail --rect '700,120,420,200' | Out-Null
  Pf fence create --smart psd --title Psd --rect '700,360,420,220' | Out-Null
  Start-Sleep -Seconds 8
  ParkMouse 1500 840
  $f = (FenceGet 'Fail').windowRect
  $s = (FenceGet 'Psd').windowRect
  Pf settings set language en | Out-Null
  Start-Sleep -Seconds 2
  Crop 'l-after-switch' 690 110 440 480 | Out-Null
  $foot = OcrText $f.x ($f.y + $f.h - 44) $f.w 44 'l-footer-en'
  $empty = OcrText $s.x ($s.y + 40) $s.w ($s.h - 40) 'l-empty-en'
  Check "2 s after a switch from Chinese to English the script fence's status line is English" ($foot -match 'exit code') "the line reads '$foot'"
  Check "2 s after a switch from Chinese to English the smart fence's empty text is English" ($empty -match 'No files match|needs Everything') "the text reads '$empty'"

  # A status line wider than the fence (German, a fence 300 px wide): its end, the exit code, shows.
  Case 'a long status line shows whole'
  Pf fence create --script "$PfRun\fail.ps1" --title Narrow --rect '1140,120,300,200' | Out-Null
  Pf settings set language de | Out-Null
  Start-Sleep -Seconds 2
  Pf fence refresh Narrow | Out-Null
  Start-Sleep -Seconds 5
  ParkMouse 1500 840
  $n = (FenceGet 'Narrow').windowRect
  Crop 'l-narrow-de' ($n.x - 4) ($n.y - 4) ($n.w + 8) ($n.h + 8) | Out-Null
  $foot = OcrText $n.x ($n.y + $n.h - 60) $n.w 60 'l-narrow-footer'
  Check 'German, a script fence 300 px wide: the status line shows the exit code' ($foot -match 'Exitcode\s*3') "the line reads '$foot'"
  PfQuit
}

Case 'cleanup'
foreach ($n in $HlFiles) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
Check 'the test files are gone from the desktop' (-not (Test-Path -LiteralPath (Join-Path $PfDesk $HlFile))) ''
