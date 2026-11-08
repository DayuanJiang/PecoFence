# Fence menu grouping (添加到桌面 submenu) and the three 功能提示 cards, with real mouse and keys.
#   run.sh --var Part=M tests/fx-menu-tips.ps1    fence menus: every kind of fence, Fluent and
#                                                 Liquid Glass, English and Chinese, keyboard path
#   run.sh --var Part=T ...                        the three tips in English (and once only, also
#                                                 after a restart); Part=TZ the same in Chinese
#   run.sh --var Part=S ...                        no tip under a test script or with the switch off
#   run.sh --bin target/wintest-bin-debug --var Part=P ...   the 功能提示 switch on the settings
#                                                 page (CDP: a debug build)
# A 0.3.1 build for the "before" runs: add --var Old=1 (skips the steps that would act on items
# the old menu does not have).

if (-not $Part) { $Part = 'M' }
$IsOld = [bool]$Old

# ---- helpers of this test

# A menu line without the Ctrl+V hint and the grey mark of 粘贴 (it depends on the clipboard).
function PlainLine($line) { return ($line -replace ' ~ Ctrl\+V', '' -replace ' \(off\)', '') }
# Menu entries in order, separators as '-'.
function MenuLine($h) {
  return ((@(MenuRows $h) | ForEach-Object { if (-not $_.text) { '-' } elseif ($_.state -band 3) { "$($_.text) (off)" } else { $_.text } }) -join ' | ')
}
# The popup menu whose entries match $regex (a submenu opens as a second #32768 window).
function MenuWith($regex, $sec = 4) {
  for ($i = 0; $i -lt $sec * 10; $i++) {
    foreach ($m in @([W]::Tops('#32768'))) { if ((MenuLine $m) -match $regex) { return $m } }
    Start-Sleep -Milliseconds 100
  }
  return [IntPtr]::Zero
}
# Right-click on a fence's title row (by title, through `fence get`).
function TitleMenu($title) {
  $f = FenceGet $title
  $r = $f.windowRect
  [G]::RClick([int]($r.x + 50), [int]($r.y + 14))
  return (MenuWin 6)
}
function HoverRow($h, $regex) {
  $p = MenuPoint $h $regex
  if (-not $p) { Log "HoverRow '$regex' not in: $(MenuLine $h)"; return $false }
  [G]::MoveTo($p[0] - 20, $p[1]); Start-Sleep -Milliseconds 150; [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 900
  return $true
}
function NoticeWin() {
  $n = [G]::FindWindowW('PecoFence.Notice', [NullString]::Value)
  if ($n -ne [IntPtr]::Zero -and [G]::IsWindowVisible($n)) { return $n }
  return [IntPtr]::Zero
}
function WaitNotice($sec = 6) {
  for ($i = 0; $i -lt $sec * 5; $i++) { $n = NoticeWin; if ($n -ne [IntPtr]::Zero) { Start-Sleep -Milliseconds 400; return $n }; Start-Sleep -Milliseconds 200 }
  return [IntPtr]::Zero
}
function NoticeTitle() { $n = NoticeWin; if ($n -eq [IntPtr]::Zero) { return '' }; return [L]::Text($n) }
# Card buttons at 100 %: 16 px padding, 32 px tall, the accent button at least 120 px wide on the
# right, the plain one 8 px left of it.
function ClickAccent() { $r = Rect (NoticeWin); [G]::Click($r.R - 46, $r.B - 32); Start-Sleep -Milliseconds 900 }
function ClickPlain() { $r = Rect (NoticeWin); [G]::Click($r.R - 170, $r.B - 32); Start-Sleep -Milliseconds 900 }
function ShotNotice($name) { $r = Rect (NoticeWin); if ($r.R -gt $r.L) { Crop $name ($r.L - 12) ($r.T - 12) ($r.R - $r.L + 24) ($r.B - $r.T + 24) | Out-Null } }
# Starts the build like PfStart, without --test-script: notices and 功能提示 may show.
function PfStartPlain() {
  if (Get-Process pecofence -ErrorAction SilentlyContinue) { PfQuit }
  $cur = PfLogPath
  if (Test-Path $cur) {
    New-Item -ItemType Directory -Force "$PfRun\logs" | Out-Null
    $n = @(Get-ChildItem "$PfRun\logs\app-*.log" -ErrorAction SilentlyContinue).Count + 1
    Copy-Item $cur "$PfRun\logs\app-$n.log"
  }
  $env:PECOFENCE_INSTANCE = $PfInst
  $env:RUST_LOG = 'pecofence=debug'
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $PfApp
  $h = WaitWin 'PecoFence.Control' 20 -Hidden
  Log "started plain $PfInst control=$h"
  Start-Sleep -Seconds 4
  # A start after a quit while a fence was in front does not see desktop presses until a window
  # opens and closes (finding S2, fixed in another branch): open and close Notepad.
  Start-Process notepad; Start-Sleep -Seconds 2
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Seconds 1
}
function NoticesJson() {
  $cfg = (PfJson paths).config
  $p = Join-Path (Split-Path $cfg) 'notices.json'
  if (Test-Path $p) { return (Get-Content $p -Raw) -replace '\s+', ' ' }
  return '(none)'
}
# A box on the empty left part of the desktop, then 「在此新建栅栏」 and Esc for the title box.
# Right of the 「桌面」 fence's cells (arranged like the desktop at the top left). One retry when
# the box offered nothing.
# What the box offers: the 「在此新建栅栏」 menu (0.3.1 and this branch) or, since the marquee
# decision (U10 B), two small buttons in PecoFence.MarqueeOffer with 「＋ 新建栅栏」 on the right.
function MarqueeOfferWin($sec = 4) {
  for ($i = 0; $i -lt $sec * 10; $i++) {
    $m = @([W]::Tops('#32768')); if ($m.Count) { return @('menu', $m[0]) }
    $o = [G]::FindWindowW('PecoFence.MarqueeOffer', [NullString]::Value)
    if ($o -ne [IntPtr]::Zero -and [G]::IsWindowVisible($o)) { return @('buttons', $o) }
    Start-Sleep -Milliseconds 100
  }
  return @('', [IntPtr]::Zero)
}
function MarqueeFence($y, $x = 110) {
  DragTo $x $y ($x + 170) ($y + 130) 12
  $o = MarqueeOfferWin
  if ($o[1] -eq [IntPtr]::Zero) {
    Log "marquee at $x,$y offered nothing; once more"
    [G]::Click($x + 100, $y + 60); Start-Sleep -Milliseconds 800
    DragTo $x $y ($x + 170) ($y + 130) 12
    $o = MarqueeOfferWin
  }
  $picked = ''
  if ($o[0] -eq 'menu') { $picked = MenuPick 'Create a fence here|在此新建栅栏' $o[1] }
  if ($o[0] -eq 'buttons') {
    $r = Rect $o[1]; $bx = $r.R - 30; $by = [int](($r.T + $r.B) / 2)
    Glide ($bx - 50) ($by + 30) $bx $by 8 25; Start-Sleep -Milliseconds 200
    [G]::Click($bx, $by); $picked = 'New fence button'
    Log "marquee buttons at $($r.L),$($r.T),$($r.R),$($r.B); clicked $bx,$by"
  }
  Start-Sleep -Milliseconds 900
  [G]::Press(0x1B); Start-Sleep -Milliseconds 700
  return $picked
}
# 添加规则 may be followed by a card saying where rules are edited (the item menu's rule offer
# shares it since the merge): read its title and close it with its x.
function CloseRuleAddedCard() {
  $n = WaitNotice 3
  if ($n -eq [IntPtr]::Zero) { return '' }
  $t = NoticeTitle
  if ($t -notmatch 'Sorting rule added|已添加整理规则') { return $t }
  $r = Rect $n
  [G]::Click($r.R - 24, $r.T + 24); Start-Sleep -Milliseconds 300
  [G]::MoveTo($Park[0], $Park[1])
  WaitGoneWin 'PecoFence.Notice' 3 | Out-Null
  Log "closed the card '$t'"
  return $t
}
# ItemPt by the name without its extension (the fence may show either).
function ItemAt($fence, $name) {
  $full = @((PfJson item list --fence $fence) | Where-Object { $_ -and ($_.name -eq $name -or $_.name -like "$name.*") } | Select-Object -First 1).name
  if (-not $full) { Log "ItemAt: '$name' not in '$fence'"; return $null }
  return (ItemPt $fence $full)
}
# 添加到桌面 › 新建栅栏 in the fence menu of $title, then Esc for the title box.
function MenuNewFence($title) {
  $m = TitleMenu $title
  HoverRow $m 'Add to desktop|添加到桌面' | Out-Null
  $sub = MenuWith 'New fence|新建栅栏'
  $picked = MenuPick '^(New fence|新建栅栏)$' $sub
  Start-Sleep -Seconds 1
  [G]::Press(0x1B); Start-Sleep -Milliseconds 700
  return $picked
}
function DragItemTo($from, $name, $to) {
  $p = ItemAt $from $name
  $t = (FenceGet $to).windowRect
  if (-not $p) { return $false }
  DragTo $p[0] $p[1] ([int]($t.x + $t.w / 2)) ([int]($t.y + $t.h - 40)) 15
  Start-Sleep -Milliseconds 1200
  return $true
}
function OpenFolderItem($fence, $name) {
  $p = ItemAt $fence $name
  if (-not $p) { return $false }
  DoubleClick $p[0] $p[1]
  Start-Sleep -Seconds 3
  CloseWindows
  Start-Sleep -Milliseconds 600
  return $true
}
function FenceCount() { return @(@(PfJson fence list) | Where-Object { $_ -and $_.title }).Count }

$Park = @(700, 820)
function ParkPointer() { [G]::MoveTo($Park[0], $Park[1]); Start-Sleep -Milliseconds 300 }

# ---- Part M: the fence menus

if ($Part -eq 'M') {
Case 'setup menus'
CloseWindows
Set-Culture en-US
GreyWallpaper
New-Item -ItemType Directory -Force "$PfRun\Shared docs" | Out-Null
Set-Content "$PfRun\Shared docs\plan.txt" 'plan'
PfStart
Pf settings set theme light | Out-Null
Pf fence create --title Work --rect '300,150,520,300' | Out-Null
Pf fence create --title Games --rect '300,500,300,200' | Out-Null
Pf fence merge Games --into Work | Out-Null
Pf fence create --title Solo --rect '860,150,300,200' | Out-Null
Pf fence create --portal "$PfRun\Shared docs" --title 'Shared docs' --rect '860,420,300,200' | Out-Null
Pf fence create --tool note --title Today --text 'note' | Out-Null
Pf fence create --tool clock | Out-Null
Start-Sleep -Seconds 2
ParkPointer
Check 'fences made' ((FenceCount) -ge 8) "fences=$(FenceCount)"

foreach ($combo in @(@('fluent', 'en'), @('liquidGlass', 'en'), @('fluent', 'zh-CN'), @('liquidGlass', 'zh-CN'))) {
  $style = $combo[0]; $lang = $combo[1]
  Pf settings set themeStyle $style | Out-Null
  Pf settings set language $lang | Out-Null
  Start-Sleep -Seconds 2
  $title = 'Solo'
  $m = TitleMenu $title
  $line = MenuLine $m
  Log "menu $style $lang : $line"
  Check "$style $lang fence menu opens" ($m -ne [IntPtr]::Zero) $line
  if ($lang -eq 'en') {
    $want = '^Roll up \| Rename \| - \| View \| Sort by \| New \| Paste \| Tabs \| - \| (Fit to content \| )?Lock position and size \| Fence options… \| Delete fence \| - \| Add to desktop$'
  } else {
    $want = '^收起 \| 重命名… \| - \| 视图 \| 排序方式 \| 新建 \| 粘贴 \| 标签页 \| - \| (适应内容大小 \| )?锁定位置和大小 \| 栅栏选项… \| 删除栅栏 \| - \| 添加到桌面$'
  }
  Check "$style $lang fence menu in the new order" ((PlainLine $line) -match $want) $line
  Shot "m-$style-$lang-menu"
  $ok = HoverRow $m 'Add to desktop|添加到桌面'
  $sub = MenuWith 'New fence|新建栅栏'
  $subLine = if ($sub -ne [IntPtr]::Zero) { MenuLine $sub } else { '' }
  Log "submenu $style $lang : $subLine"
  if ($lang -eq 'en') { $wantSub = 'New fence | New note | New tool | Show a folder on the desktop' } else { $wantSub = '新建栅栏 | 新建便签 | 新建小工具 | 在桌面显示文件夹' }
  Check "$style $lang Add to desktop holds the four global entries" ($subLine -eq $wantSub) $subLine
  Shot "m-$style-$lang-submenu"
  CloseMenus
  ParkPointer
}
Pf settings set themeStyle fluent | Out-Null
Pf settings set language en | Out-Null
Start-Sleep -Seconds 1

Case 'every kind of fence'
$kinds = @(
  @('Work', 'tab fence', 'Roll up \| Rename \| - \| View \| Sort by \| New \| Paste \| Tabs \| - \| (Fit to content \| )?Lock position and size \| Fence options… \| Delete fence \| - \| Add to desktop$'),
  @('Shared docs', 'folder fence', '^Roll up \| Rename \| Open folder in File Explorer .*\| - \| View .*\| - \| (Fit to content \| )?Lock position and size \| Fence options… \| Delete fence \| - \| Add to desktop$'),
  @('Today', 'note', '^Roll up \| Rename \| - \| Edit note \| - \| Lock position and size \| Fence options… \| Delete fence \| - \| Add to desktop$'),
  @('Clock', 'clock', '^Roll up \| Rename \| - \| Show seconds \| - \| Lock position and size \| Fence options… \| Delete fence \| - \| Add to desktop$')
)
foreach ($k in $kinds) {
  $f = FenceGet $k[0]
  if (-not $f -or -not $f.windowRect) { Check "$($k[1]) found" $false "title=$($k[0])"; continue }
  $m = TitleMenu $k[0]
  $line = MenuLine $m
  Log "menu $($k[1]): $line"
  Check "$($k[1]) menu groups" ((PlainLine $line) -match $k[2]) $line
  Shot "k-$($k[1])"
  CloseMenus
  ParkPointer
}
# The 「桌面」 fence: its own menu needs 像桌面一样排列 off (arranged, the window is only icons).
Pf fence set Desktop desktopLayout false | Out-Null
Pf fence move Desktop --rect '40,500,220,200' | Out-Null
Start-Sleep -Seconds 1
$m = TitleMenu 'Desktop'
$line = MenuLine $m
Log "menu inbox: $line"
Check 'Desktop fence menu: hide when empty and arrange like the desktop in place of delete' ((PlainLine $line) -match '^Roll up \| Rename \| - \| View .*\| - \| (Fit to content \| )?Lock position and size \| Fence options… \| Hide when empty \| Arrange like the desktop( \| Place freely)? \| - \| Add to desktop$') $line
Shot 'k-inbox'
CloseMenus
ParkPointer

Case 'entries of Add to desktop work'
$before = FenceCount
$m = TitleMenu 'Solo'
HoverRow $m 'Add to desktop' | Out-Null
$sub = MenuWith 'New fence'
$picked = MenuPick '^New fence$' $sub
Start-Sleep -Seconds 1
[G]::Press(0x1B); Start-Sleep -Milliseconds 600
$after = FenceCount
Check 'Add to desktop > New fence makes a fence' ($after -eq $before + 1) "picked=$picked fences $before -> $after"
$m = TitleMenu 'Solo'
HoverRow $m 'Add to desktop' | Out-Null
$sub = MenuWith 'New note'
$picked = MenuPick '^New note$' $sub
$ed = WaitWin 'PecoFence.NoteEdit' 6
Check 'Add to desktop > New note opens a note editor' ($ed -ne [IntPtr]::Zero) "picked=$picked editor=$ed"
[G]::Press(0x1B); Start-Sleep -Milliseconds 800
$m = TitleMenu 'Solo'
HoverRow $m 'Add to desktop' | Out-Null
$sub = MenuWith 'Show a folder on the desktop'
HoverRow $sub 'Show a folder on the desktop' | Out-Null
$folders = MenuWith 'Choose another folder'
Check 'Add to desktop > Show a folder on the desktop lists the folders' ($folders -ne [IntPtr]::Zero) (MenuLine $folders)
Shot 'k-show-folder'
CloseMenus
$sub = $null
$m = TitleMenu 'Solo'
HoverRow $m 'Add to desktop' | Out-Null
$sub = MenuWith 'New tool'
HoverRow $sub 'New tool' | Out-Null
$tools = MenuWith 'Your own script'
Check 'Add to desktop > New tool lists the tools' ($tools -ne [IntPtr]::Zero) (MenuLine $tools)
CloseMenus
ParkPointer

Case 'keyboard: the menu key reaches Add to desktop'
# The CLI gets no answer while a menu is open: count first.
$before = FenceCount
$f = FenceGet 'Solo'
$r = $f.windowRect
[G]::Click([int]($r.x + $r.w / 2), [int]($r.y + $r.h - 30)); Start-Sleep -Milliseconds 500
[G]::Press(0x5D); Start-Sleep -Milliseconds 900
$m = MenuWin 4
Check 'menu key opens the fence menu' ($m -ne [IntPtr]::Zero) (MenuLine $m)
[G]::Press(0x26); Start-Sleep -Milliseconds 300
$last = @(MenuRows $m | Where-Object { $_.text } | Select-Object -Last 1).text
Check 'Up from the top lands on the last entry' ($last -ne $null) "last=$last"
if (-not $IsOld) {
  [G]::Press(0x27); Start-Sleep -Milliseconds 700
  $sub = MenuWith 'New fence' 2
  Check 'Right opens Add to desktop' ($sub -ne [IntPtr]::Zero) (MenuLine $sub)
  [G]::Press(0x0D); Start-Sleep -Seconds 1
  [G]::Press(0x1B); Start-Sleep -Milliseconds 600
  Check 'Enter on its first entry makes a fence' ((FenceCount) -eq $before + 1) "fences $before -> $(FenceCount)"
}
CloseMenus
[G]::Press(0x1B)

Case 'cleanup menus'
PfQuit
}

# ---- Parts T / TZ: the three tips, once each

if ($Part -eq 'T' -or $Part -eq 'TZ') {
$Zh = $Part -eq 'TZ'
$Lng = if ($Zh) { 'zh-CN' } else { 'en' }
if ($Zh) {
  $TRoll = '栅栏多了可以收起'; $TFolder = '把常用的文件夹放到桌面上'; $TRule = '让新文件自动进来'
} else {
  $TRoll = 'Roll up fences when there are many'; $TFolder = 'Put a folder you use often on the desktop'; $TRule = 'Let new files come in by themselves'
}

Case "setup tips $Lng"
CloseWindows
Set-Culture en-US
GreyWallpaper
$made = @()
foreach ($n in 'doc1.pdf', 'doc2.pdf', 'doc3.pdf', 'doc4.pdf') { $made += (DeskFile $n) }
New-Item -ItemType Directory -Force "$PfDesk\Projects" | Out-Null
Set-Content "$PfDesk\Projects\plan.txt" 'plan'
$made += "$PfDesk\Projects"
PfStart -Lang $Lng
if ($Zh) { Pf settings set themeStyle liquidGlass | Out-Null }
# The first start's 文件与文档 fence: named in the box's language (English), or in $Lng where the
# untouched first-start names follow the language.
Start-Sleep -Seconds 1
$FFiles = (@(PfJson fence list) | Where-Object { $_ -and $_.title -in @('Files and documents', '文件与文档') } | Select-Object -First 1).title
Log "the documents fence is '$FFiles'"
FenceIds
Log "fences at start: $($Id.Keys -join ', ')"
# Two fences besides 桌面 (文件与文档 and Work): the third the user makes gives no tip yet.
foreach ($t in @($Id.Keys)) {
  $g = FenceGet $t
  if ($t -ne $FFiles -and $g.kind -ne 'inbox') { Pf fence delete $t | Out-Null }
}
Pf fence create --title Work --rect '300,150,520,300' | Out-Null
Pf fence move $FFiles --rect '880,150,400,420' | Out-Null
Pf fence set --all sort name | Out-Null
Pf fence set --all foldersFirst false | Out-Null
Pf item move "$PfDesk\Projects" --to Work | Out-Null
foreach ($n in 'doc1.pdf', 'doc2.pdf', 'doc3.pdf', 'doc4.pdf') { Pf item move "$PfDesk\$n" --to $FFiles | Out-Null }
Start-Sleep -Seconds 1
Log "items: $((PfJson item list) | ForEach-Object { "$($_.name)@$($_.fenceTitle)" })"
PfStartPlain
ParkPointer
Start-Sleep -Seconds 3
Check 'no card at a normal start' ((NoticeWin) -eq [IntPtr]::Zero) "title=$(NoticeTitle)"

Case "roll tip $Lng"
$p1 = MarqueeFence 200
Start-Sleep -Seconds 1
Check 'third fence of the user: no tip yet' ((NoticeWin) -eq [IntPtr]::Zero) "picked=$p1 fences=$(FenceCount) title=$(NoticeTitle)"
$p2 = MarqueeFence 420
$n = WaitNotice 5
Check 'fourth fence: the roll-up tip' ((NoticeTitle) -eq $TRoll) "picked=$p2 title=$(NoticeTitle)"
ShotNotice "t-roll-$Lng"

Case "rule tip waits while another card shows $Lng"
foreach ($n in 'doc1', 'doc2', 'doc3') { DragItemTo $FFiles $n 'Work' | Out-Null }
Check 'three pdfs moved into Work' (@((PfJson item list --fence Work) | Where-Object { $_.name -like 'doc*' }).Count -eq 3) ((PfJson item list --fence Work | ForEach-Object { $_.name }) -join ', ')
Check 'rule tip held back while the roll-up tip shows' ((NoticeTitle) -eq $TRoll) "title=$(NoticeTitle)"
ClickAccent
Check 'Got it closes the roll-up tip' (WaitGoneWin 'PecoFence.Notice' 3) "title=$(NoticeTitle)"
DragItemTo $FFiles 'doc4' 'Work' | Out-Null
$n = WaitNotice 5
Check 'fourth pdf: the rule tip' ((NoticeTitle) -eq $TRule) "title=$(NoticeTitle)"
ShotNotice "t-rule-$Lng"
$rulesBefore = @(Rules).Count
if ($Zh) {
  # The plain button: closed, no rule.
  ClickPlain
  Check 'No thanks closes the rule tip' (WaitGoneWin 'PecoFence.Notice' 3) "title=$(NoticeTitle)"
  Check 'No thanks adds no rule' (@(Rules).Count -eq $rulesBefore) "count $rulesBefore -> $(@(Rules).Count)"
}
if (-not $Zh) {
  ClickAccent
  $added = CloseRuleAddedCard
  Log "after Add rule: '$added'"
  Start-Sleep -Seconds 1
  $first = @(Rules)[0]
  Check 'Add rule puts a .pdf rule into Work first in the list' (@(Rules).Count -eq $rulesBefore + 1 -and $first.targetTitle -eq 'Work') "name=$($first.name) target=$($first.targetTitle) count $rulesBefore -> $(@(Rules).Count)"
  $made += (DeskFile 'later.pdf')
  Check 'a new pdf goes into Work by the rule' ((WaitItemFence 'later.pdf' 'Work' 10) -eq 'Work') "fence=$(ItemFence 'later.pdf')"
}

Case "folder tip $Lng"
OpenFolderItem 'Work' 'Projects' | Out-Null
OpenFolderItem 'Work' 'Projects' | Out-Null
Check 'two opens: no folder tip' ((NoticeWin) -eq [IntPtr]::Zero) "title=$(NoticeTitle)"
OpenFolderItem 'Work' 'Projects' | Out-Null
$n = WaitNotice 5
Check 'third open: the folder tip' ((NoticeTitle) -eq $TFolder) "title=$(NoticeTitle)"
ShotNotice "t-folder-$Lng"
$before = FenceCount
ClickAccent
Start-Sleep -Seconds 2
$portal = @(PfJson fence list) | Where-Object { $_ -and $_.kind -eq 'portal' } | Select-Object -First 1
Check 'Try it shows Projects as a folder fence' ($portal -and (FenceCount) -eq $before + 1) "fences $before -> $(FenceCount) portal=$($portal.title)"
Shot "t-folder-fence-$Lng"
Log "notices.json: $(NoticesJson)"
Check 'notices.json records the three tips' ((NoticesJson) -match '"rollTipShown": true' -and (NoticesJson) -match '"folderTipShown": true' -and (NoticesJson) -match '"ruleTipShown": true') (NoticesJson)

Case "after a restart no tip comes again $Lng"
PfStartPlain
ParkPointer
Start-Sleep -Seconds 2
Check 'no card at the restart' ((NoticeWin) -eq [IntPtr]::Zero) "title=$(NoticeTitle)"
# By the fence menu: right after a restart the desktop box is not seen (finding S2).
$f0 = FenceCount
MenuNewFence 'Work' | Out-Null
Start-Sleep -Seconds 2
Check 'another new fence: no roll-up tip' ((FenceCount) -eq $f0 + 1 -and (NoticeWin) -eq [IntPtr]::Zero) "fences $f0 -> $(FenceCount) title=$(NoticeTitle)"
$portalTitle = (@(PfJson fence list) | Where-Object { $_ -and $_.kind -eq 'portal' } | Select-Object -First 1).title
if ($portalTitle) { Pf fence delete $portalTitle | Out-Null }
OpenFolderItem 'Work' 'Projects' | Out-Null
Check 'fourth open: no folder tip' ((NoticeWin) -eq [IntPtr]::Zero) "title=$(NoticeTitle)"
foreach ($n in 't1.txt', 't2.txt', 't3.txt') { $made += (DeskFile $n); Start-Sleep -Milliseconds 300 }
Start-Sleep -Seconds 2
foreach ($n in 't1.txt', 't2.txt', 't3.txt') { Pf item move "$PfDesk\$n" --to $FFiles | Out-Null }
Start-Sleep -Seconds 1
foreach ($n in 't1', 't2', 't3') { DragItemTo $FFiles $n 'Work' | Out-Null }
Start-Sleep -Seconds 1
Check 'three txt files moved into Work' (@((PfJson item list --fence Work) | Where-Object { $_.name -like 't?' -or $_.name -like 't?.txt' }).Count -eq 3) ((PfJson item list --fence Work | ForEach-Object { $_.name }) -join ', ')
Check 'three more files of one type: no rule tip' ((NoticeWin) -eq [IntPtr]::Zero) "title=$(NoticeTitle)"
$due = @(PfLogAll | Where-Object { $_ -match 'feature tip due|tip:|roll tip shown' })
Log ("tip log lines:`n" + ($due -join "`n"))

Case "cleanup tips $Lng"
PfQuit
foreach ($f in $made) { Remove-Item -LiteralPath $f -Recurse -Force -ErrorAction SilentlyContinue }
}

# ---- Part S: no tip under a test script or with the switch off

if ($Part -eq 'S') {
Case 'setup switch'
CloseWindows
Set-Culture en-US
GreyWallpaper
$made = @()
foreach ($n in 'doc1.pdf', 'doc2.pdf', 'doc3.pdf', 'doc4.pdf') { $made += (DeskFile $n) }
New-Item -ItemType Directory -Force "$PfDesk\Projects" | Out-Null
$made += "$PfDesk\Projects"
PfStart
$FFiles = 'Files and documents'
Pf fence create --title Work --rect '300,150,520,300' | Out-Null
Pf fence move $FFiles --rect '880,150,400,420' | Out-Null
Pf fence set --all sort name | Out-Null
Pf fence set --all foldersFirst false | Out-Null
Pf item move "$PfDesk\Projects" --to Work | Out-Null
foreach ($n in 'doc1.pdf', 'doc2.pdf', 'doc3.pdf', 'doc4.pdf') { Pf item move "$PfDesk\$n" --to $FFiles | Out-Null }
Start-Sleep -Seconds 1
ParkPointer

Shot 's-setup'

Case 'under a test script'
$f0 = FenceCount
MarqueeFence 600 880 | Out-Null
MarqueeFence 600 1080 | Out-Null
foreach ($n in 'doc1', 'doc2', 'doc3') { DragItemTo $FFiles $n 'Work' | Out-Null }
foreach ($i in 1..3) { OpenFolderItem 'Work' 'Projects' | Out-Null }
Check 'test script run: two new fences' ((FenceCount) -eq $f0 + 2) "fences $f0 -> $(FenceCount)"
Check 'test script run: no tip after fences, drags and opens' ((NoticeWin) -eq [IntPtr]::Zero) "title=$(NoticeTitle) fences=$(FenceCount)"
Check 'test script run: notices.json has no tip' ((NoticesJson) -notmatch 'TipShown": true') (NoticesJson)

Case 'switch off'
Pf settings set featureTips false | Out-Null
Check 'featureTips reads false' ((Pf settings get featureTips) -match 'false') (Pf settings get featureTips)
PfStartPlain
ParkPointer
$f0 = FenceCount
MarqueeFence 480 300 | Out-Null
Check 'switch off: the box made a fence' ((FenceCount) -eq $f0 + 1) "fences $f0 -> $(FenceCount)"
DragItemTo $FFiles 'doc4' 'Work' | Out-Null
OpenFolderItem 'Work' 'Projects' | Out-Null
Start-Sleep -Seconds 1
Check 'switch off: no tip after a fence, a drag and an open' ((NoticeWin) -eq [IntPtr]::Zero) "title=$(NoticeTitle) fences=$(FenceCount)"
Pf settings set featureTips true | Out-Null
MarqueeFence 480 520 | Out-Null
$n = WaitNotice 5
Check 'switch back on: the next new fence shows the roll-up tip' ((NoticeTitle) -eq 'Roll up fences when there are many') "title=$(NoticeTitle)"
ShotNotice 's-roll-on'

Case 'cleanup switch'
PfQuit
foreach ($f in $made) { Remove-Item -LiteralPath $f -Recurse -Force -ErrorAction SilentlyContinue }
}

# ---- Part P: the 功能提示 switch on the settings page (debug build, CDP)

if ($Part -eq 'P') {
Case 'settings page switch'
CloseWindows
Set-Culture en-US
if ($PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
PfStart -Lang en
$ok = PfSettings
Check 'settings open over CDP' $ok "debug=$PfDebugBuild"
$card = Js "(() => { const t = document.querySelector('[data-bind=featureTips]'); if (!t) return null; const c = t.closest('.card'); let h = c; while (h && h.tagName !== 'H3') h = h.previousElementSibling; return { title: c.querySelector('.t').textContent, help: c.querySelector('.s').textContent, on: t.getAttribute('aria-checked'), page: t.closest('section').id, section: h ? h.textContent : '' }; })()"
Log "card: $($card | ConvertTo-Json -Compress)"
Check 'General page has Feature tips, on' ($card -and $card.title -eq 'Feature tips' -and $card.on -eq 'true' -and $card.page -eq 'page-general') ($card | ConvertTo-Json -Compress)
PageRect '[data-bind=featureTips]' | Out-Null
Start-Sleep -Milliseconds 500
Shot 'p-switch-on-en'
PageClick '[data-bind=featureTips]'
Start-Sleep -Seconds 1
Check 'a click on the switch turns featureTips off' ((Pf settings get featureTips) -match 'false') (Pf settings get featureTips)
Check 'the switch shows off' ((Js "document.querySelector('[data-bind=featureTips]').getAttribute('aria-checked')") -eq 'false')
PageClick '[data-bind=featureTips]'
Start-Sleep -Seconds 1
Check 'a second click turns it back on' ((Pf settings get featureTips) -match 'true') (Pf settings get featureTips)
Pf settings set featureTips false | Out-Null
Start-Sleep -Seconds 1
Check 'the CLI turns the page switch off' ((Js "document.querySelector('[data-bind=featureTips]').getAttribute('aria-checked')") -eq 'false')
Pf settings set featureTips true | Out-Null
foreach ($q in 'feature tips', 'hints') {
  PageType '#settingsSearch' $q
  Start-Sleep -Milliseconds 800
  $hits = Js "document.getElementById('settingsSearchList').textContent"
  Check "Find a setting: '$q' finds Feature tips" ($hits -match 'Feature tips') $hits
}
Shot 'p-search-en'
PageType '#settingsSearch' ''
Pf settings set language zh-CN | Out-Null
$zh = JsWait "(() => { const t = document.querySelector('[data-bind=featureTips]'); return !!t && t.closest('.card').querySelector('.t').textContent === '功能提示'; })()" 10
$zhText = Js "document.querySelector('[data-bind=featureTips]').closest('.card').innerText"
Check 'in Chinese: 功能提示 with its description' ($zh -and $zhText -match '每条只出现一次') $zhText
PageRect '[data-bind=featureTips]' | Out-Null
Start-Sleep -Milliseconds 500
Shot 'p-switch-zh'
PageType '#settingsSearch' '小技巧'
Start-Sleep -Milliseconds 800
$hits = Js "document.getElementById('settingsSearchList').textContent"
Check "查找设置: '小技巧' finds 功能提示" ($hits -match '功能提示') $hits
PageType '#settingsSearch' ''

Case 'cleanup page'
PfQuit
}
