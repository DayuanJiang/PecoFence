# scripts/wintest/page.ps1: the settings page (WebView2) over CDP, loaded by lib.ps1. UI Automation
# sees none of the page on the box: start a debug build with
# $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352', read rects and state with Js, click
# with the real pointer. $PfDebugBuild tells the two builds apart.

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
function PfSettings([string]$ReadyJs = 'true') {
  Pf settings open-ui | Out-Null
  $h = WaitWin 'PecoFence.Settings' 30
  if ($h -eq [IntPtr]::Zero) { return $false }
  if (-not $PfDebugBuild) { Start-Sleep -Seconds 6; return $true }
  if (-not (CdpOpen)) { return $false }
  return [bool](JsWait "document.documentElement.classList.contains('ready') && ($ReadyJs)" 20)
}

$PfDebugBuild = IsDebugBuild
