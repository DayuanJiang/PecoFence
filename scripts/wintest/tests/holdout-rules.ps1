# Hold-out test: rule editing in Settings > Sorting rules (feedback #104), done with real clicks
# and typing in the settings page. Debug build: element rects and DOM state over CDP (WebView2
# with --remote-debugging-port=9352). Release build: the main path (R1) only, by keyboard and
# client coordinates measured in the debug run.

# ---- lib candidates

# Client origin of a window (the settings page's (0,0) is the client area's top left).
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class CW {
  [StructLayout(LayoutKind.Sequential)] public struct P { public int X, Y; }
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref P p);
  [DllImport("user32.dll")] static extern void mouse_event(uint f, int dx, int dy, int d, UIntPtr e);
  public static void Wheel(int notches) { mouse_event(0x0800, 0, 0, notches * 120, UIntPtr.Zero); }
}
'@
function ClientOrigin($h) { $p = New-Object CW+P; [CW]::ClientToScreen($h, [ref]$p) | Out-Null; return $p }
# Mouse wheel over a client point of a window (negative notches scroll down).
function WheelAt($h, $cx, $cy, $notches) { $o = ClientOrigin $h; [G]::MoveTo($o.X + $cx, $o.Y + $cy); Start-Sleep -Milliseconds 100; [CW]::Wheel($notches); Start-Sleep -Milliseconds 900 }

# Whether the build under test is a debug build (only those honour PECOFENCE_WEBVIEW_ARGS).
function IsDebugBuild() { return [Text.Encoding]::ASCII.GetString([IO.File]::ReadAllBytes("$PfBin\pecofence.exe")).Contains('PECOFENCE_WEBVIEW_ARGS') }

# CDP: connects to the settings page of a debug build started with
# PECOFENCE_WEBVIEW_ARGS=--remote-debugging-port=9352; $true when connected.
function CdpOpen($port = 9352, $sec = 30) {
  for ($i = 0; $i -lt $sec; $i++) {
    try {
      $list = (New-Object Net.WebClient).DownloadString("http://127.0.0.1:$port/json") | ConvertFrom-Json
      $page = @($list | Where-Object { $_.type -eq 'page' }) | Select-Object -First 1
      if ($page) {
        $ws = New-Object Net.WebSockets.ClientWebSocket
        $ws.ConnectAsync([Uri]$page.webSocketDebuggerUrl, [Threading.CancellationToken]::None).Wait(10000) | Out-Null
        if ($ws.State -eq 'Open') { $script:Cdp = $ws; $script:CdpN = 0; Log "cdp connected: $($page.webSocketDebuggerUrl)"; return $true }
      }
    } catch { Log "cdp: $($_.Exception.Message)" }
    Start-Sleep -Seconds 1
  }
  return $false
}

# Value of a JavaScript expression in the page (through JSON); throws on a page exception.
function Js([string]$expr) {
  $script:CdpN++
  $id = $script:CdpN
  $body = @{ id = $id; method = 'Runtime.evaluate'; params = @{ expression = "JSON.stringify({ v: ($expr) })"; returnByValue = $true } } | ConvertTo-Json -Depth 5 -Compress
  $bytes = [Text.Encoding]::UTF8.GetBytes($body)
  $none = [Threading.CancellationToken]::None
  $script:Cdp.SendAsync((New-Object 'ArraySegment[byte]' -ArgumentList (, $bytes)), [Net.WebSockets.WebSocketMessageType]::Text, $true, $none).Wait()
  while ($true) {
    $ms = New-Object IO.MemoryStream
    $buf = New-Object byte[] 65536
    do {
      $r = $script:Cdp.ReceiveAsync((New-Object 'ArraySegment[byte]' -ArgumentList (, $buf)), $none).Result
      $ms.Write($buf, 0, $r.Count)
    } while (-not $r.EndOfMessage)
    $resp = [Text.Encoding]::UTF8.GetString($ms.ToArray()) | ConvertFrom-Json
    if ($resp.id -eq $id) { break }
  }
  if ($resp.result.exceptionDetails) { throw "page exception: $($resp.result.exceptionDetails.exception.description)" }
  $s = $resp.result.result.value
  if (-not $s) { return $null }
  return ($s | ConvertFrom-Json).v
}
function JsWait([string]$expr, $sec = 10) {
  for ($i = 0; $i -lt $sec * 5; $i++) { $v = Js $expr; if ($v) { return $v }; Start-Sleep -Milliseconds 200 }
  return $v
}

# Screen rect of the i-th element matching a CSS selector (scrolled into view when outside);
# cx/cy = centre, client = the centre relative to the window's client origin.
function PageRect([string]$sel, [int]$i = 0) {
  $q = $sel | ConvertTo-Json
  $r = Js "(() => { const e = document.querySelectorAll($q)[$i]; if (!e) return null; let b = e.getBoundingClientRect(); if (b.top < 0 || b.bottom > innerHeight) { e.scrollIntoView({ block: 'center', behavior: 'instant' }); b = e.getBoundingClientRect(); } const d = devicePixelRatio; return { x: b.left * d, y: b.top * d, w: b.width * d, h: b.height * d }; })()"
  if (-not $r) { return $null }
  $o = ClientOrigin ([G]::FindWindowW('PecoFence.Settings', [NullString]::Value))
  return @{ x = [int]($o.X + $r.x); y = [int]($o.Y + $r.y); w = [int]$r.w; h = [int]$r.h
    cx = [int]($o.X + $r.x + $r.w / 2); cy = [int]($o.Y + $r.y + $r.h / 2); client = "$([int]($r.x + $r.w / 2)),$([int]($r.y + $r.h / 2))" }
}
# A real click on the i-th element matching a selector ($dx: that far from its left edge).
function PageClick([string]$sel, [int]$i = 0, $dx = $null) {
  $r = PageRect $sel $i
  if (-not $r) { throw "no element $sel [$i]" }
  $x = if ($null -ne $dx) { $r.x + $dx } else { $r.cx }
  [G]::Click($x, $r.cy); Start-Sleep -Milliseconds 350
  Log "click $sel [$i] at $x,$($r.cy) (client $($r.client))"
}
# Click into a text box, select all and type (an empty text deletes what is there).
function PageType([string]$sel, [string]$text, [int]$i = 0) {
  PageClick $sel $i
  Chord 0x11 0x41; Start-Sleep -Milliseconds 80
  if ($text -eq '') { [G]::Press(0x2E) } else { [G]::Type($text) }
  Start-Sleep -Milliseconds 250
}
# Picks an option of a <select> by keyboard: a click on its label's caption focuses it (a click
# on the select opens a popup whose keys proved unreliable), then arrow keys on the closed select.
function PagePick([string]$sel, [string]$value, [int]$i = 0) {
  $q = $sel | ConvertTo-Json; $vq = $value | ConvertTo-Json
  $ix = Js "(() => { const s = document.querySelectorAll($q)[$i]; return s ? [s.selectedIndex, [...s.options].findIndex(o => o.value === $vq)] : null; })()"
  if (-not $ix -or $ix[1] -lt 0) { throw "no option $value in $sel [$i]" }
  PageRect $sel $i | Out-Null
  $lab = Js "(() => { const l = document.querySelectorAll($q)[$i].closest('label'); if (!l) return null; const b = l.getBoundingClientRect(), d = devicePixelRatio; return { x: (b.left + 6) * d, y: (b.top + 6) * d }; })()"
  if (-not $lab) { throw "no label around $sel [$i]" }
  $o = ClientOrigin ([G]::FindWindowW('PecoFence.Settings', [NullString]::Value))
  [G]::Click([int]($o.X + $lab.x), [int]($o.Y + $lab.y)); Start-Sleep -Milliseconds 300
  if (-not (Js "document.activeElement === document.querySelectorAll($q)[$i]")) { throw "a click on the caption did not focus $sel [$i]" }
  $n = $ix[1] - $ix[0]
  $vk = if ($n -gt 0) { 0x28 } else { 0x26 }
  for ($k = 0; $k -lt [Math]::Abs($n); $k++) { [G]::Press($vk); Start-Sleep -Milliseconds 80 }
  Start-Sleep -Milliseconds 250
  $now = Js "document.querySelectorAll($q)[$i].value"
  if ($now -ne $value) { throw "picked '$now' instead of '$value' in $sel [$i]" }
}
# The settings page's message bar: { on, error, text }.
function PageToast() { return Js "(() => { const e = document.getElementById('toast'); return { on: e.classList.contains('on'), error: e.classList.contains('error'), text: document.getElementById('toastText').textContent }; })()" }

# Opens Settings (pecofence-cli settings open-ui) and, on a debug build, connects over CDP.
function PfSettings() {
  Pf settings open-ui | Out-Null
  $h = WaitWin 'PecoFence.Settings' 30
  if ($h -eq [IntPtr]::Zero) { return $false }
  if (-not $PfDebugBuild) { Start-Sleep -Seconds 6; return $true }
  if (-not (CdpOpen)) { return $false }
  return [bool](JsWait "document.documentElement.classList.contains('ready') && document.querySelectorAll('#ruleList [data-row-id]').length > 0" 20)
}

# Desktop files: one with that name (and a line of text), and the fence title of a desktop item.
function DeskFile($name) { $p = Join-Path $PfDesk $name; Set-Content -LiteralPath $p -Value "hold-out $name" -Encoding ASCII; return $p }
function ItemFence($file) {
  $j = PfJson item list
  foreach ($it in @($j.items) + @($j)) { if ($it -and $it.fileName -eq $file) { return $it.fenceTitle } }
  return $null
}
function WaitItemFence($file, $want, $sec = 10) {
  for ($k = 0; $k -lt $sec; $k++) { $f = ItemFence $file; if ($f -eq $want) { return $f }; Start-Sleep -Seconds 1 }
  return $f
}

# The app log of this job across restarts: a start renames the last log to .prev.log, so
# after a second PfStart the current log no longer has the lines of the first run.
function PfLogAll() {
  $p = PfLogPath
  return @(@(($p -replace '\.log$', '.prev.log'), $p) | Where-Object { Test-Path $_ } | ForEach-Object { Get-Content $_ -Encoding UTF8 })
}

# Rules from `rule list`; one rule as a comparable string; a rule by id.
function Rules() { return @((PfJson rule list).list) }
function RuleSig($r) { return ($r | Select-Object id, name, enabled, target, allOf, priorityClass, template | ConvertTo-Json -Depth 8 -Compress) }
function RuleById($id) { return (Rules | Where-Object { $_.id -eq $id } | Select-Object -First 1) }
function WaitRule($id, [scriptblock]$ok, $sec = 8) {
  for ($k = 0; $k -lt $sec * 2; $k++) { $r = RuleById $id; if ($r -and (& $ok $r)) { return $r }; Start-Sleep -Milliseconds 500 }
  return $r
}

# ---- steps

$EditorJs = @'
(() => { const f = document.querySelector('#ruleList .rule.editing'); if (!f) return { open: false };
  const conds = [...f.querySelectorAll('.cond')], q = (c, s) => c.querySelector(s);
  const a = document.activeElement;
  return { open: true, id: f.dataset.rowId, head: (q(f, '.re-head .t') || {}).textContent,
    name: q(f, '.r-name').value, target: q(f, '.r-target').value,
    targets: [...q(f, '.r-target').options].map(o => o.textContent),
    kinds: conds.map(c => q(c, '.c-kind') ? q(c, '.c-kind').value : 'fixed'),
    texts: conds.map(c => q(c, '.c-text') ? q(c, '.c-text').value : null),
    fixed: conds.map(c => q(c, '.c-fixed') ? q(c, '.c-fixed').textContent : null),
    cats: conds.map(c => [...c.querySelectorAll('.c-cats input:checked')].map(i => i.value).join(',')),
    days: conds.map(c => q(c, '.c-days') ? q(c, '.c-days').value : null),
    dels: conds.map(c => q(c, '.c-del').hidden),
    save: q(f, '[data-act=save]').textContent,
    active: a ? (a.dataset.act || a.className || a.tagName) : null,
    editOff: document.querySelectorAll('#ruleList [data-act=edit]:disabled').length };
})()
'@
$RowsJs = "[...document.querySelectorAll('#ruleList [data-row-id]')].map(r => ({ id: r.dataset.rowId, name: (r.querySelector('.t') || {}).textContent, disabled: r.classList.contains('disabled'), editing: r.classList.contains('editing') }))"
function Ed() { return (Js $EditorJs) }
function Row($id) { return (@(Js $RowsJs) | Where-Object { $_.id -eq $id } | Select-Object -First 1) }
function EditSel($id) { return "#ruleList [data-row-id=`"$id`"] [data-act=edit]" }
function CondSel($k) { return "#ruleList .rule.editing .cond-list > .cond:nth-child($($k + 2))" }
$E = '#ruleList .rule.editing'

# Client coordinates (100 %, 1600x900) the release run clicks, measured in the debug run.
# The page scrolls by wheel: to the bottom for the rule rows, back to the top for Apply now.
$Fixed = @{ nav = '120,166'; edit2 = '699,297'; apply = '827,294' }
function FixedClick($name) {
  $h = [G]::FindWindowW('PecoFence.Settings', [NullString]::Value)
  $o = ClientOrigin $h
  $xy = $Fixed[$name] -split ','
  [G]::Click($o.X + [int]$xy[0], $o.Y + [int]$xy[1]); Start-Sleep -Milliseconds 500
  Log "fixed click $name at client $($Fixed[$name])"
}

$Made = @('hr-alpha.txt', 'hr-beta.txt', 'hr3-keep.txt', 'hr3-other.txt', 'hr3-keep.log', 'zz-keep.txt', 'zz-none.png', 'hr11.md')
function CleanDesk() { foreach ($n in $Made) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue } }

Case 'setup'
$PfDebugBuild = IsDebugBuild
Log "debug build: $PfDebugBuild"
CloseWindows
Set-Culture en-US
CleanDesk
DeskFile 'hr-alpha.txt' | Out-Null
DeskFile 'hr-beta.txt' | Out-Null
if ($PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
PfStart
Pf fence create --title Alpha --rect '40,560,300,200' | Out-Null
Pf fence create --title Beta --rect '360,560,300,200' | Out-Null
Start-Sleep -Seconds 2
$alpha = PfFence 'Alpha'; $beta = PfFence 'Beta'
$rules0 = Rules
$docs = $rules0 | Where-Object { ($_.allOf | ConvertTo-Json -Depth 5 -Compress) -match 'documents' } | Select-Object -First 1
$docsIx = [array]::IndexOf(@($rules0 | ForEach-Object { $_.id }), $docs.id)
Check 'a fresh instance has the first-run rules' ($rules0.Count -eq 3 -and $docs -and $alpha -and $beta) "rules=$($rules0.Count) docs='$($docs.name)' at $docsIx; fences Alpha=$($alpha.id) Beta=$($beta.id)"
Check 'the desktop files start in the documents fence' ((WaitItemFence 'hr-alpha.txt' $docs.targetTitle) -eq $docs.targetTitle) "hr-alpha.txt in '$(ItemFence 'hr-alpha.txt')', want '$($docs.targetTitle)'"
$opened = PfSettings
Check 'Settings opens (and CDP connects on a debug build)' $opened "debug=$PfDebugBuild"
if ($PfDebugBuild) {
  $nav = PageRect 'nav a[data-page=rules]'
  PageClick 'nav a[data-page=rules]'
  Check 'the Sorting rules page shows' (JsWait "document.getElementById('page-rules').classList.contains('on')" 5)
  $rowsN = @(Js $RowsJs).Count
  Check 'the page lists the same rules as the CLI' ($rowsN -eq $rules0.Count) "page=$rowsN cli=$($rules0.Count)"
} else {
  FixedClick nav
}
$sh = [G]::FindWindowW('PecoFence.Settings', [NullString]::Value)
WheelAt $sh 260 400 -30
Shot 'r00-rules-page'

Case 'R1 edit a default rule'
$before = $docs
if ($PfDebugBuild) {
  $er = PageRect (EditSel $docs.id)
  WheelAt $sh 260 400 30
  $ap = PageRect '#applyRules'
  WheelAt $sh 260 400 -30
  Log "coords nav=$($nav.client) edit2=$($er.client) apply=$($ap.client)"
  Check 'the fixed release coordinates still match the page' ($Fixed.nav -eq $nav.client -and $Fixed.edit2 -eq $er.client -and $Fixed.apply -eq $ap.client) "measured nav=$($nav.client) edit2=$($er.client) apply=$($ap.client); table nav=$($Fixed.nav) edit2=$($Fixed.edit2) apply=$($Fixed.apply)"
  PageClick (EditSel $docs.id)
  $ed = Ed
  Check 'the editor opens with the rule as stored' ($ed.open -and $ed.id -eq $docs.id -and $ed.name -eq $docs.name -and $ed.target -eq $docs.target.fence -and $ed.kinds -join ',' -eq 'type' -and $ed.active -eq 'r-name') "open=$($ed.open) name='$($ed.name)' target=$($ed.target) kinds=$($ed.kinds -join ',') cats=$($ed.cats -join ';') active=$($ed.active)"
} else {
  FixedClick edit2
}
# Keyboard from here, the same in both builds: name, Tab, target by its first letters, Tab,
# condition kind two down (File type > Extension > Name contains), Tab, value, Tab x3 = Save.
Chord 0x11 0x41; Start-Sleep -Milliseconds 80
[G]::Type('Alpha notes'); Start-Sleep -Milliseconds 200
[G]::Press(0x09); Start-Sleep -Milliseconds 200
[G]::Type('Al'); Start-Sleep -Milliseconds 400
[G]::Press(0x09); Start-Sleep -Milliseconds 200
[G]::Press(0x28); Start-Sleep -Milliseconds 150; [G]::Press(0x28); Start-Sleep -Milliseconds 300
[G]::Press(0x09); Start-Sleep -Milliseconds 200
[G]::Type('alpha'); Start-Sleep -Milliseconds 200
if ($PfDebugBuild) {
  $ed = Ed
  Check 'typed into the editor: name, target, kind, value' ($ed.name -eq 'Alpha notes' -and $ed.target -eq $alpha.id -and $ed.kinds -join ',' -eq 'contains' -and $ed.texts -join ',' -eq 'alpha') "name='$($ed.name)' target=$($ed.target) kinds=$($ed.kinds -join ',') texts=$($ed.texts -join ',') active=$($ed.active)"
}
foreach ($k in 1..3) { [G]::Press(0x09); Start-Sleep -Milliseconds 150 }
if ($PfDebugBuild) { $ed = Ed; Check 'three Tabs reach Save' ($ed.active -eq 'save') "active=$($ed.active)" }
[G]::Press(0x0D); Start-Sleep -Milliseconds 300
WheelAt $sh 260 400 30
if ($PfDebugBuild) {
  $t = PageToast
  Check 'Save closes the editor and says Rule saved' (-not (Ed).open -and $t.text -eq 'Rule saved' -and -not $t.error) "toast='$($t.text)' error=$($t.error)"
  $row = Row $docs.id
  Check 'the row shows the new name' ($row.name -eq 'Alpha notes') "row='$($row.name)'"
}
$r1 = WaitRule $docs.id { param($r) $r.name -eq 'Alpha notes' }
$ids1 = @(Rules | ForEach-Object { $_.id })
Check 'rule list: new name, target and condition' ($r1.name -eq 'Alpha notes' -and $r1.target.fence -eq $alpha.id -and $r1.targetTitle -eq 'Alpha' -and ($r1.allOf | ConvertTo-Json -Compress -Depth 5) -eq '{"cond":"name","value":{"op":"contains","value":"alpha"}}' -and $r1.priorityClass -eq 'name') (RuleSig $r1)
Check 'rule list: same id, place and switch' ($r1.id -eq $before.id -and $r1.enabled -eq $before.enabled -and [array]::IndexOf($ids1, $docs.id) -eq $docsIx -and $ids1.Count -eq 3) "index=$([array]::IndexOf($ids1, $docs.id)) (was $docsIx) enabled=$($r1.enabled) count=$($ids1.Count)"
Check 'saving does not move items by itself' ((ItemFence 'hr-alpha.txt') -eq $docs.targetTitle) "hr-alpha.txt in '$(ItemFence 'hr-alpha.txt')'"
if ($PfDebugBuild) { PageClick '#applyRules' } else { FixedClick apply }
if ($PfDebugBuild) { $ok = JsWait "document.getElementById('toastText').textContent.startsWith('Rules applied')" 5; Check 'Apply now reports what it sorted' $ok "toast='$((PageToast).text)'" }
$fl = PfJson fence list
$inbox = @($fl.fences) + @($fl) | Where-Object { $_ -and $_.kind -eq 'inbox' } | Select-Object -First 1
Check 'Apply now moves the matching item into Alpha' ((WaitItemFence 'hr-alpha.txt' 'Alpha' 8) -eq 'Alpha') "hr-alpha.txt in '$(ItemFence 'hr-alpha.txt')'"
Check 'and the document no rule matches any more to the desktop fence' ((WaitItemFence 'hr-beta.txt' $inbox.title 5) -eq $inbox.title) "hr-beta.txt in '$(ItemFence 'hr-beta.txt')', inbox '$($inbox.title)'"
Shot 'r01-after-apply'

if (-not $PfDebugBuild) {
  Case 'R1 restart (release)'
  PfQuit
  PfStart
  $r = RuleById $docs.id
  Check 'the edited rule is there after a restart' ((RuleSig $r) -eq (RuleSig $r1)) (RuleSig $r)
  $saved = @(PfLogAll | Where-Object { $_ -match 'rules saved from page rules=\d+' })
  $bad = @(PfLogAll | Where-Object { $_ -match 'bad rules from page' })
  $errs = @(PfLogAll | Where-Object { $_ -match ' ERROR |panicked' })
  Check 'no ERROR or panic line before the restart either' ($errs.Count -eq 0) (($errs | Select-Object -First 2) -join ' | ')
  Check 'the log has rules saved from page and no bad rules' ($saved.Count -ge 1 -and $bad.Count -eq 0) "saved=$($saved.Count) bad=$($bad.Count) $($saved | Select-Object -Last 1)"
  Case 'cleanup'
  PfQuit
  CleanDesk
  return
}

Case 'R2 cancel changes nothing'
$sigs = @(Rules | ForEach-Object { RuleSig $_ })
$prog = (Rules)[0]
PageClick (EditSel $prog.id)
PageType "$E .r-name" 'Changed name'
PagePick "$E .r-target" $beta.id
PagePick "$E .c-kind" 'endsWith'
PageType "$E .c-text" '_zz'
PageClick "$E .c-add"
PagePick "$E .c-kind" 'glob' 1
PageType "$E .c-text" '*.zz' 1
$ed = Ed
Check 'every field changed in the editor' ($ed.name -eq 'Changed name' -and $ed.target -eq $beta.id -and $ed.kinds -join ',' -eq 'endsWith,glob' -and $ed.texts -join ',' -eq '_zz,*.zz') "name='$($ed.name)' kinds=$($ed.kinds -join ',') texts=$($ed.texts -join ',')"
Check 'while it is open, the other rules cannot be edited' ($ed.editOff -eq 2) "disabled edit buttons=$($ed.editOff)"
PageClick "$E [data-act=cancel]"
Start-Sleep -Seconds 1
$after = @(Rules | ForEach-Object { RuleSig $_ })
Check 'Cancel closes the editor' (-not (Ed).open)
Check 'Cancel: the rule list is unchanged' (($after -join '|') -eq ($sigs -join '|')) ((RuleSig (Rules)[0]))
Check 'Cancel: the row shows the old name' ((Row $prog.id).name -eq $prog.name) "row='$((Row $prog.id).name)'"
PageClick (EditSel $prog.id)
PageType "$E .r-name" 'Esc name'
[G]::Press(0x1B); Start-Sleep -Milliseconds 600
Check 'Esc closes the editor without saving' (-not (Ed).open -and ((@(Rules | ForEach-Object { RuleSig $_ })) -join '|') -eq ($sigs -join '|')) "row='$((Row $prog.id).name)'"

Case 'R3 new rule with three conditions'
PageClick '#nrPanel > summary'
PageType '#nrName' 'Triple'
PagePick '#nrTarget' $beta.id
PagePick '#nrKind' 'startsWith'
PageType '#nrValue' 'hr3'
PageClick '#nrAddCond'
PagePick '#nrConds .c-kind' 'ext' 1
PageType '#nrConds .c-text' 'txt' 1
PageClick '#nrAddCond'
PagePick '#nrConds .c-kind' 'contains' 2
PageType '#nrConds .c-text' 'keep' 2
PageClick '#nrAdd'
$t = PageToast
Check 'Add rule says Rule added' ($t.text -eq 'Rule added' -and -not $t.error) "toast='$($t.text)'"
Start-Sleep -Seconds 1
$tri = (Rules)[0]
Check 'the new rule is first, with its three conditions' ($tri.name -eq 'Triple' -and $tri.target.fence -eq $beta.id -and ($tri.allOf | ConvertTo-Json -Compress -Depth 5) -eq '[{"cond":"name","value":{"op":"startsWith","value":"hr3"}},{"cond":"ext","value":["txt"]},{"cond":"name","value":{"op":"contains","value":"keep"}}]' -and $tri.priorityClass -eq 'type') (RuleSig $tri)
Check 'the form is empty again' ((Js "[document.getElementById('nrName').value, document.getElementById('nrValue').value, document.querySelectorAll('#nrConds .cond').length].join('|')") -eq '||1')
foreach ($n in 'hr3-keep.txt', 'hr3-other.txt', 'hr3-keep.log', 'zz-keep.txt', 'zz-none.png') { DeskFile $n | Out-Null }
Check 'a file meeting all three lands in Beta' ((WaitItemFence 'hr3-keep.txt' 'Beta' 10) -eq 'Beta') "hr3-keep.txt in '$(ItemFence 'hr3-keep.txt')'"
$where = @('hr3-other.txt', 'hr3-keep.log', 'zz-keep.txt', 'zz-none.png' | ForEach-Object { "$_=$(ItemFence $_)" })
Check 'files meeting some or none do not' (-not ($where -match '=Beta$') -and -not ($where -match '=$')) ($where -join ' ')

Case 'R4 remove a condition'
PageClick (EditSel $tri.id)
$ed = Ed
Check 'the editor shows the three conditions, each with a cross' ($ed.kinds -join ',' -eq 'startsWith,ext,contains' -and $ed.dels -join ',' -eq 'False,False,False') "kinds=$($ed.kinds -join ',') hidden=$($ed.dels -join ',')"
PageClick "$E .c-del" 1
$ed = Ed
Check 'the cross removes the middle condition' ($ed.kinds -join ',' -eq 'startsWith,contains' -and $ed.texts -join ',' -eq 'hr3,keep') "kinds=$($ed.kinds -join ',') texts=$($ed.texts -join ',')"
PageClick "$E [data-act=save]"
$r = WaitRule $tri.id { param($r) @($r.allOf).Count -eq 2 }
Check 'saved with the two others, in order; class follows' (($r.allOf | ConvertTo-Json -Compress -Depth 5) -eq '[{"cond":"name","value":{"op":"startsWith","value":"hr3"}},{"cond":"name","value":{"op":"contains","value":"keep"}}]' -and $r.priorityClass -eq 'name') (RuleSig $r)
PageClick (EditSel $tri.id)
PageClick "$E .c-del" 0
$ed = Ed
$vis = Js "getComputedStyle(document.querySelector('$E .c-del')).visibility"
PageClick "$E .c-del" 0
$ed2 = Ed
Check 'the last condition has no cross; a click where it would be keeps it' ($ed.kinds -join ',' -eq 'contains' -and $ed.dels -join ',' -eq 'True' -and $vis -eq 'hidden' -and $ed2.kinds -join ',' -eq 'contains') "kinds=$($ed.kinds -join ',') hidden=$($ed.dels -join ',') visibility=$vis after click=$($ed2.kinds -join ',')"
PageClick "$E [data-act=cancel]"
Check 'Cancel keeps both conditions' (@((RuleById $tri.id).allOf).Count -eq 2)

Case 'R5 cleanup template'
$n0 = (Rules).Count
PageClick '#templates [data-template=cleanup]'
Start-Sleep -Seconds 2
$cl = Rules | Where-Object { $_.template -eq 'cleanup' } | Select-Object -First 1
Check 'the Cleanup chip adds the tagged rule with two conditions' ($cl -and @($cl.allOf).Count -eq 2) (RuleSig $cl)
JsWait "!!document.querySelector('#ruleList [data-row-id=`"$($cl.id)`"]')" 5 | Out-Null
PageClick (EditSel $cl.id)
$ed = Ed
Check 'the editor shows both conditions as fields' ($ed.kinds -join ',' -eq 'type,idle' -and $ed.cats[0] -eq 'archives,installers' -and $ed.days[1] -eq '30') "kinds=$($ed.kinds -join ',') cats=$($ed.cats -join ';') days=$($ed.days -join ',')"
PageType "$E .r-name" 'Old installers'
PageClick "$E [data-act=save]"
$r = WaitRule $cl.id { param($r) $r.name -eq 'Old installers' }
$types = (@($r.allOf[0].value) | Sort-Object) -join ','
Check 'name changed; conditions, tag, target and switch stay' ($r.name -eq 'Old installers' -and @($r.allOf).Count -eq 2 -and $r.allOf[0].cond -eq 'type' -and $types -eq 'archives,installers' -and $r.allOf[1].cond -eq 'idleDays' -and $r.allOf[1].value.min -eq 30 -and $r.template -eq 'cleanup' -and $r.target.fence -eq $cl.target.fence -and $r.enabled) "before=$($cl.allOf | ConvertTo-Json -Compress -Depth 5) after=$($r.allOf | ConvertTo-Json -Compress -Depth 5) template=$($r.template)"
Log "R5 note: type list order before=$($cl.allOf[0].value -join ',') after=$($r.allOf[0].value -join ',')"
PageClick '#templates [data-template=cleanup]'
JsWait "document.getElementById('toastText').textContent.includes('already exists')" 5 | Out-Null
$t = PageToast
$pg = Js "document.querySelector('nav a.sel').dataset.page"
Check 'the chip again: already exists, no second rule, its fence shown' ((Rules).Count -eq $n0 + 1 -and $t.text -like '*already exists*' -and $pg -eq 'fences') "rules=$((Rules).Count) toast='$($t.text)' page=$pg"
PageClick 'nav a[data-page=rules]'

Case 'R6 rule with CLI-only conditions'
Pf rule add --name CliRule --to Beta --name-is 'hr6 exact.txt' --name-contains ' (1)' --json '[{\"cond\":\"shortcutTarget\",\"value\":{\"op\":\"startsWith\",\"value\":\"steam://x\"}}]' | Out-Null
$cli = Rules | Where-Object { $_.name -eq 'CliRule' } | Select-Object -First 1
Check 'the CLI rule has three conditions' ($cli -and @($cli.allOf).Count -eq 3) (RuleSig $cli)
JsWait "!!document.querySelector('#ruleList [data-row-id=`"$($cli.id)`"]')" 5 | Out-Null
PageClick (EditSel $cli.id)
$ed = Ed
$fixedN = @($ed.fixed | Where-Object { $_ }).Count
Check 'CLI-only conditions show as text, the other as a field' ($fixedN -eq 2 -and ($ed.texts -contains '" (1)"')) "kinds=$($ed.kinds -join ',') texts=$($ed.texts -join '|') fixed=$($ed.fixed -join '|')"
PageType "$E .r-name" 'CLI rule edited'
PageClick "$E [data-act=save]"
$r = WaitRule $cli.id { param($r) $r.name -eq 'CLI rule edited' }
Check 'rule list: renamed, every condition kept as written' ($r.name -eq 'CLI rule edited' -and ($r.allOf | ConvertTo-Json -Compress -Depth 5) -eq ($cli.allOf | ConvertTo-Json -Compress -Depth 5)) (RuleSig $r)
$cfgPath = (PfJson status).configPath
$inCfg = $null
for ($k = 0; $k -lt 20 -and -not ($inCfg -and $inCfg.name -eq 'CLI rule edited'); $k++) {
  Start-Sleep -Milliseconds 500
  $inCfg = (Get-Content $cfgPath -Raw -Encoding UTF8 | ConvertFrom-Json).rules.list | Where-Object { $_.id -eq $cli.id }
}
Check 'config.json: renamed, the CLI conditions kept' ($inCfg.name -eq 'CLI rule edited' -and ($inCfg.allOf | ConvertTo-Json -Compress -Depth 5) -eq ($cli.allOf | ConvertTo-Json -Compress -Depth 5)) "config=$cfgPath $($inCfg.allOf | ConvertTo-Json -Compress -Depth 5)"

Case 'R7 editor survives a refresh and a language switch'
PageClick (EditSel $tri.id)
PageType "$E .r-name" 'Draft R7'
PageType "$E .c-text" 'hr7' 0
Pf fence create --title Gamma --rect '680,560,300,200' | Out-Null
Start-Sleep -Seconds 2
$ed = Ed
Check 'after a host refresh (a CLI fence): open, typed text kept, new fence offered' ($ed.open -and $ed.name -eq 'Draft R7' -and $ed.texts[0] -eq 'hr7' -and $ed.target -eq $beta.id -and $ed.targets -contains 'Gamma') "open=$($ed.open) name='$($ed.name)' texts=$($ed.texts -join ',') targets=$($ed.targets -join ',')"
$head = $ed.head
Pf settings set language ja | Out-Null
Start-Sleep -Seconds 2
$ed = Ed
Check 'after a language switch: open, captions changed, typed text kept' ($ed.open -and $ed.head -ne $head -and $ed.name -eq 'Draft R7' -and $ed.texts[0] -eq 'hr7' -and $ed.target -eq $beta.id) "head='$($ed.head)' save='$($ed.save)' name='$($ed.name)' texts=$($ed.texts -join ',')"
Shot 'r07-ja-editor'
Pf settings set language en | Out-Null
Start-Sleep -Seconds 2
$ed = Ed
Check 'back in English, still open with the draft' ($ed.open -and $ed.head -eq 'Edit rule' -and $ed.name -eq 'Draft R7') "head='$($ed.head)' name='$($ed.name)'"
PageClick "$E [data-act=save]"
$r = WaitRule $tri.id { param($r) $r.name -eq 'Draft R7' }
Check 'Save stores the draft' ($r.name -eq 'Draft R7' -and $r.allOf[0].value.value -eq 'hr7') (RuleSig $r)

Case 'R8 switch and order after an edit'
$fold = Rules | Where-Object { ($_.allOf | ConvertTo-Json -Depth 5 -Compress) -match 'folders' -and $_.name -notmatch 'edited' } | Select-Object -First 1
PageClick "#ruleList [data-row-id=`"$($prog.id)`"] .toggle"
$r = WaitRule $prog.id { param($r) -not $r.enabled }
Check 'a click on the switch turns the rule off' (-not $r.enabled) (RuleSig $r)
PageClick (EditSel $prog.id)
PageType "$E .r-name" 'Programs edited'
PageClick "$E [data-act=save]"
$r = WaitRule $prog.id { param($r) $r.name -eq 'Programs edited' }
Check 'the disabled rule, edited, stays off' ($r.name -eq 'Programs edited' -and -not $r.enabled -and (Row $prog.id).disabled) "enabled=$($r.enabled) row disabled=$((Row $prog.id).disabled)"
PageClick (EditSel $fold.id)
PageType "$E .r-name" 'Folders edited'
PageClick "$E [data-act=save]"
$r = WaitRule $fold.id { param($r) $r.name -eq 'Folders edited' }
Check 'an enabled rule, edited, stays on' ($r.name -eq 'Folders edited' -and $r.enabled) "enabled=$($r.enabled)"
$ids = @(Rules | ForEach-Object { $_.id }); $i0 = [array]::IndexOf($ids, $fold.id)
PageClick "#ruleList [data-row-id=`"$($fold.id)`"] [data-act=down]"
Start-Sleep -Milliseconds 600
$i1 = [array]::IndexOf(@(Rules | ForEach-Object { $_.id }), $fold.id)
PageClick "#ruleList [data-row-id=`"$($fold.id)`"] [data-act=up]"
Start-Sleep -Milliseconds 600
$i2 = [array]::IndexOf(@(Rules | ForEach-Object { $_.id }), $fold.id)
Check 'down and up move the edited rule' ($i1 -eq $i0 + 1 -and $i2 -eq $i0) "index $i0 -> $i1 -> $i2"
$n = (Rules).Count
PageClick "#ruleList [data-row-id=`"$($prog.id)`"] [data-act=del]"
Start-Sleep -Milliseconds 800
Check 'delete removes the edited rule' ((Rules).Count -eq $n - 1 -and -not (RuleById $prog.id)) "rules $n -> $((Rules).Count)"

Case 'R10 odd input'
Pf rule add --name Odd --to Beta --ext odd | Out-Null
$odd = Rules | Where-Object { $_.name -eq 'Odd' } | Select-Object -First 1
JsWait "!!document.querySelector('#ruleList [data-row-id=`"$($odd.id)`"]')" 5 | Out-Null
$others = @(Rules | Where-Object { $_.id -ne $odd.id } | ForEach-Object { RuleSig $_ })
# empty condition value
PageClick (EditSel $odd.id)
PagePick "$E .c-kind" 'contains'
PageType "$E .c-text" ''
PageClick "$E [data-act=save]"
$t = PageToast; $ed = Ed
Check 'empty value: refused with a message, editor stays, cursor in the box' ($t.error -and $t.text -eq 'Enter a keyword' -and $ed.open -and $ed.active -match 'c-text') "toast='$($t.text)' error=$($t.error) open=$($ed.open) active=$($ed.active)"
Check 'empty value: nothing saved' ((RuleSig (RuleById $odd.id)) -eq (RuleSig $odd))
[G]::Press(0x1B); Start-Sleep -Milliseconds 500
# empty name
PageClick (EditSel $odd.id)
PageType "$E .r-name" ''
PageClick "$E [data-act=save]"
$r = WaitRule $odd.id { param($r) $r.name -ne 'Odd' }
Check 'empty name: saved, named after its condition' ($r.name -and $r.name -ne 'Odd' -and (Row $odd.id).name -eq $r.name) "name='$($r.name)' row='$((Row $odd.id).name)'"
# very long values
$longName = 'N' * 300; $longVal = (('long value ' * 30).Trim())
PageClick (EditSel $odd.id)
PageType "$E .r-name" $longName
PagePick "$E .c-kind" 'contains'
PageType "$E .c-text" $longVal
PageClick "$E [data-act=save]"
$t = PageToast
$r = WaitRule $odd.id { param($r) $r.name.Length -gt 100 }
Check 'long name and value: saved whole (or refused clearly)' (($r.name -eq $longName -and $r.allOf[0].value.value -eq $longVal) -or ($t.error -and (Ed).open)) "nameLen=$($r.name.Length) valueLen=$($r.allOf[0].value.value.Length) toast='$($t.text)'"
$chk = Pf config check
Check 'config check after the long name' ($LASTEXITCODE -eq 0) "exit=$LASTEXITCODE $($chk.Trim())"
# quotes
PageClick (EditSel $odd.id)
PageType "$E .r-name" 'Odd quotes'
PageType "$E .c-text" '"my report"'
PageClick "$E .c-add"
PagePick "$E .c-kind" 'endsWith' 1
PageType "$E .c-text" "it's `"x`"" 1
PageClick "$E [data-act=save]"
$r = WaitRule $odd.id { param($r) $r.name -eq 'Odd quotes' }
$q1 = ($r.allOf | ConvertTo-Json -Compress -Depth 5)
Check 'quotes: a quoted value loses its quotes, inner quotes stay' ($r.allOf[0].value.op -eq 'contains' -and $r.allOf[0].value.value -eq 'my report' -and $r.allOf[1].value.op -eq 'endsWith' -and $r.allOf[1].value.value -eq "it's `"x`"") $q1
PageClick (EditSel $odd.id)
$ed = Ed
PageClick "$E [data-act=save]"
Start-Sleep -Seconds 1
Check 'quotes: reopened and saved unchanged, nothing changes' ($ed.texts[1] -eq "it's `"x`"" -and ((RuleById $odd.id).allOf | ConvertTo-Json -Compress -Depth 5) -eq $q1) "texts=$($ed.texts -join ' | ')"
# size
PageClick (EditSel $odd.id)
PagePick "$E .c-kind" 'size' 1
PageType "$E .c-min" '5' 1
PageType "$E .c-max" '1' 1
PageClick "$E [data-act=save]"
$t1 = PageToast
PageType "$E .c-min" 'abc' 1
PageClick "$E [data-act=save]"
$t2 = PageToast
Check 'size: max below min and a word are refused' ($t1.error -and $t2.error -and (Ed).open -and ((RuleById $odd.id).allOf | ConvertTo-Json -Compress -Depth 5) -eq $q1) "toasts='$($t1.text)' / '$($t2.text)'"
PageType "$E .c-min" '0.5' 1
PageType "$E .c-max" '10' 1
PageClick "$E [data-act=save]"
$r = WaitRule $odd.id { param($r) $r.allOf[1].cond -eq 'sizeMb' }
Check 'size: 0.5 to 10 MB saved' ($r.allOf[1].cond -eq 'sizeMb' -and $r.allOf[1].value.min -eq 0.5 -and $r.allOf[1].value.max -eq 10) ($r.allOf[1] | ConvertTo-Json -Compress -Depth 5)
# idle days
PageClick (EditSel $odd.id)
PagePick "$E .c-kind" 'idle' 1
PageType "$E .c-days" '0' 1
PageClick "$E [data-act=save]"
$t1 = PageToast
PageType "$E .c-days" '99999' 1
PageClick "$E [data-act=save]"
$t2 = PageToast
Check 'idle days 0 and 99999 are refused, saying the range' ($t1.error -and $t2.error -and $t2.text -like '*36500*') "toasts='$($t1.text)' / '$($t2.text)'"
PageType "$E .c-days" '45' 1
PageClick "$E [data-act=save]"
$r = WaitRule $odd.id { param($r) $r.allOf[1].cond -eq 'idleDays' }
Check 'idle days 45 saved' (($r.allOf[1] | ConvertTo-Json -Compress -Depth 5) -eq '{"cond":"idleDays","value":{"min":45}}') ($r.allOf[1] | ConvertTo-Json -Compress -Depth 5)
# creation time, typed into the time boxes (hour, minutes, AM/PM)
PageClick (EditSel $odd.id)
PagePick "$E .c-kind" 'time' 1
PageClick "$E .c-from" 1 12; [G]::Type('0900A'); Start-Sleep -Milliseconds 200
PageClick "$E .c-to" 1 12; [G]::Type('0530P'); Start-Sleep -Milliseconds 200
$tv = Js "[...document.querySelectorAll('$E .c-from, $E .c-to')].map(e => e.value).join('-')"
PageClick "$E [data-act=save]"
$r = WaitRule $odd.id { param($r) $r.allOf[1].cond -eq 'createdTime' }
Check 'creation time 09:00 to 17:30 typed and saved' (($r.allOf[1] | ConvertTo-Json -Compress -Depth 5) -eq '{"cond":"createdTime","value":{"from_min":540,"to_min":1050}}') "boxes=$tv saved=$($r.allOf[1] | ConvertTo-Json -Compress -Depth 5)"
# weekdays
PageClick (EditSel $odd.id)
PagePick "$E .c-kind" 'weekday' 1
PageClick "$E [data-act=save]"
$t1 = PageToast
PageClick "$(CondSel 1) .c-week label" 0
PageClick "$(CondSel 1) .c-week label" 4
PageClick "$E [data-act=save]"
$r = WaitRule $odd.id { param($r) $r.allOf[1].cond -eq 'createdWeekday' }
Check 'weekdays: none refused, Monday and Friday saved' ($t1.error -and ($r.allOf[1] | ConvertTo-Json -Compress -Depth 5) -eq '{"cond":"createdWeekday","value":[0,4]}') "toast='$($t1.text)' saved=$($r.allOf[1] | ConvertTo-Json -Compress -Depth 5)"
$others2 = @(Rules | Where-Object { $_.id -ne $odd.id } | ForEach-Object { RuleSig $_ })
Check 'odd input changed no other rule' (($others2 -join '|') -eq ($others -join '|')) "before=$($others.Count) after=$($others2.Count)"

Case 'R11 add, delete, apply'
PageClick '#nrPanel > summary'
if (-not (Js "document.getElementById('nrPanel').open")) { PageClick '#nrPanel > summary' }
PageType '#nrName' 'Markdown'
PagePick '#nrTarget' $beta.id
PagePick '#nrKind' 'ext'
PageType '#nrValue' 'md'
PageClick '#nrAdd'
Start-Sleep -Seconds 1
$md = (Rules)[0]
Check 'a one-condition rule is added first' ($md.name -eq 'Markdown' -and ($md.allOf | ConvertTo-Json -Compress -Depth 5) -eq '{"cond":"ext","value":["md"]}') (RuleSig $md)
DeskFile 'hr11.md' | Out-Null
Check 'a new .md file lands in Beta' ((WaitItemFence 'hr11.md' 'Beta' 10) -eq 'Beta') "hr11.md in '$(ItemFence 'hr11.md')'"
PageClick "#ruleList [data-row-id=`"$($md.id)`"] [data-act=del]"
Start-Sleep -Milliseconds 800
Check 'its delete button removes it' (-not (RuleById $md.id))
PageClick '#applyRules'
$t = PageToast
Check 'Apply now sends the file back to the desktop fence' ((WaitItemFence 'hr11.md' $inbox.title 8) -eq $inbox.title) "hr11.md in '$(ItemFence 'hr11.md')' toast='$($t.text)'"

Case 'R9 restart keeps the edits'
$final = @(Rules | ForEach-Object { RuleSig $_ })
$names = @(Rules | ForEach-Object { $_.name })
$saved = @(PfLogAll | Where-Object { $_ -match 'rules saved from page rules=\d+' })
$bad = @(PfLogAll | Where-Object { $_ -match 'bad rules from page' })
Check 'the log has rules saved from page lines and no bad rules' ($saved.Count -ge 5 -and $bad.Count -eq 0) "saved=$($saved.Count) bad=$($bad.Count) last: $($saved | Select-Object -Last 1)"
PfQuit
PfStart
$again = @(Rules | ForEach-Object { RuleSig $_ })
Check 'after a restart the rule list is the same' (($again -join '|') -eq ($final -join '|')) "before=$($final.Count) after=$($again.Count)"
$opened = PfSettings
PageClick 'nav a[data-page=rules]'
$pageNames = @(Js $RowsJs | ForEach-Object { $_.name })
Check 'and the page lists them' (($pageNames -join '|') -eq ($names -join '|')) ($pageNames -join ' | ')
$errs = @(PfLogAll | Where-Object { $_ -match ' ERROR |panicked' })
Check 'no ERROR or panic line before the restart either' ($errs.Count -eq 0) (($errs | Select-Object -First 2) -join ' | ')
Shot 'r09-after-restart'

Case 'cleanup'
PfQuit
CleanDesk
