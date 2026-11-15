# 网页层 sample: the test web plugin (assets/webdemo) in a fence. Part A (debug build, CDP on the
# page): the page loads in the fence's visual tree, the mouse, wheel and keyboard reach it, pf.time.now
# answers, and what the plugin must not do is refused. Part B (release build): screenshots
# next to a normal fence, clips of roll-up, fade, move and resize. Part M (release build): the
# memory of PecoFence's process group with no web plugin, one, two, and after 60 s hidden.
# Part I (debug build): Chinese typing with Microsoft Pinyin in the page. run.sh --var Part=A|B|M|I (default A).
if (-not $Part) { $Part = 'A' }
$Demo = 'C:\pf\kit\assets\webdemo'

# Installs the test plugin as `id` (a copy of webdemo under another id for the second plugin).
function InstallDemo([string]$id = 'webdemo') {
  $to = "$script:Config\plugins\$id"
  New-Item -ItemType Directory -Force $to | Out-Null
  Copy-Item "$Demo\*" $to -Force
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
if ($Part -in 'A', 'I' -and $PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
PfStart -Lang en
$script:Config = Split-Path (PfJson status).configPath
InstallDemo
Check 'the test plugin is installed' (Test-Path "$script:Config\plugins\webdemo\index.html") $script:Config

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
  InstallDemo 'webdemo2'
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
