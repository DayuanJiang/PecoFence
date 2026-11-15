# 网页层 sample: the test web plugin (assets/webdemo) in a fence. Part A (debug build, CDP on the
# page): the page loads in the fence's visual tree, the mouse, wheel and keyboard reach it, pf.time.now
# answers, and what the plugin must not do is refused. Part B (release build): screenshots
# next to a normal fence, clips of roll-up, fade, move and resize. Part M (release build): the
# memory of PecoFence's process group with no web plugin, one, two, and after 60 s hidden.
# Part I (debug build): Chinese typing with Microsoft Pinyin in the page. Part C (debug build): the
# page's pf calls, main.js beside the page, the theme, the right-click menu, crashes and
# 重新加载, a background plugin, the profile deleted at the uninstall. Part N: no WebView2.
# run.sh --var Part=A|B|M|I|C|N (default A).
if (-not $Part) { $Part = 'A' }
$Demo = 'C:\pf\kit\assets\webdemo'

# Installs the test plugin as `id` (a copy of webdemo under another id for the second plugin),
# as an install does: in its version's folder, which current.txt names.
function InstallDemo([string]$id = 'webdemo') {
  $to = "$script:Config\plugins\$id\1.0.0"
  New-Item -ItemType Directory -Force $to | Out-Null
  Copy-Item "$Demo\*" $to -Force
  Set-Content "$script:Config\plugins\$id\current.txt" '1.0.0' -Encoding ASCII -NoNewline
  if ($id -ne 'webdemo') { (Get-Content "$to\plugin.json" -Raw) -replace '"webdemo"', "`"$id`"" | Set-Content "$to\plugin.json" -Encoding UTF8 }
}
# A script fence at `rect` (x,y,w,h px) switched to plugin `id`; its fence id.
function NewWebFence([string]$title, [string]$rect, [string]$id = 'webdemo') {
  Set-Content 'C:\pf\run\idle.cmd' '@echo off' -Encoding ASCII
  $f = PfJson fence create --title $title --script 'C:\pf\run\idle.cmd' --rect $rect
  $fid = if ($f.fence) { $f.fence.id } else { $f.id }
  Pf fence set $fid scriptPath "plugin:$id" | Out-Null
  return $fid
}
function FenceWin([string]$title) { return @([W]::Tops('PecoFence.Fence') | Where-Object { [L]::Text($_) -eq $title }) | Select-Object -First 1 }
# Screen rect of an element of the plugin page in fence window $h: the page fills the window
# below the title row (a fence has no frame outside its client area), so it ends at its bottom right.
function WebRect($h, [string]$sel) {
  $q = $sel | ConvertTo-Json
  $r = Js "(() => { const e = document.querySelector($q); if (!e) return null; const b = e.getBoundingClientRect(), d = devicePixelRatio; return { x: b.left * d, y: b.top * d, w: b.width * d, h: b.height * d, pw: innerWidth * d, ph: innerHeight * d }; })()"
  if (-not $r) { return $null }
  $w = Rect $h
  $px = $w.R - $r.pw; $py = $w.B - $r.ph
  return @{ cx = [int]($px + $r.x + $r.w / 2); cy = [int]($py + $r.y + $r.h / 2); x = [int]($px + $r.x); y = [int]($py + $r.y); w = [int]$r.w; h = [int]$r.h }
}
# PecoFence and every process it started (the WebView2 browser and its children): private
# working set and private commit in MB, and the number of msedgewebview2 processes.
function GroupMem() {
  $pf = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
  if (-not $pf) { return $null }
  $all = @(Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId, Name)
  $ids = @([int]$pf.Id); $i = 0
  while ($i -lt $ids.Count) { $ids += @($all | Where-Object { $_.ParentProcessId -eq $ids[$i] -and $_.Name -ne 'pecofence-watchdog.exe' } | ForEach-Object { [int]$_.ProcessId }); $i++ }
  $ws = 0.0
  $s = (Get-Counter '\Process(*)\ID Process', '\Process(*)\Working Set - Private' -ErrorAction SilentlyContinue).CounterSamples
  $byInst = @{}
  foreach ($x in $s) {
    $inst = $x.Path.Split('(')[1].Split(')')[0]
    if (-not $byInst[$inst]) { $byInst[$inst] = @{} }
    if ($x.Path -like '*\id process') { $byInst[$inst].id = [int]$x.CookedValue } else { $byInst[$inst].ws = $x.CookedValue }
  }
  foreach ($v in $byInst.Values) { if ($ids -contains $v.id) { $ws += $v.ws } }
  $commit = 0.0; $web = 0
  foreach ($id in $ids) { $p = Get-Process -Id $id -ErrorAction SilentlyContinue; if ($p) { $commit += $p.PrivateMemorySize64; if ($p.Name -eq 'msedgewebview2') { $web++ } } }
  return @{ ws = [Math]::Round($ws / 1MB, 1); commit = [Math]::Round($commit / 1MB, 1); procs = $ids.Count; web = $web }
}
# The median of three readings 5 s apart, logged and kept for the table.
function MemNow([string]$what) {
  $r = @(1..3 | ForEach-Object { GroupMem; Start-Sleep -Seconds 5 })
  $ws = ($r | ForEach-Object { $_.ws } | Sort-Object)[1]; $cm = ($r | ForEach-Object { $_.commit } | Sort-Object)[1]
  $line = "$what`t$ws`t$cm`t$($r[1].procs)`t$($r[1].web)"
  Add-Content "$PfShots\memory.tsv" $line
  Log "memory $line"
  return @{ ws = $ws; commit = $cm; web = $r[1].web }
}

Case 'setup'
CloseWindows
if ($Part -eq 'I') {
  # Microsoft Pinyin for pf, before PecoFence starts (taken away again at the end).
  $langs = Get-WinUserLanguageList
  $script:AddedZh = -not ($langs | Where-Object { $_.LanguageTag -like 'zh-Hans*' })
  if ($AddedZh) { $langs.Add('zh-Hans-CN'); Set-WinUserLanguageList $langs -Force }
}
if ($Part -in 'A', 'I', 'C' -and $PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
if ($Part -eq 'N') {
  # WebView2 is missing: the loader looks for the runtime in an empty folder.
  New-Item -ItemType Directory -Force 'C:\pf\run\no-webview2' | Out-Null
  $env:WEBVIEW2_BROWSER_EXECUTABLE_FOLDER = 'C:\pf\run\no-webview2'
}
# The test plugins go in before the start: PecoFence reads the installed plugins once as it starts.
$script:Config = "$PfApp\config"
InstallDemo
if ($Part -eq 'M') { InstallDemo 'webdemo2' }
if ($Part -eq 'C') {
  InstallDemo 'webbg'
  $j = "$script:Config\plugins\webbg\1.0.0\plugin.json"
  (Get-Content $j -Raw) -replace '"network": \["example.org"\]', '"network": ["example.org"], "background": true' | Set-Content $j -Encoding UTF8
}
PfStart -Lang en
Check 'the portable configuration is where the plugins went' ((Split-Path (PfJson status).configPath) -eq $script:Config) (PfJson status).configPath
Check 'the test plugin is installed' (Test-Path "$script:Config\plugins\webdemo\1.0.0\index.html") $script:Config

if ($Part -eq 'A') {
  Case 'page in the fence'
  $fid = NewWebFence 'Web' '420,160,380,460'
  $loaded = PfWaitLog 'web plugin: page loading' 30
  Check 'the browser is made and the page loads' ($null -ne $loaded) "$loaded"
  $h = FenceWin 'Web'
  $reached = CdpOpen -Url 'https://webdemo.plugins.pecofence/*' -sec 30
  Check 'CDP reaches the plugin page' $reached ''
  $ok = JsWait "document.readyState === 'complete' && document.getElementById('list').children.length === 40" 15
  Check 'the page ran its script (40 list rows)' $ok ''
  Check 'pf.time.now is there' ((Js "typeof (window.pf && pf.time.now)") -eq 'function') ''
  Check 'the page background is transparent' ((Js "getComputedStyle(document.body).backgroundColor") -eq 'rgba(0, 0, 0, 0)') (Js "getComputedStyle(document.body).backgroundColor")
  Shot 'A1-page'

  Case 'mouse'
  $b = WebRect $h '#btn'
  [G]::Click($b.cx, $b.cy); Start-Sleep -Milliseconds 800
  $d = Js "window.demo"
  Check 'a click on the button reaches the page' ($d.clicks -eq 1) "clicks=$($d.clicks)"
  Check 'the click got the time from pf.time.now' ($d.now -gt 1700000000000) "now=$($d.now)"
  $l = WebRect $h '#list'
  Wheel $l.cx $l.cy -3; Start-Sleep -Milliseconds 600
  $top = Js "document.getElementById('list').scrollTop"
  Check 'the wheel scrolls the list' ($top -gt 0) "scrollTop=$top"
  [G]::MoveTo($b.cx, $b.cy); Start-Sleep -Milliseconds 500
  $hov = Js "document.querySelector('#btn:hover') !== null"
  Check 'the page sees the pointer over the button' $hov ''
  Shot 'A2-hover'

  Case 'keyboard'
  $t = WebRect $h '#box'
  [G]::Click($t.cx, $t.cy); Start-Sleep -Milliseconds 600
  Check 'the text box has the focus after a click' (Js "document.activeElement && document.activeElement.id === 'box'") (Fg)
  [G]::Type('hello web'); Start-Sleep -Milliseconds 600
  $v = Js "document.getElementById('box').value"
  Check 'English typing reaches the text box' ($v -eq 'hello web') "value='$v' fg=$(Fg)"
  [G]::Press(0x08); [G]::Press(0x08); Start-Sleep -Milliseconds 300
  $v = Js "document.getElementById('box').value"
  Check 'Backspace works in the page' ($v -eq 'hello w') "value='$v'"
  Shot 'A3-typed'
  DeskClick; Start-Sleep -Milliseconds 600
  [G]::Type('zz'); Start-Sleep -Milliseconds 400
  $v = Js "document.getElementById('box').value"
  Check 'after a click on the desktop the page no longer gets keys' ($v -eq 'hello w') "value='$v' fg=$(Fg)"

  Case 'refused'
  Js "(window.selfResult = null, selfTest().then(r => window.selfResult = r), 1)" | Out-Null
  $r = JsWait "window.selfResult" 40
  Log ("self test: " + ($r | ConvertTo-Json -Compress -Depth 4))
  Check 'fetch to an undeclared host is refused' ("$($r.fetchUndeclared)" -like 'refused*') "$($r.fetchUndeclared)"
  Check 'fetch to the declared host is allowed by the CSP' ("$($r.fetchDeclared)" -eq 'loaded' -or -not (@($r.violations) -like '*example.org*')) "$($r.fetchDeclared)"
  Check 'a WebSocket to an undeclared host fails' ("$($r.webSocket)" -ne 'open') "$($r.webSocket)"
  Check 'window.open gives nothing' ("$($r.windowOpen)" -eq 'null') "$($r.windowOpen)"
  Check 'geolocation is denied' ("$($r.geolocation)" -like 'denied*') "$($r.geolocation)"
  Check 'notifications are denied' ("$($r.notification)" -eq 'denied') "$($r.notification)"
  Check 'reading the clipboard is refused' ("$($r.clipboardRead)" -like 'refused*') "$($r.clipboardRead)"
  Check 'eval is refused by the CSP' ("$($r.eval)" -like 'refused*') "$($r.eval)"
  Check 'WebRTC is not there or finds no candidates' ("$($r.webrtc)" -like 'candidates: none*' -or "$($r.webrtc)" -like 'threw*') "$($r.webrtc)"
  Check 'a new frame of the page has no WebRTC either' ("$($r.webrtcInFrame)" -ne 'function') "$($r.webrtcInFrame)"
  Check 'the CSP reported the blocked connections' ((@($r.violations) -join ' ') -match 'connect-src') (@($r.violations) -join ' | ')
  Check 'the host refused the new window' ($null -ne (PfWaitLog 'web plugin: new window refused' 5)) ''
  Check 'the host refused permissions' ($null -ne (PfWaitLog 'web plugin: permission refused' 5)) ''
  Check 'the host refused the download' ($null -ne (PfWaitLog 'web plugin: download refused' 5)) ''
  Js "(tryNavigate(), 1)" | Out-Null; Start-Sleep -Seconds 2
  $reached = CdpOpen -Url 'https://webdemo.plugins.pecofence/*' -sec 5
  $href = if ($reached) { Js "location.href" } else { '' }
  Check 'navigation to another site is refused, the page stays' ($href -like 'https://webdemo.plugins.pecofence/*' -and $null -ne (PfWaitLog 'navigation to another site refused' 5)) "href=$href"
  Check 'the page still answers after the refusals' ((Js "window.demo.clicks") -eq 1) ''

  Case 'hidden'
  # A DevTools session keeps the page from being suspended: let go of it while hidden.
  $script:Cdp.Dispose(); $script:Cdp = $null
  Pf fence roll $fid | Out-Null
  $hid = PfWaitLog 'web plugin: page shown or hidden.*shown=false' 8
  Check 'rolling up hides the page' ($null -ne $hid) "$hid"
  $sus = PfWaitLog 'web plugin: hidden page suspended' 45
  Check 'the hidden page is suspended after 30 s' ($sus -match 'Ok\(true\)') "$sus"
  Pf fence unroll $fid | Out-Null
  $shown = PfWaitLog 'web plugin: page shown or hidden.*shown=true' 8
  Check 'expanding shows the page again' ($null -ne $shown) "$shown"
  Start-Sleep -Seconds 2
  $reached = CdpOpen -Url 'https://webdemo.plugins.pecofence/*' -sec 10
  $b = WebRect $h '#btn'
  [G]::Click($b.cx, $b.cy); Start-Sleep -Milliseconds 800
  Check 'the page works again after the suspend' ((Js "window.demo.clicks") -eq 2) "clicks=$(Js 'window.demo.clicks')"
  Pf windows-desktop start | Out-Null
  $hid = PfWaitLog 'web plugin: page shown or hidden.*shown=false' 8
  Check 'hiding the fences hides the page' ($null -ne $hid) ''
  Pf windows-desktop end | Out-Null; Start-Sleep -Seconds 2
  Shot 'A4-back'
  Check 'no errors in the log' (@(PfLogErrors).Count -eq 0) ((PfLogErrors) -join ' | ')
}

if ($Part -eq 'B') {
  . C:\pf\kit\rec.ps1
  Case 'looks'
  ClearFences
  Pf fence create --title 'Normal' --rect '120,150,380,460' | Out-Null
  $fid = NewWebFence 'Web' '540,150,380,460'
  Check 'the page loads' ($null -ne (PfWaitLog 'web plugin: page loading' 30)) ''
  Check 'the mouse goes through the browser window' ($null -ne (PfWaitLog 'goes through the browser' 10)) ''
  Start-Sleep -Seconds 3; ParkMouse
  Shot 'B1-side-by-side'
  RecStart 'B2-roll' -X 100 -Y 110 -W 840 -H 520 -Fps 20
  RecMark '收起'; Pf fence roll --all | Out-Null; Start-Sleep -Milliseconds 1500
  RecMark '展开'; Pf fence unroll --all | Out-Null; Start-Sleep -Milliseconds 1500
  RecStop
  RecStart 'B3-fade' -X 100 -Y 110 -W 840 -H 520 -Fps 20
  RecMark '隐藏栅栏'; Pf windows-desktop start | Out-Null; Start-Sleep -Milliseconds 1500
  RecMark '显示栅栏'; Pf windows-desktop end | Out-Null; Start-Sleep -Milliseconds 1800
  RecStop
  $h = FenceWin 'Web'; $r = Rect $h
  RecStart 'B4-move' -X 100 -Y 110 -W 1100 -H 700 -Fps 20
  RecMark '拖动标题'; [G]::MoveTo($r.L + 200, $r.T + 14); Start-Sleep -Milliseconds 200; MDown
  Glide ($r.L + 200) ($r.T + 14) ($r.L + 420) ($r.T + 200) 24 30; Start-Sleep -Milliseconds 300; MUp
  Start-Sleep -Milliseconds 800; RecStop
  $r = Rect $h
  Check 'the web fence moved' ($r.L -gt 700) "$($r.L),$($r.T)"
  RecStart 'B5-resize' -X 100 -Y 110 -W 1300 -H 780 -Fps 20
  RecMark '拖动右下角'; [G]::MoveTo($r.R - 2, $r.B - 2); Start-Sleep -Milliseconds 300; MDown
  Glide ($r.R - 2) ($r.B - 2) ($r.R + 140) ($r.B + 90) 24 30; Start-Sleep -Milliseconds 300; MUp
  Start-Sleep -Milliseconds 800; RecStop
  $r2 = Rect $h
  Check 'the web fence was resized from its corner over the page' (($r2.R - $r2.L) -gt ($r.R - $r.L) + 60) "$($r.R - $r.L)x$($r.B - $r.T) -> $($r2.R - $r2.L)x$($r2.B - $r2.T)"
  Shot 'B6-after-resize'
  Check 'no errors in the log' (@(PfLogErrors).Count -eq 0) ((PfLogErrors) -join ' | ')
}

if ($Part -eq 'M') {
  Case 'memory'
  Start-Sleep -Seconds 20
  $m0 = MemNow 'no web plugin'
  $f1 = NewWebFence 'Web 1' '420,160,380,460'
  Check 'the first page loads' ($null -ne (PfWaitLog 'web plugin: page loading' 30)) ''
  Start-Sleep -Seconds 15
  $m1 = MemNow 'one web plugin'
  $f2 = NewWebFence 'Web 2' '820,160,380,460' 'webdemo2'
  Check 'the second page loads' ($null -ne (PfWaitLog 'page loading plugin=webdemo2' 30)) ''
  Start-Sleep -Seconds 15
  $m2 = MemNow 'two web plugins'
  Shot 'M1-two'
  Pf fence roll --all | Out-Null
  Start-Sleep -Seconds 62
  $m3 = MemNow 'two, hidden 60 s'
  $sus = @(PfLogAll | Where-Object { $_ -match 'hidden page suspended.*Ok\(true\)' }).Count
  Check 'both hidden pages are suspended' ($sus -ge 2) "suspended=$sus"
  Pf fence unroll --all | Out-Null
  Start-Sleep -Seconds 10
  $m4 = MemNow 'two, shown again'
  Check 'memory readings taken' ($m0 -and $m1 -and $m2 -and $m3 -and $m4) ((Get-Content "$PfShots\memory.tsv") -join ' / ')
}

if ($Part -eq 'I') {
  Add-Type @"
using System; using System.Runtime.InteropServices;
public static class WebIme {
  [DllImport("user32.dll")] static extern IntPtr LoadKeyboardLayoutW(string id, uint flags);
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern bool PostMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  // Asks the foreground window to switch to Chinese (PRC) with its default IME, Microsoft Pinyin.
  public static void Chinese() { var hkl = LoadKeyboardLayoutW("00000804", 1); PostMessageW(GetForegroundWindow(), 0x50, IntPtr.Zero, hkl); }
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, IntPtr pid);
  [DllImport("user32.dll")] static extern IntPtr GetKeyboardLayout(uint thread);
  public static string Layout() { var t = GetWindowThreadProcessId(GetForegroundWindow(), IntPtr.Zero); return GetKeyboardLayout(t).ToInt64().ToString("X"); }
  public static void English() { var hkl = LoadKeyboardLayoutW("00000409", 1); PostMessageW(GetForegroundWindow(), 0x50, IntPtr.Zero, hkl); }
}
"@
  Case 'chinese input'
  $fid = NewWebFence 'Web' '420,160,380,460'
  Check 'the page loads' ($null -ne (PfWaitLog 'web plugin: page loading' 30)) ''
  $h = FenceWin 'Web'
  $reached = CdpOpen -Url 'https://webdemo.plugins.pecofence/*' -sec 30
  JsWait "document.getElementById('list').children.length === 40" 15 | Out-Null
  Start-Sleep -Seconds 2
  $t = WebRect $h '#box'
  [G]::Click($t.cx, $t.cy); Start-Sleep -Milliseconds 800
  [WebIme]::Chinese(); Start-Sleep -Milliseconds 1500
  Log "layout after the request: $([WebIme]::Layout())"
  if ([WebIme]::Layout() -notlike '*0804') { Chord 0x5B 0x20; Start-Sleep -Milliseconds 1500; Log "layout after Win+Space: $([WebIme]::Layout())" }
  Check 'the fence thread has the Chinese layout' ([WebIme]::Layout() -like '*0804') ([WebIme]::Layout())
  foreach ($c in 'NIHAO'.ToCharArray()) { [G]::Press([uint16][int]$c); Start-Sleep -Milliseconds 150 }
  Start-Sleep -Milliseconds 1200
  Shot 'I1-composing'
  [G]::Press(0x20); Start-Sleep -Milliseconds 1000
  $v = Js "document.getElementById('box').value"
  Check 'Chinese typed with Microsoft Pinyin lands in the text box' ($v -match '[一-鿿]') "value='$v' fg=$(Fg)"
  foreach ($c in 'SHIJIE'.ToCharArray()) { [G]::Press([uint16][int]$c); Start-Sleep -Milliseconds 150 }
  Start-Sleep -Milliseconds 1200
  Shot 'I2-composing-after-text'
  [G]::Press(0x20); Start-Sleep -Milliseconds 1000
  $v = Js "document.getElementById('box').value"
  Log "box: $v"
  Check 'a second word follows the first' ($v.Length -ge 4) "value='$v'"
  [WebIme]::English(); Start-Sleep -Milliseconds 800
  Shot 'I3-typed'
  if ($AddedZh) { $langs = Get-WinUserLanguageList; $zh = $langs | Where-Object { $_.LanguageTag -like 'zh-Hans*' }; foreach ($z in @($zh)) { [void]$langs.Remove($z) }; Set-WinUserLanguageList $langs -Force }
}

if ($Part -eq 'C') {
  # The renderer processes of PecoFence's WebView2 browser (the plugin pages; the settings window
  # is closed here).
  function Renderers() {
    $pf = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
    $all = @(Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId, Name, CommandLine)
    $ids = @([int]$pf.Id); $i = 0
    while ($i -lt $ids.Count) { $ids += @($all | Where-Object { $_.ParentProcessId -eq $ids[$i] } | ForEach-Object { [int]$_.ProcessId }); $i++ }
    return @($all | Where-Object { $ids -contains $_.ProcessId -and $_.Name -eq 'msedgewebview2.exe' -and $_.CommandLine -match '--type=renderer' } | ForEach-Object { [int]$_.ProcessId })
  }
  # Lines of this run's own log (the shareable copy has them too).
  function LogCount([string]$regex) { return @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match $regex }).Count }
  function WaitCount([string]$regex, [int]$n, $sec = 20) {
    for ($i = 0; $i -lt $sec * 4; $i++) { if ((LogCount $regex) -ge $n) { return $true }; Start-Sleep -Milliseconds 250 }
    return $false
  }

  Case 'pf calls from the page'
  $fid = NewWebFence 'Web' '420,160,380,460'
  Check 'the page loads' ($null -ne (PfWaitLog 'web plugin: page loading' 30)) ''
  $h = FenceWin 'Web'
  $reached = CdpOpen -Url 'https://webdemo.plugins.pecofence/*' -sec 30
  $ok = JsWait "document.readyState === 'complete' && document.getElementById('list').children.length === 40" 15
  Check 'CDP reaches the page and its script ran' ($reached -and $ok) ''
  Js "(window.callResult = null, callTest().then(r => window.callResult = r), 1)" | Out-Null
  $r = JsWait "window.callResult" 45
  Log ("calls: " + ($r | ConvertTo-Json -Compress -Depth 5))
  Check 'pf.time.now answers the time' ($r.now.ok -gt 1700000000000) "$($r.now | ConvertTo-Json -Compress)"
  Check 'pf.http.get to the declared host answers 200' ($r.declared.ok.status -eq 200) "$($r.declared.ok.status) $($r.declared.refused)"
  Check 'pf.http.get to an undeclared host is refused' ([bool]$r.undeclared.refused) "$($r.undeclared | ConvertTo-Json -Compress)"
  Check 'pf.clipboard.text is refused without the clipboard permission' ([bool]$r.clipboard.refused) "$($r.clipboard | ConvertTo-Json -Compress)"
  Check 'pf.open with no click just before is refused' ("$($r.openNoClick.refused)" -match 'click') "$($r.openNoClick | ConvertTo-Json -Compress)"
  Check 'the page fetches its own file (connect-src self)' ($r.ownFile -eq 'webdemo') "$($r.ownFile)"
  Check 'pf tells the theme, the language and the fence' ($r.theme -eq 'light' -and $r.lang -eq 'en' -and $r.fence.width -gt 100 -and $r.fence.id -eq $fid) "$($r.theme) $($r.lang) $($r.fence | ConvertTo-Json -Compress)"
  Check 'the page cannot change pf' ($r.frozen -eq $true) ''
  $o = WebRect $h '#open'
  [G]::Click($o.cx, $o.cy); Start-Sleep -Milliseconds 1500
  $op = Js "window.demo.open"
  Check 'after a click pf.open passes the click rule (the host rule then refuses example.com)' ("$op" -ne '' -and "$op" -notmatch 'click') "$op"

  Case 'main.js and the page'
  Check 'the web plugin main.js runs beside the page' ($null -ne (PfWaitLog 'plugin fence starts its main.js' 5)) ''
  Js "(pf.post({ n: 41 }), 1)" | Out-Null
  $m = JsWait "window.demo.messages.length > 0 && window.demo.messages[0]" 10
  Check "the page's message reaches main.js and its answer comes back" ($m.from -eq 'main.js' -and $m.echo.n -eq 41) "$($m | ConvertTo-Json -Compress)"

  # The page's theme is what the fence draws: dark text (a light page) on the box's bright
  # wallpaper even in the dark theme, light text (a dark page) on a dark wallpaper.
  Case 'theme: what the fence draws'
  $IsDark = "matchMedia('(prefers-color-scheme: dark)').matches"
  Check 'light theme, bright wallpaper: the page is light' ((Js "!$IsDark && pf.theme === 'light'") -eq $true) ''
  Shot 'C1-light'
  Pf settings set theme dark | Out-Null; Start-Sleep -Seconds 3
  Check 'dark theme, bright wallpaper: the fence keeps dark text, the page stays light' ((Js "!$IsDark && pf.theme === 'light'") -eq $true) (Js "window.demo.events.join(',')")
  ParkMouse; Shot 'C1-dark-bright-wallpaper'
  $b = New-Object Drawing.Bitmap 32, 32; $g = [Drawing.Graphics]::FromImage($b); $g.Clear([Drawing.Color]::FromArgb(20, 20, 24)); $g.Dispose()
  $b.Save("$PfRun\dark.bmp", [Drawing.Imaging.ImageFormat]::Bmp); $b.Dispose()
  [G]::SystemParametersInfoW(0x14, 0, "$PfRun\dark.bmp", 3) | Out-Null
  $d = JsWait "$IsDark && pf.theme === 'dark'" 20
  Check 'dark theme, dark wallpaper: the page sees prefers-color-scheme dark and pf.theme dark' ($d -eq $true) ''
  Check 'the page got the theme event' ((Js "window.demo.events.includes('theme:dark')") -eq $true) (Js "window.demo.events.join(',')")
  Start-Sleep -Milliseconds 800; ParkMouse
  Shot 'C1-dark'
  $wp = @('C:\Windows\Web\Wallpaper\Windows\img0.jpg', 'C:\Windows\Web\Screen\img100.jpg') | Where-Object { Test-Path $_ } | Select-Object -First 1
  if ($wp) { [G]::SystemParametersInfoW(0x14, 0, $wp, 3) | Out-Null }
  Pf settings set theme light | Out-Null
  $l = JsWait "!$IsDark && pf.theme === 'light'" 20
  Check 'back to light' ($l -eq $true) ''

  Case 'right-click'
  $l = WebRect $h '#list'
  [G]::RClick($l.cx, $l.cy)
  $m = MenuWin 3
  $t = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
  Shot 'C2-menu'
  Check 'a right-click on the page opens the fence menu' ($m -ne [IntPtr]::Zero -and $t -match 'Rename') $t
  CloseMenus; Start-Sleep -Milliseconds 500
  $n = WebRect $h '#nomenu'
  [G]::RClick($n.cx, $n.cy)
  $m = MenuWin 2
  Check 'where the page calls preventDefault there is no menu' ($m -eq [IntPtr]::Zero -and (Js "window.demo.nomenu") -eq 1) "menu=$m nomenu=$(Js 'window.demo.nomenu')"
  CloseMenus

  Case 'show and hide events'
  Pf fence roll $fid | Out-Null; Start-Sleep -Seconds 1
  Pf fence unroll $fid | Out-Null; Start-Sleep -Seconds 2
  $ev = Js "window.demo.events.join(',')"
  Check 'the page got hide and show' ($ev -match 'hide' -and $ev -match 'show') "$ev"

  Case 'a page that crashes'
  $script:Cdp.Dispose(); $script:Cdp = $null
  $loads = LogCount 'web plugin: page loading'
  $k = Renderers; Log "renderers: $($k -join ',')"
  $k | ForEach-Object { Stop-Process -Id $_ -Force -ErrorAction SilentlyContinue }
  Check 'the first crash is seen' (WaitCount 'page failed; made anew.*count=1 again=true' 1 15) ''
  Check 'the page is made anew by itself' (WaitCount 'web plugin: page loading' ($loads + 1) 30) ''
  Start-Sleep -Seconds 3
  $k = Renderers; Log "renderers: $($k -join ',')"
  $k | ForEach-Object { Stop-Process -Id $_ -Force -ErrorAction SilentlyContinue }
  Check 'the second crash: made anew again' ((WaitCount 'page failed; made anew.*count=2 again=true' 1 15) -and (WaitCount 'web plugin: page loading' ($loads + 2) 30)) ''
  Start-Sleep -Seconds 3
  $k = Renderers; Log "renderers: $($k -join ',')"
  $k | ForEach-Object { Stop-Process -Id $_ -Force -ErrorAction SilentlyContinue }
  Check 'the third crash within the hour stays stopped' (WaitCount 'page failed; made anew.*count=3 again=false' 1 15) ''
  Start-Sleep -Seconds 2
  $out = (PfJson fence get $fid).script.output
  Check 'the fence says why' ("$out" -match 'stopped three times' -and "$out" -match "process ended") "$out"
  Check 'no page is made by itself after the third' ((LogCount 'web plugin: page loading') -eq $loads + 2) ''
  $r = Rect $h
  ParkMouse; Shot 'C3-stopped'
  $clicked = OcrClick 'Reload' $r.L $r.T ($r.R - $r.L) ($r.B - $r.T)
  Check '重新加载 is there and starts the page again' ($clicked -and (WaitCount 'web plugin: page loading' ($loads + 3) 30)) ''
  $reached = CdpOpen -Url 'https://webdemo.plugins.pecofence/*' -sec 20
  $ok = JsWait "document.getElementById('list').children.length === 40" 15
  Check 'the page works again' ($reached -and $ok) ''

  Case 'a background plugin is not suspended'
  $fb = NewWebFence 'Bg' '820,160,380,460' 'webbg'
  Check 'the background plugin page loads' ($null -ne (PfWaitLog 'page loading plugin=webbg.*background=true' 30)) ''
  $reached = CdpOpen -Url 'https://webbg.plugins.pecofence/*' -sec 30
  JsWait "document.getElementById('list').children.length === 40" 15 | Out-Null
  $t0 = Js "window.demo.ticks"
  $script:Cdp.Dispose(); $script:Cdp = $null
  Pf fence roll --all | Out-Null
  Check 'the background page stays running when hidden' ($null -ne (PfWaitLog 'runs on in the background plugin=webbg' 8)) ''
  Start-Sleep -Seconds 38
  $sus = @(PfLogAll | Where-Object { $_ -match 'hidden page suspended' })
  Check 'the other page is suspended, the background one is not' (@($sus | Where-Object { $_ -match 'plugin=webdemo' }).Count -ge 1 -and @($sus | Where-Object { $_ -match 'plugin=webbg' }).Count -eq 0) ($sus -join ' | ')
  $reached = CdpOpen -Url 'https://webbg.plugins.pecofence/*' -sec 10
  $t1 = Js "window.demo.ticks"
  Check 'its timers ran on while hidden' ($t1 - $t0 -ge 35) "ticks $t0 -> $t1"
  Check 'it got the hide event' ((Js "window.demo.events.includes('hide')") -eq $true) ''
  Pf fence unroll --all | Out-Null; Start-Sleep -Seconds 2

  Case 'uninstall deletes the profile'
  $udf = Get-ChildItem "$env:LOCALAPPDATA\PecoFence\WebView2Profiles" -Directory | Sort-Object CreationTime -Descending | Select-Object -First 1
  $prof = "$($udf.FullName)\EBWebView\WV2Profile_plugin-webdemo"
  Log "profiles: $((Get-ChildItem "$($udf.FullName)\EBWebView" -Directory -ErrorAction SilentlyContinue | ForEach-Object { $_.Name }) -join ', ')"
  Check 'the plugin has a profile folder of its own' (Test-Path $prof) $prof
  $script:Cdp.Dispose(); $script:Cdp = $null
  $set = PfSettings
  Js "(window.chrome.webview.postMessage({ type: 'action', name: 'pluginUninstall', id: 'webdemo' }), 1)" | Out-Null
  $dlg = WaitWin '#32770' 5
  Check 'the uninstall asks first' ($dlg -ne [IntPtr]::Zero) ''
  [L]::PostMessageW($dlg, 0x111, [IntPtr]6, [IntPtr]::Zero) | Out-Null; Start-Sleep -Seconds 3
  $line = PfWaitLog 'web plugin: (profile deleted|profile in use)' 5
  $left = Test-Path "$($udf.FullName)\profiles-to-delete.txt"
  Check 'the uninstall deletes the profile folder, or marks it for the next start' (-not (Test-Path $prof) -or $left) "$line"
  PfQuit
  PfStart -Lang en
  PfSettings | Out-Null; Start-Sleep -Seconds 2
  Check 'after the next start the profile folder is gone' (-not (Test-Path $prof) -and -not (Test-Path "$($udf.FullName)\profiles-to-delete.txt")) ''
  Check 'no errors in the log' (@(PfLogErrors).Count -eq 0) ((PfLogErrors) -join ' | ')
}

if ($Part -eq 'N') {
  Case 'no WebView2'
  $fid = NewWebFence 'Web' '420,160,380,460'
  Check 'without WebView2 the fence says so' ($null -ne (PfWaitLog 'no WebView2, the fence says so' 30)) ''
  Start-Sleep -Seconds 2
  $out = (PfJson fence get $fid).script.output
  Check 'the fence text names WebView2' ("$out" -match 'needs WebView2') "$out"
  $h = FenceWin 'Web'; $r = Rect $h
  ParkMouse; Shot 'N1-no-webview2'
  $t = OcrText $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) 'no-webview2'
  Check 'the fence offers the download and 重新加载' ($t -match 'Download WebView2' -and $t -match 'Reload') "ocr: $t"
  $clicked = OcrClick 'Reload' $r.L $r.T ($r.R - $r.L) ($r.B - $r.T)
  $twice = @(PfLogAll | Where-Object { $_ -match 'no WebView2, the fence says so' }).Count
  Start-Sleep -Seconds 2
  $twice = @(PfLogAll | Where-Object { $_ -match 'no WebView2, the fence says so' }).Count
  Check '重新加载 tries again (still missing here)' ($clicked -and $twice -ge 2) "tries=$twice"
  Remove-Item Env:WEBVIEW2_BROWSER_EXECUTABLE_FOLDER -ErrorAction SilentlyContinue
  Check 'no errors in the log but the missing WebView2' (@(PfLogErrors | Where-Object { $_ -notmatch 'WebView2 environment failed' }).Count -eq 0) ((PfLogErrors) -join ' | ')
  # The missing WebView2 is logged as an error on purpose: the end-of-job check would count it.
  $PfAllowLogErrors = $true
}
