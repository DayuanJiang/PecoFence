# The settings page after the review of 2026-11-12 (drop-down lists, 查找设置, the keyboard in the
# nav, swatch rows and tile captions at the default size, the smallest window, text contrast, the
# rule editor, the example themes, wording, 视图 and 排序方式 on 栅栏, the nav's selected row).
# A debug build (build.sh --debug, run.sh --bin target/wintest-bin-debug): the page is read over
# CDP, the keys and clicks are real.
$mine = '季度报告 第三季度.docx', '照片.jpg', '预算表 2026 年度汇总（最终版）.xlsx'
function TidyDesk() { foreach ($n in $mine) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue } }

Case 'setup'
CloseWindows
TidyDesk
foreach ($n in $mine) { DeskFile $n | Out-Null }
Check 'a debug build (the page is reached over CDP)' $PfDebugBuild
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
PfStart -Lang zh-CN
Start-Sleep -Seconds 2
Pf fence create --title '工作文档' --rect '420,120,360,220' | Out-Null
Pf fence create --portal "$env:USERPROFILE\Documents" --rect '800,360,300,200' | Out-Null
Pf rule add --name '文档' --ext 'docx,xlsx,pdf' --to '工作文档' | Out-Null
Pf snapshot save '整理之前' | Out-Null
Check 'settings open over CDP' (PfSettings)
$win = [G]::FindWindowW('PecoFence.Settings', [NullString]::Value)
# Helpers in the page: pfRatio(colours behind, colours of the text or row) paints each list
# bottom to top and gives the contrast; pfBehind(el) lists the fills behind an element.
Js @"
(window.pfPaint = list => { const c = document.createElement('canvas'); c.width = c.height = 1; const x = c.getContext('2d', { willReadFrequently: true }); for (const col of list) { x.fillStyle = col; x.fillRect(0, 0, 1, 1); } const d = x.getImageData(0, 0, 1, 1).data; const l = [d[0], d[1], d[2]].map(v => { v /= 255; return v <= .03928 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4; }); return .2126 * l[0] + .7152 * l[1] + .0722 * l[2]; },
 window.pfRatio = (a, b) => { const x = pfPaint(a), y = pfPaint(b); return Math.round((Math.max(x, y) + .05) / (Math.min(x, y) + .05) * 100) / 100; },
 window.pfBehind = el => { const out = []; for (let e = el; e; e = e.parentElement) out.unshift(getComputedStyle(e).backgroundColor); return [getComputedStyle(document.documentElement).getPropertyValue('--sheet'), ...out]; },
 true)
"@ | Out-Null
function PageSet($q, $value) {
  $s = $q | ConvertTo-Json; $v = $value | ConvertTo-Json
  Js "(() => { const e = document.querySelector($s); e.value = $v; e.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
  Start-Sleep -Milliseconds 700
}
function FocusAt() { return Js "(() => { const a = document.activeElement; return !a || a === document.body ? 'none' : a.closest('nav') ? (a.dataset.page ? 'nav:' + a.dataset.page : 'search') : a.closest('main') ? 'page:' + (a.id || a.dataset.bind || a.dataset.fence || a.tagName) : 'other'; })()" }
function FindSetting($text) {
  $q = $text | ConvertTo-Json
  return @(Js "(() => { const b = document.getElementById('settingsSearch'); b.focus(); b.value = $q; b.dispatchEvent(new Event('input', { bubbles: true })); return [...document.querySelectorAll('#settingsSearchList [role=option] b')].map(x => x.textContent); })()")
}

Case 'the example themes show at the first click on 自定义'
$before = @(Explorers).Count
PageClick 'label.tile-custom'
$names = JsWait "(() => { const s = document.getElementById('customTheme'); return !s.disabled && s.options.length === 3 ? [...s.options].map(o => o.text).join('|') : null; })()" 8
Check 'the list shows the three examples' ([bool]$names) "list=$names; help=$(Js "document.getElementById('customThemeHelp').textContent")"
$theme = (PfJson settings get).customTheme
Check 'the first one is in use' ((Js "document.getElementById('customTheme').value") -and $theme) "customTheme=$theme"
Check 'no Explorer window opened' (@(Explorers).Count -eq $before) "$(@(Explorers) -join '; ')"
Shot 'themes-first-click'
PageClick 'label.tile-fluent'; Start-Sleep -Milliseconds 600

Case 'drop-down lists: the page draws the open list, the keys are the system''s'
$ua = Js 'navigator.userAgent'
$styled = Js "CSS.supports('appearance', 'base-select')"
Log "user agent $ua; base-select $styled"
Check 'this WebView2 can style an open list' $styled "$ua"
function ListState($q) { $s = $q | ConvertTo-Json; return Js "(() => { const s = document.querySelector($s); return { v: s.value, open: s.matches(':open'), look: getComputedStyle(s).appearance, row: Math.round(s.options[0].getBoundingClientRect().height) }; })()" }
PageClick '#page-general h2'
Js "(() => { const s = document.querySelector('[data-bind=titleAlign]'); s.scrollIntoView({ block: 'center' }); s.focus(); return true; })()" | Out-Null
$q = '[data-bind=titleAlign]'
[G]::Press(0x28); Start-Sleep -Milliseconds 300; $a = ListState $q
Check 'closed list, Down: the next choice, the list stays closed' ($a.v -eq 'center' -and -not $a.open) "$($a | ConvertTo-Json -Compress)"
[G]::Press(0x23); Start-Sleep -Milliseconds 300; $a = ListState $q
Check 'closed list, End: the last choice' ($a.v -eq 'right' -and -not $a.open) "$($a | ConvertTo-Json -Compress)"
[G]::Press(0x24); Start-Sleep -Milliseconds 300; $a = ListState $q
Check 'closed list, Home: the first choice' ($a.v -eq 'left' -and -not $a.open) "$($a | ConvertTo-Json -Compress)"
$saved = (PfJson settings get).titleAlign
Check 'the choice is saved' ($saved -eq 'left') "titleAlign=$saved"
Chord 0x12 0x28; Start-Sleep -Milliseconds 500; $a = ListState $q
Check 'Alt+Down opens the list; its rows are 36 px and drawn by the page' ($a.open -and $a.look -eq 'base-select' -and $a.row -eq 36) "$($a | ConvertTo-Json -Compress)"
Shot 'list-open-dark'
[G]::Press(0x28); Start-Sleep -Milliseconds 200; $mid = ListState $q
[G]::Press(0x0D); Start-Sleep -Milliseconds 400; $a = ListState $q
Check 'open list, Down then Enter: picks the next choice and closes' ($mid.v -eq 'left' -and $a.v -eq 'center' -and -not $a.open) "$($mid | ConvertTo-Json -Compress) $($a | ConvertTo-Json -Compress)"
[G]::Press(0x0D); Start-Sleep -Milliseconds 400; [G]::Press(0x28); Start-Sleep -Milliseconds 200; [G]::Press(0x1B); Start-Sleep -Milliseconds 400; $a = ListState $q
Check 'Enter opens, Down, Esc: closed, the choice unchanged, the window still open' ($a.v -eq 'center' -and -not $a.open -and [G]::IsWindowVisible($win)) "$($a | ConvertTo-Json -Compress)"
Check 'the focus is back on the list' ((FocusAt) -eq 'page:titleAlign') (FocusAt)
Js "(() => { const s = document.querySelector('[data-bind=""snapping.gapPx""]'); s.scrollIntoView({ block: 'center' }); s.focus(); return true; })()" | Out-Null
[G]::Type('1'); Start-Sleep -Milliseconds 1300; $a = ListState '[data-bind="snapping.gapPx"]'
Check 'closed list, typing 1: the first choice that starts with it (12 px)' ($a.v -eq '12' -and -not $a.open) "$($a | ConvertTo-Json -Compress)"
PageSet '[data-bind="snapping.gapPx"]' '8'
PageSet '[data-bind=titleAlign]' 'left'
$bars = Js "['nav', '.search-list', 'textarea', '.fb-preview'].map(q => getComputedStyle(document.querySelector(q), '::-webkit-scrollbar').width).join()"
Check 'the nav, the search list, the feedback box and its preview have the thin scrollbar' ($bars -eq '12px,12px,12px,12px') $bars

Case 'keyboard: the nav is one Tab stop'
PageClick '#page-general h2'
Chord 0x11 0x46; Start-Sleep -Milliseconds 300; $a0 = FocusAt
[G]::Press(0x09); Start-Sleep -Milliseconds 200; $a1 = FocusAt
[G]::Press(0x09); Start-Sleep -Milliseconds 200; $a2 = FocusAt
Check 'Ctrl+F, Tab, Tab: the search box, the nav, the page''s first control' ($a0 -eq 'search' -and $a1 -eq 'nav:general' -and $a2 -eq 'page:language') "$a0, $a1, $a2"
Chord 0x10 0x09; Start-Sleep -Milliseconds 200
[G]::Press(0x28); Start-Sleep -Milliseconds 150; [G]::Press(0x28); Start-Sleep -Milliseconds 200; $a3 = FocusAt
Check 'Shift+Tab, Down, Down: the focus is on 栅栏, the page is still 常规' ($a3 -eq 'nav:fences' -and (Js "document.getElementById('page-general').classList.contains('on')")) $a3
Shot 'nav-focus'
[G]::Press(0x0D); Start-Sleep -Milliseconds 500
$shown = Js "document.getElementById('page-fences').classList.contains('on')"
[G]::Press(0x09); Start-Sleep -Milliseconds 200; $a4 = FocusAt
Check 'Enter shows 栅栏 and the next Tab is on its first control' ($shown -and $a4 -eq 'page:fenceSel') "shown=$shown, $a4"
Chord 0x10 0x09; Start-Sleep -Milliseconds 200; [G]::Press(0x24); Start-Sleep -Milliseconds 200; $a5 = FocusAt
[G]::Press(0x20); Start-Sleep -Milliseconds 500
Check 'Home goes to 常规 and Space shows it' ($a5 -eq 'nav:general' -and (Js "document.getElementById('page-general').classList.contains('on')")) $a5

Case '查找设置'
$r = FindSetting '快照'
Check '快照 finds the setting and no saved snapshot' ($r -contains '把当前布局存为快照' -and $r -notcontains '整理之前') ($r -join ' | ')
$r = FindSetting '文档'
Check 'a rule is not listed as a setting' ($r -notcontains '文档') ($r -join ' | ')
foreach ($p in @(@('帮助', '使用手册'), @('关于', '使用手册'), @('命令行', '让 AI 助手整理桌面'), @('撤销', '把当前布局存为快照'), @('dark', '颜色模式'), @('排序', '排序方式'))) {
  $r = FindSetting $p[0]
  Check "$($p[0]) finds $($p[1])" ($r -contains $p[1]) ($r -join ' | ')
}
foreach ($mode in 'light', 'dark') {
  PageSet '[data-bind=theme]' $mode
  FindSetting '图标' | Out-Null
  Start-Sleep -Milliseconds 300
  $c = Js "(() => { const l = document.getElementById('settingsSearchList'), row = l.querySelector('[aria-selected=true]'); return pfRatio(pfBehind(l), pfBehind(row)); })()"
  Check "$mode mode: the selected result stands out from the list (at least 1.2 : 1)" ($c -ge 1.2) "$c : 1"
  if ($mode -eq 'light') { Shot 'search-light' }
}
Js "(() => { const b = document.getElementById('settingsSearch'); b.value = ''; b.dispatchEvent(new Event('input', { bubbles: true })); b.blur(); return true; })()" | Out-Null

Case 'the default window size'
$inner = Js '[innerWidth, innerHeight].join()'
PageClick 'nav a[data-page=fences]'; Start-Sleep -Milliseconds 500
PageSet '#fenceSel' (FenceGet '工作文档').id
$w = Js "(() => { const row = document.getElementById('fenceTintSwatches'), l = row.closest('.card').querySelector('.l').getBoundingClientRect(), b = row.getBoundingClientRect(); return { n: row.children.length, beside: b.top < l.bottom && b.left >= l.right, card: Math.round(row.closest('.card').getBoundingClientRect().height) }; })()"
Check '色调: the 11 swatches stand beside the text' ($w.n -eq 11 -and $w.beside) "inner $inner; $($w | ConvertTo-Json -Compress)"
PageClick 'nav a[data-page=general]'; Start-Sleep -Milliseconds 500
$cap = Js "(() => { const c = document.querySelector('.tile-glass .cap'); return { text: c.textContent.trim(), h: Math.round(c.getBoundingClientRect().height) }; })()"
Check 'Liquid Glass · 液态玻璃 is on one line' ($cap.h -le 22) "$($cap | ConvertTo-Json -Compress)"

Case 'placeholders and small grey text'
PageClick 'nav a[data-page=about]'; Start-Sleep -Milliseconds 500
foreach ($mode in 'dark', 'light') {
  PageSet '[data-bind=theme]' $mode
  $c = Js "(() => { const r = (q, pseudo) => { const e = document.querySelector(q), back = pfBehind(e); return pfRatio(back, [...back, getComputedStyle(e, pseudo).color]); }; return { feedback: r('#fbMessage', '::placeholder'), search: r('#settingsSearch', '::placeholder'), count: r('#fbCount'), summary: r('#workspaceSummary') }; })()"
  Check "$mode mode: placeholders and small grey text are at least 4.5 : 1" ($c.feedback -ge 4.5 -and $c.search -ge 4.5 -and $c.count -ge 4.5 -and $c.summary -ge 4.5) "$($c | ConvertTo-Json -Compress)"
}
PageSet '[data-bind=theme]' 'dark'

Case 'rule editor'
PageClick 'nav a[data-page=rules]'; Start-Sleep -Milliseconds 500
Js "(document.getElementById('nrPanel').open = true)" | Out-Null
Start-Sleep -Milliseconds 500
PageSet '#nrKind' 'exact'
$k = Js "(() => { const s = document.getElementById('nrKind'), pen = document.createElement('canvas').getContext('2d'); pen.font = getComputedStyle(s).font; return { need: Math.round(pen.measureText(s.selectedOptions[0].text).width + 43), has: Math.round(s.getBoundingClientRect().width), text: s.selectedOptions[0].text }; })()"
Check 'the chosen condition type is shown whole' ($k.has -ge $k.need) "$($k | ConvertTo-Json -Compress)"
PageSet '#nrKind' 'type'
$cols = Js "(() => { const b = [...document.querySelectorAll('#nrCats label')].map(l => l.getBoundingClientRect()); const lefts = [...new Set(b.map(x => Math.round(x.left)))], first = b.filter(x => Math.round(x.top) === Math.round(b[0].top)).map(x => Math.round(x.left)); return { boxes: b.length, lefts: lefts.length, columns: first.length, aligned: lefts.every(x => first.includes(x)) }; })()"
Check 'the nine file type boxes stand in columns' ($cols.boxes -eq 9 -and $cols.aligned) "$($cols | ConvertTo-Json -Compress)"
Shot 'rule-editor'

Case 'wording'
$bad = Js "(() => { const t = document.querySelector('main').textContent; return [String.fromCharCode(0x300c), String.fromCharCode(0x300d), '关掉', '关上', '标题行', 'Chameleon', '没有命中'].filter(w => t.includes(w)).join(' '); })()"
Check 'one kind of quotation mark and one word for a switch that is off' (-not $bad) "still on the page: $bad"
$titles = @(Js "[...document.querySelectorAll('main .card .t')].map(e => e.textContent.trim())")
Check 'one card is named Windows 桌面图标, the other 修复 Windows 桌面图标' (@($titles | Where-Object { $_ -eq 'Windows 桌面图标' }).Count -eq 1 -and $titles -contains '修复 Windows 桌面图标') ''
$opts = @(Js "[...document.getElementById('fenceSel').options].map(o => o.text)")
Check 'the fence list says 桌面 and marks the folder fence' ($opts -contains '桌面' -and @($opts | Where-Object { $_ -match '（文件夹栅栏）$' }).Count -eq 1) ($opts -join ' | ')

Case '栅栏: 视图 and 排序方式'
PageClick 'nav a[data-page=fences]'; Start-Sleep -Milliseconds 500
PageSet '#fenceSel' (FenceGet '工作文档').id
$has = Js "!!document.getElementById('fenceLayout') && !!document.getElementById('fenceSort')"
Check 'the page has 视图 and 排序方式' $has ''
Js "(() => { const s = document.getElementById('fenceLayout'); s.scrollIntoView({ block: 'center' }); s.focus(); return true; })()" | Out-Null
[G]::Press(0x28); Start-Sleep -Milliseconds 800
$f = FenceGet '工作文档'
Check '视图, Down on the list: the fence shows a list' ($f.layout -eq 'list') "layout=$($f.layout)"
PageSet '#fenceSort' 'name'
PageClick '#page-fences .toggle[data-fence=reverse]'; Start-Sleep -Milliseconds 500
PageClick '#page-fences .toggle[data-fence=foldersFirst]'; Start-Sleep -Milliseconds 500
$f = FenceGet '工作文档'
Check '排序方式 按名称, 倒序 and 文件夹优先 reach the fence' ($f.sort -eq 'name' -and $f.reverse -and $f.foldersFirst) "sort=$($f.sort) reverse=$($f.reverse) foldersFirst=$($f.foldersFirst)"
Pf fence set '工作文档' sort date | Out-Null
Pf fence set '工作文档' layout icons | Out-Null
$back = JsWait "document.getElementById('fenceSort').value === 'date' && document.getElementById('fenceLayout').value === 'icons' && document.querySelector('#page-fences .toggle[data-fence=reverse]').classList.contains('on')" 8
Check 'a change made with the CLI shows on the page' $back "sort=$(Js "document.getElementById('fenceSort').value") view=$(Js "document.getElementById('fenceLayout').value")"
Shot 'fence-view-sort'

Case 'the nav''s selected row'
foreach ($mode in 'dark', 'light') {
  PageSet '[data-bind=theme]' $mode
  Start-Sleep -Milliseconds 300
  $n = Js "(() => { const a = document.querySelector('nav a.sel'), o = document.querySelector('nav a:not(.sel)'), back = pfBehind(a.parentElement); return { bar: getComputedStyle(a, '::before').content, tint: pfRatio(back, [...back, getComputedStyle(a).backgroundColor]), text: pfRatio([...back, getComputedStyle(a).backgroundColor], [...back, getComputedStyle(a).backgroundColor, getComputedStyle(a).color]), accent: getComputedStyle(a).color !== getComputedStyle(o).color }; })()"
  Check "$mode mode: a tint and the accent colour, no bar at the row's edge" (($n.bar -eq 'none' -or $n.bar -eq 'normal') -and $n.tint -ge 1.2 -and $n.text -ge 4.5 -and $n.accent) "$($n | ConvertTo-Json -Compress)"
  Shot "nav-$mode"
}
PageSet '[data-bind=theme]' 'dark'

Case 'the smallest window'
[L]::MoveWindow($win, 300, 150, 200, 150, $true) | Out-Null
Start-Sleep -Milliseconds 900
$m = Js "(() => { const n = document.querySelector('nav'), b = document.getElementById('settingsSearch'); return { inner: innerWidth + 'x' + innerHeight, sideways: n.scrollWidth > n.clientWidth, down: n.scrollHeight > n.clientHeight, cut: b.getBoundingClientRect().right > n.getBoundingClientRect().left + n.clientWidth, squeezed: [...n.querySelectorAll('a')].filter(a => a.scrollHeight > a.clientHeight).length }; })()"
Check 'zh-CN: no scrollbar in the nav, the search box whole' (-not $m.sideways -and -not $m.down -and -not $m.cut -and $m.squeezed -eq 0) "$($m | ConvertTo-Json -Compress)"
Shot 'min-zh'
Pf settings set language ru | Out-Null
Start-Sleep -Seconds 2
$m = Js "(() => { const n = document.querySelector('nav'), b = document.getElementById('settingsSearch'); return { sideways: n.scrollWidth > n.clientWidth, cut: b.getBoundingClientRect().right > n.getBoundingClientRect().left + n.clientWidth, squeezed: [...n.querySelectorAll('a')].filter(a => a.scrollHeight > a.clientHeight).length }; })()"
Check 'Russian: no sideways scrolling, the search box whole, no page name squeezed' (-not $m.sideways -and -not $m.cut -and $m.squeezed -eq 0) "$($m | ConvertTo-Json -Compress)"
Shot 'min-ru'
Pf settings set language zh-CN | Out-Null

Case 'cleanup'
TidyDesk
Check 'the test files are gone from the desktop' (-not (Test-Path -LiteralPath (Join-Path $PfDesk $mine[0]))) ''
