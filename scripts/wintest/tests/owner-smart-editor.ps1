# 智能栅栏编辑器 (新建智能栅栏…) holds its content in every interface language, on a debug build
# (build.sh --debug, run.sh --bin target/wintest-bin-debug: the page is read over CDP). In each of
# the eleven languages the window is opened from a fence's menu and, at the size it opened with,
# checked from the page for a new fence, each of the five templates (最近一周的截图 has the most
# rows: five, two of them with the type buttons) and two hand-made sets of four rows (位置, 名称,
# 大小 介于, 创建于星期; 位置, 创建时段, 快捷方式目标, 闲置天数): the list does not scroll, no control
# is cut by the list or the window, no drop-down is narrower than its longest option, no button's
# or caption's text is cut or wrapped, and no control of a row or of the bottom row dropped to a
# second line. Then, in Italian (the widest rows), twelve rows: the list scrolls and the bottom
# row stays in the window.
# --var Part=A: Everything not running (the long hint under the count); Part=B: Everything
# running (the short one). Part A saves the window with the first template as ed-<language>.png
# and with 最近一周的截图 as ed5-<language>.png.
if (-not $Part) { $Part = 'AB' }

# The page's own account of what does not fit: { bad: [what is wrong], info: {numbers} }.
$SeCheck = @'
() => {
  const w = document.getElementById('seWindow'), bad = [], info = {};
  const rc = e => e.getBoundingClientRect();
  const vis = e => { const r = rc(e); return r.width > 0 && r.height > 0; };
  const nm = e => { const c = [...e.classList].find(c => c.startsWith('se-')); const row = e.closest('.se-row'); return (row ? row.dataset.kind + ' row ' : '') + (c || e.tagName.toLowerCase()) + (e.tagName === 'BUTTON' || e.tagName === 'LABEL' ? ' "' + e.textContent.trim().slice(0, 24) + '"' : ''); };
  // How many lines the text of an element (or of a text node) takes.
  const lines = n => { const r = document.createRange(); if (n.nodeType === 3) r.selectNode(n); else r.selectNodeContents(n); const tops = [...r.getClientRects()].filter(q => q.width > 1 && q.height > 1).map(q => q.top).sort((a, b) => a - b); let k = 0, last = -1e9; for (const t of tops) { if (t - last > 6) k++; last = t; } return k; };
  const list = w.querySelector('.se-rows'), lr = rc(list), de = document.documentElement;
  info.rows = list.children.length; info.rowsNeed = Math.round(rc(list.lastElementChild).bottom - rc(list.firstElementChild).top); info.listHas = list.clientHeight; info.inner = innerWidth + 'x' + innerHeight;
  if (list.scrollHeight > list.clientHeight + 1) bad.push(`the list scrolls: its rows need ${list.scrollHeight} px, it has ${list.clientHeight}`);
  if (list.scrollWidth > list.clientWidth + 1) bad.push(`the list scrolls sideways: ${list.scrollWidth} > ${list.clientWidth}`);
  if (de.scrollHeight > innerHeight + 1 || de.scrollWidth > innerWidth + 1) bad.push(`the page is larger than the window: ${de.scrollWidth}x${de.scrollHeight} in ${innerWidth}x${innerHeight}`);
  // Every control whole inside the window; a row's controls whole inside the list.
  const cut = { list: [], window: [] };
  for (const e of w.querySelectorAll('h2, input:not([type=checkbox]), select, button, .se-chips label, .se-cap, .se-path, .se-n, .se-how, .se-match > span, .se-limit, .se-namel > span')) {
    if (!vis(e)) continue;
    const r = rc(e), inList = list.contains(e), box = inList ? lr : { left: 0, top: 0, right: innerWidth, bottom: innerHeight };
    if (r.left < box.left - 0.5 || r.right > box.right + 0.5 || r.top < box.top - 0.5 || r.bottom > box.bottom + 0.5) cut[inList ? 'list' : 'window'].push(nm(e));
  }
  for (const k of ['list', 'window']) if (cut[k].length) bad.push(`cut by the ${k}: ${cut[k].slice(0, 3).join('; ')}${cut[k].length > 3 ? ` and ${cut[k].length - 3} more` : ''}`);
  // A drop-down holds its longest option.
  const cv = document.createElement('canvas').getContext('2d');
  const tw = (e, s) => { const cs = getComputedStyle(e); cv.font = `${cs.fontWeight} ${cs.fontSize} ${cs.fontFamily}`; return cv.measureText(s).width; };
  const narrow = new Set();
  for (const s of w.querySelectorAll('select')) {
    if (!vis(s)) continue;
    const cs = getComputedStyle(s), room = s.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight);
    const o = [...s.options].map(o => [tw(s, o.text), o.text]).sort((a, b) => b[0] - a[0])[0];
    if (o && o[0] > room + 0.5) narrow.add(`the ${[...s.classList].find(c => c.startsWith('se-')) || 'limit'} drop-down is ${s.offsetWidth} px wide, its option "${o[1]}" needs ${Math.ceil(o[0] - room)} px more`);
  }
  bad.push(...narrow);
  // Buttons and captions keep their text whole and on one line.
  for (const b of w.querySelectorAll('button, .se-chips label')) {
    if (!vis(b)) continue;
    if (b.scrollWidth > b.clientWidth + 1 || b.scrollHeight > b.clientHeight + 1 || (b.tagName === 'BUTTON' && lines(b) > 1)) bad.push(`${nm(b)}: its text is cut or wrapped`);
  }
  for (const c of w.querySelectorAll('h2, .se-cap, .se-match > span, .se-namel > span')) if (vis(c) && lines(c) > 1) bad.push(`${nm(c)} "${c.textContent.trim().slice(0, 30)}" is on ${lines(c)} lines`);
  // The count (现在找到 N 个文件) on one line; a sentence in its place and the hint under it on two at most.
  const n = w.querySelector('.se-n'), how = w.querySelector('.se-how');
  if (n && vis(n) && lines(n) > (n.querySelector('b') ? 1 : 2)) bad.push(`the count line "${n.textContent.slice(0, 30)}" is on ${lines(n)} lines`);
  if (how && lines(how) > 2) bad.push(`the hint under the count is on ${lines(how)} lines`);
  const lim = w.querySelector('.se-limit');
  if (lines(lim.firstChild) > 1) bad.push(`the caption "${lim.firstChild.textContent}" is on ${lines(lim.firstChild)} lines`);
  // Nothing dropped to a second line: a row's controls, the bottom row, the two buttons, the 全部 / 任一 line.
  const oneLine = (what, els) => { els = els.filter(vis); if (els.length < 2) return; const top = Math.max(...els.map(e => rc(e).top)), bottom = Math.min(...els.map(e => rc(e).bottom)); if (top >= bottom) bad.push(`${what}: its parts are on more than one line`); };
  for (const row of list.children) {
    oneLine(`${row.dataset.kind} row`, [...row.querySelector('.se-val').children]);
    const p = row.querySelector('.se-path');
    if (p && p.clientWidth < Math.min(100, p.scrollWidth)) bad.push(`${row.dataset.kind} row: the folder's path has ${p.clientWidth} px`);
  }
  oneLine('the bottom row', [...w.querySelector('.se-foot').children]);
  oneLine('the two buttons', [...w.querySelectorAll('.se-btns button')]);
  oneLine('the all / any line', [...w.querySelectorAll('.se-match > :not(.se-always)')]);
  info.kind = list.firstElementChild.querySelector('.se-kind').offsetWidth;
  info.count = w.querySelector('.se-count').textContent.slice(0, 60);
  return { bad, info };
}
'@
# Puts a template (by its key) or a hand-made set of four rows into the editor; the rows' kinds.
$SeSet = @'
(what) => {
  const w = document.getElementById('seWindow');
  const rows = () => [...w.querySelectorAll('.se-row')];
  const fire = (e, type) => e.dispatchEvent(new Event(type, { bubbles: true }));
  const click = k => w.querySelector(`[data-preset=${k}]`).click();
  const kind = (i, k) => { const s = rows()[i].querySelector('.se-kind'); s.value = k; fire(s, 'change'); };
  const pick = (i, sel, v) => { const s = rows()[i].querySelector(sel); s.value = v; fire(s, 'change'); };
  const type = (i, v) => { const s = rows()[i].querySelector('.se-text'); s.value = v; fire(s, 'input'); };
  if (what === 'hand-made A') { click('downloads'); kind(1, 'name'); type(1, 'report'); kind(2, 'size'); pick(2, '.se-op', 'between'); kind(3, 'weekday'); }
  else if (what === 'hand-made B') { click('recent'); kind(1, 'created'); kind(2, 'target'); type(2, 'steam'); kind(3, 'idle'); }
  else if (what === 'many rows') { click('downloads'); for (let i = 0; i < 8; i++) rows().at(-1).querySelector('.se-plus').click(); }
  else if (what !== 'new') click(what);
  if (document.activeElement) document.activeElement.blur();
  return rows().map(r => r.dataset.kind).join();
}
'@
$Contents = [ordered]@{
  'new' = 'where,name'; 'downloads' = 'where,sub,time,type'; 'screenshots' = 'where,sub,time,type,type'; 'large' = 'where,type,size'
  'psd' = 'where,type,ext'; 'recent' = 'where,sub,type,ext'; 'hand-made A' = 'where,name,size,weekday'; 'hand-made B' = 'where,created,target,idle'
}

function Fences() { return @(@(PfJson fence list) | Where-Object { $_ -and $_.id -and $_.kind -ne 'inbox' }) }
function LastRow($m) { return (@(MenuRows $m) | Where-Object { $_.text } | Select-Object -Last 1) }
# 新建智能栅栏… in any language: the last row of 添加到桌面, the last row of a fence's menu. The
# editor's window with its page connected, or zero.
function OpenNewEditor() {
  $wr = (FenceGet (Fences)[0].id).windowRect
  [G]::RClick([int]($wr.x + $wr.w / 2), [int]($wr.y + 14)); Start-Sleep -Milliseconds 800
  $m = MenuWin 5
  $r = LastRow $m
  [G]::Click([int](($r.L + $r.R) / 2), [int](($r.T + $r.B) / 2)); Start-Sleep -Milliseconds 800
  $sub = @([W]::Tops('#32768')) | Where-Object { $_ -ne $m } | Select-Object -First 1
  $s = LastRow $sub
  Log "menu: $($r.text) > $($s.text)"
  [G]::Click([int](($s.L + $s.R) / 2), [int](($s.T + $s.B) / 2))
  return (EditorWin)
}
# Waits for the count that follows a change (asked 0.35 s after it).
function WaitCounted() {
  Start-Sleep -Milliseconds 600
  JsWait "(() => { const c = document.querySelector('#seWindow .se-count'); return !c.hidden && !c.classList.contains('busy'); })()" 8 | Out-Null
  Start-Sleep -Milliseconds 150
}
function EditorLanguages($shots) {
  trap { PfTrap $_; continue }
  foreach ($lang in @('en', 'zh-CN', 'zh-TW', 'ja', 'ko', 'de', 'fr', 'es', 'pt-BR', 'ru', 'it')) {
    Case "$lang, Everything $(if (EvRunning) { 'running' } else { 'not running' })"
    Pf settings set language $lang | Out-Null
    Start-Sleep -Seconds 2
    $ed = OpenNewEditor
    Check "$lang`: the editor opens" ($ed -ne [IntPtr]::Zero) ''
    if ($ed -eq [IntPtr]::Zero) { CloseMenus; continue }
    Js "(() => { window.seCheck = $SeCheck; window.seSet = $SeSet; return true; })()" | Out-Null
    $r = Rect $ed
    $wa = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea
    Check "$lang`: the window is inside the work area" ($r.L -ge $wa.Left -and $r.T -ge $wa.Top -and $r.R -le $wa.Right -and $r.B -le $wa.Bottom) "window $($r.R - $r.L)x$($r.B - $r.T) at $($r.L),$($r.T), work area $($wa.Width)x$($wa.Height)"
    foreach ($what in $Contents.Keys) {
      $rows = Js "window.seSet('$what')"
      WaitCounted
      $v = Js "window.seCheck()"
      $info = "window $($r.R - $r.L)x$($r.B - $r.T), page $($v.info.inner), $($v.info.rows) rows need $($v.info.rowsNeed) of $($v.info.listHas) px, first drop-down $($v.info.kind) px, count: $($v.info.count)"
      Check "$lang, $what`: all rows show, nothing cut or wrapped" ($rows -eq $Contents[$what] -and @($v.bad).Count -eq 0) "$(@($v.bad) -join ' | ') [$rows; $info]"
      if ($shots -and ($what -eq 'downloads' -or $what -eq 'screenshots')) {
        [G]::MoveTo($r.R + 60, $r.B + 20); Start-Sleep -Milliseconds 400
        Crop "$(if ($what -eq 'downloads') { 'ed' } else { 'ed5' })-$lang" $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) | Out-Null
      }
    }
    if ($lang -eq 'it') {
      # More rows than fit (twelve): that is what the list's scrollbar is for.
      $rows = Js "window.seSet('many rows')"
      WaitCounted
      $v = Js "(() => { const w = document.getElementById('seWindow'), l = w.querySelector('.se-rows'), b = w.querySelector('.se-btns').getBoundingClientRect(); return { rows: l.children.length, need: l.scrollHeight, has: l.clientHeight, page: document.documentElement.scrollHeight, inner: innerHeight, btnBottom: b.bottom }; })()"
      Check "$lang, twelve rows: the list scrolls, the page does not, the buttons stay in the window" ($v.rows -eq 12 -and $v.need -gt $v.has + 1 -and $v.page -le $v.inner + 1 -and $v.btnBottom -le $v.inner) ($v | ConvertTo-Json -Compress)
    }
    Js "(() => { document.querySelector('#seWindow .se-cancel').click(); return true; })()" | Out-Null
    Check "$lang`: 取消 closes the editor" (WaitGoneWin 'PecoFence.SmartEditor' 8) ''
    Start-Sleep -Milliseconds 400
  }
}

Case 'setup'
CloseWindows
DownloadsScene
Add-Type -AssemblyName System.Windows.Forms
Check 'a debug build (the page is read over CDP)' $PfDebugBuild
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'

if ($Part -match 'A') {
  Check 'Everything is not running' (EvStop)
  PfStart -Lang en
  Start-Sleep -Seconds 3
  EditorLanguages $true
  PfQuit
}
if ($Part -match 'B') {
  Check 'Everything runs' (EvStart)
  PfStart -Lang en
  Start-Sleep -Seconds 3
  EditorLanguages $false
  PfQuit
}

Case 'cleanup'
EvStop | Out-Null
ClearDownloads
