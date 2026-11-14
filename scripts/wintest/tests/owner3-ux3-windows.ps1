# The moments in which the app's own windows open, close and wait (owner round 2026-11-12, third
# batch), with the real mouse on the real windows. A debug build (the page is read over CDP):
#   E  新建智能栅栏… from a fence's menu, the browser not running yet: something of the editor's
#      window is on screen within 0.4 s of the click (measured 0.22 to 0.31 s; it was 0.63 to
#      0.76 s; the app shows the frame 10 to 20 ms after the menu closes, the rest is Windows
#      opening a window on a box without a GPU, where a recorded frame takes 30 to 90 ms), no
#      frame is white, black or in the other colour mode, and the name box has the keyboard
#   S  the settings window, opened for the first time after the app started: the same
#   O  栅栏选项…: the window opens beside the fence and never stands anywhere else
#   U  关于 › 检查更新: a ring turns beside 「正在检查更新…」 while the check runs
#   X  the window that is minimised comes back to where it was, in front
#   C  a window closed while only its frame is there (the browser still starting) is gone
#      without an error, and the next open shows the page
# The part is $Part (default: all). The frames of every open are kept as strips in shots\.
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'ESOUXC' }

Add-Type @'
public static class WinPx {
  // How many pixels of two same-size grabs (Px.Pixels) differ by more than thr in a channel.
  public static int Differ(int[] a, int[] b, int thr) {
    int n = 0;
    for (int i = 0; i < a.Length; i++) {
      int p = a[i], q = b[i];
      if (System.Math.Abs(((p >> 16) & 255) - ((q >> 16) & 255)) > thr || System.Math.Abs(((p >> 8) & 255) - ((q >> 8) & 255)) > thr || System.Math.Abs((p & 255) - (q & 255)) > thr) n++;
    }
    return n;
  }
  // How many pixels differ from one colour by more than thr in a channel.
  public static int OffColour(int[] a, int colour, int thr) {
    int n = 0;
    foreach (int p in a)
      if (System.Math.Abs(((p >> 16) & 255) - ((colour >> 16) & 255)) > thr || System.Math.Abs(((p >> 8) & 255) - ((colour >> 8) & 255)) > thr || System.Math.Abs((p & 255) - (colour & 255)) > thr) n++;
    return n;
  }
  [System.Runtime.InteropServices.DllImport("user32.dll")] public static extern bool ShowWindow(System.IntPtr h, int cmd);
  [System.Runtime.InteropServices.DllImport("user32.dll")] public static extern bool IsIconic(System.IntPtr h);
  // The mean of all three channels of a grab (0 black to 255 white).
  public static int Mean(int[] a) {
    long s = 0;
    foreach (int p in a) s += ((p >> 16) & 255) + ((p >> 8) & 255) + (p & 255);
    return (int)(s / System.Math.Max(1, a.Length * 3L));
  }
}
'@

function SettingsWin() { return [G]::FindWindowW('PecoFence.Settings', [NullString]::Value) }
function CloseSettings() {
  $h = SettingsWin; $r = [IntPtr]::Zero
  if ($h -ne [IntPtr]::Zero) { [G]::SendMessageTimeoutW($h, 0x10, [IntPtr]0, [IntPtr]0, 2, 5000, [ref]$r) | Out-Null }
  for ($i = 0; $i -lt 50 -and (SettingsWin) -ne [IntPtr]::Zero; $i++) { Start-Sleep -Milliseconds 100 }
}
function LastLogMs($what) {
  $line = (PfLogAll | Where-Object { $_ -match $what } | Select-Object -Last 1)
  if ($line -match 'ms=(?:Some\()?(\d+)') { return [int]$Matches[1] }
  return -1
}
function Fences() { return @(@(PfJson fence list) | Where-Object { $_ -and $_.id -and $_.kind -ne 'inbox' }) }
function LastRow($m) { return (@(MenuRows $m) | Where-Object { $_.text } | Select-Object -Last 1) }

# What the frames of the last burst show, for a window that opens in colour mode $mode over the
# grey wallpaper. Frame 0 is the desktop before the click. `sheet` is the page's background
# colour. Each later frame is the desktop still, the plain background colour (the window's
# frame before its page), the finished page (what the last frame shows), on the way (the
# window fading in, the page's content coming in), or WRONG: white in dark mode, or darker than
# the wallpaper in light mode (a black window, the page in dark colours); and once the window
# stands in its plain colour, a frame darker than the page's colours (a part of it black: the
# browser's window before its first picture), or in light mode one that is not light.
function OpenKinds($tag, $mode, $every = 1) {
  $sheet = if ($mode -eq 'dark') { 0x202020 } else { 0xF3F3F3 }
  $n = [Px]::Frames.Count
  $desk = [Px]::Pixels([Px]::Frames[0]); $final = [Px]::Pixels([Px]::Frames[$n - 1])
  $total = $desk.Length; $deskMean = [WinPx]::Mean($desk)
  $kinds = @(); $first = -1; $page = -1; $wrong = 0; $opaque = $false
  for ($i = 1; $i -lt $n; $i++) {
    $px = [Px]::Pixels([Px]::Frames[$i]); $t = [Px]::Times[$i]; $mean = [WinPx]::Mean($px)
    $k = if ([WinPx]::Differ($px, $desk, 6) -le $total / 200) { 'desktop' }
      elseif ([WinPx]::Differ($px, $final, 24) -le $total / 200) { 'page' }
      elseif ([WinPx]::OffColour($px, $sheet, 6) -le $total / 200) { 'plain' }
      elseif (($mode -eq 'dark' -and $mean -gt [math]::Max($deskMean, 0x2b) + 40) -or ($mode -eq 'light' -and $mean -lt [math]::Min($deskMean, 0xF3) - 40)) { $wrong++; "WRONG(mean $mean)" }
      elseif ($opaque -and (($mode -eq 'dark' -and $mean -lt 0x20 - 6) -or ($mode -eq 'light' -and $mean -lt 0xF3 - 28))) { $wrong++; "WRONG(mean $mean)" }
      else { "on the way(mean $mean)" }
    if ($k -eq 'plain' -or $k -eq 'page') { $opaque = $true }
    if ($k -ne 'desktop' -and $first -lt 0) { $first = $t }
    if ($k -eq 'page' -and $page -lt 0) { $page = $t }
    if ($k -match 'WRONG' -and $wrong -le 2) { [Px]::Frames[$i].Save("$PfShots\open-$tag-wrong-$wrong.png") }
    $kinds += "${t}:$k"
  }
  # (A strip of every frame of a long burst is more than GDI+ saves.)
  [Px]::Strip("$PfShots\open-$tag-strip.png", $every)
  # Runs of the same kind in one word each: "desktop x3 (0..66)".
  $runs = @(); $prev = ''; $count = 0; $from = 0; $to = 0
  foreach ($e in $kinds) {
    $t, $k = $e -split ':', 2; $k = $k -replace '\(mean \d+\)', ''
    if ($k -ne $prev) { if ($prev) { $runs += "$prev x$count ($from..$to ms)" }; $prev = $k; $count = 0; $from = $t }
    $count++; $to = $t
  }
  if ($prev) { $runs += "$prev x$count ($from..$to ms)" }
  Log "open ${tag}: desktop mean $deskMean; first of the window after $first ms, the finished page after $page ms, $wrong wrong; $($kinds -join ' ')"
  return @{ first = $first; page = $page; wrong = $wrong; runs = ($runs -join ', ') }
}

Case 'setup'
CloseWindows
Check 'a debug build (the page is reached over CDP)' $PfDebugBuild
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
GreyWallpaper
PfStart -Lang zh-CN
Start-Sleep -Seconds 2
ParkMouse
# Two fences at the right edge, clear of where the windows open.
function TwoFences() {
  if ((Fences).Count -ge 2) { return }
  Pf fence create --title '工作文档' --rect '1180,80,360,220' | Out-Null
  Pf fence create --title '图片和截图' --rect '1180,330,360,220' | Out-Null
  Start-Sleep -Seconds 1
}

# ---- S: the settings window, first open after the app started
function PartS() {
  trap { PfTrap $_; continue }
  Case 'S: the settings window answers at once, the first time after the app started'
  foreach ($mode in 'light', 'dark') {
    PfQuit; Start-Sleep -Seconds 3
    PfStart -Lang zh-CN
    Pf settings set theme $mode | Out-Null
    Start-Sleep -Seconds 3
    ParkMouse
    # The middle of where the window opens (960 x 660, centred on the work area), right of the
    # navigation column.
    Start-Process "$PfApp\pecofence-cli.exe" -ArgumentList 'settings', 'open-ui' -WindowStyle Hidden | Out-Null
    [Px]::Burst(600, 180, 560, 440, 3500, 0)
    $o = OpenKinds "settings-$mode" $mode 3
    $frame = LastLogMs 'settings: window frame shown'
    $ready = LastLogMs 'settings: page ready'
    $drawn = LastLogMs 'settings: page drawn'
    $shown = if ($frame -ge 0) { $frame } else { $drawn }
    "settings-$mode`t$($o.first)`t$($o.page)`t$frame`t$ready`t$drawn" | Add-Content "$PfShots\open-times.tsv" -Encoding UTF8
    Check "S $mode mode: the window is on screen within 150 ms of the app taking the command" ($shown -ge 0 -and $shown -le 150) "the app's log: the window shown after $shown ms, page ready after $ready ms, page drawn after $drawn ms; frames (ms after the CLI started): $($o.runs)"
    Check "S $mode mode: no frame is white, black or in the other colour mode" ($o.wrong -eq 0 -and $o.page -ge 0) "$($o.wrong) wrong frames; $($o.runs)"
    CloseSettings; Start-Sleep -Seconds 1
  }
}
if ($Part -match 'S') { PartS }

# ---- E: 新建智能栅栏… from a fence's menu
function PartE() {
  trap { PfTrap $_; continue }
  Case 'E: the smart fence editor answers the click at once'
  CloseSettings
  TwoFences
  foreach ($mode in 'light', 'dark') {
    Pf settings set theme $mode | Out-Null
    # (The browser of the window before has gone by now: its processes end with its window.)
    Start-Sleep -Seconds 4
    # 新建智能栅栏…: the last row of 添加到桌面, the last row of a fence's menu.
    $wr = (FenceGet (Fences)[0].id).windowRect
    [G]::RClick([int]($wr.x + $wr.w / 2), [int]($wr.y + 14)); Start-Sleep -Milliseconds 800
    $m = MenuWin 5
    $row = LastRow $m
    [G]::Click([int](($row.L + $row.R) / 2), [int](($row.T + $row.B) / 2)); Start-Sleep -Milliseconds 800
    $sub = @([W]::Tops('#32768')) | Where-Object { $_ -ne $m } | Select-Object -First 1
    $s = LastRow $sub
    [G]::MoveTo([int](($s.L + $s.R) / 2), [int](($s.T + $s.B) / 2)); Start-Sleep -Milliseconds 500
    # The middle of where the editor opens (880 x 660, centred on the work area), clear of the
    # menus: frame 0 is the desktop, then the click.
    [Px]::Burst(390, 170, 420, 440, 2000, 1)
    $o = OpenKinds "editor-$mode" $mode
    $frame = LastLogMs 'settings: window frame shown'
    $ready = LastLogMs 'smart fence editor: page ready'
    $drawn = LastLogMs 'smart fence editor: page drawn'
    "editor-$mode`t$($o.first)`t$($o.page)`t$frame`t$ready`t$drawn" | Add-Content "$PfShots\open-times.tsv" -Encoding UTF8
    Check "E $mode mode: something of the window is on screen within 400 ms of the click" ($o.first -ge 0 -and $o.first -le 400) "first frame of the window $($o.first) ms after the click, the finished page after $($o.page) ms; the app's log: frame shown $frame ms, page ready $ready ms, page drawn $drawn ms; frames: $($o.runs)"
    Check "E $mode mode: no frame is white, black or in the other colour mode" ($o.wrong -eq 0 -and $o.page -ge 0) "$($o.wrong) wrong frames; $($o.runs)"
    $ed = EditorWin
    Check "E $mode mode: the editor opened" ($ed -ne [IntPtr]::Zero) ''
    if ($ed -eq [IntPtr]::Zero) { continue }
    # The keyboard is in the name box without a click: what is typed lands there.
    [G]::Type('abc'); Start-Sleep -Milliseconds 500
    $name = Js "document.querySelector('#seWindow .se-name').value"
    Check "E $mode mode: the name box has the keyboard when the window is there" ($name -eq 'abc') "typed abc, the name box holds '$name'; the keyboard is in $(Js "document.activeElement.className")"
    # 取消: the frames of the closing window are kept for a look.
    $er = Rect $ed
    $c = PageRect '#seWindow .se-cancel'
    if ($c) {
      [G]::MoveTo($c.cx, $c.cy); Start-Sleep -Milliseconds 400
      [Px]::Burst($er.L + 30, $er.T + 74, 420, 440, 700, 1); [Px]::Strip("$PfShots\close-editor-$mode-strip.png", 1)
    }
    Start-Sleep -Milliseconds 600
    if ((EditorCount) -gt 0) { [G]::Press(0x1B); Start-Sleep -Milliseconds 800 }
    Check "E $mode mode: 取消 closes the editor" ((EditorCount) -eq 0) "editor windows $(EditorCount)"
  }
}
if ($Part -match 'E') { PartE }

# ---- O: 栅栏选项… opens the window beside the fence at once
function PartO() {
  trap { PfTrap $_; continue }
  Case 'O: 栅栏选项… opens beside the fence and stands still'
  CloseSettings
  TwoFences
  $f = (Fences)[0]
  $wr = (FenceGet $f.id).windowRect
  [G]::RClick([int]($wr.x + $wr.w / 2), [int]($wr.y + 14)); Start-Sleep -Milliseconds 800
  $m = MenuWin 5
  $p = MenuPoint $m '栅栏选项'
  Check 'O the fence menu has 栅栏选项…' ($null -ne $p) (MenuTexts $m)
  if (-not $p) { return }
  [G]::Click($p[0], $p[1])
  # Every place the window is seen at from the moment it is visible.
  $sw = [Diagnostics.Stopwatch]::StartNew(); $places = @()
  while ($sw.ElapsedMilliseconds -lt 3000) {
    $h = SettingsWin
    if ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) { $r = Rect $h; $t = "$($r.L),$($r.T) $($r.R - $r.L)x$($r.B - $r.T)"; if ($places.Count -eq 0 -or $places[-1] -ne $t) { $places += $t } }
    Start-Sleep -Milliseconds 5
  }
  $r = Rect (SettingsWin)
  $clear = ($r.R -le $wr.x) -or ($r.L -ge $wr.x + $wr.w)
  Check 'O the window is seen at one place only, beside the fence' ($places.Count -eq 1 -and $clear) "places while visible: $($places -join ' -> '); fence $($wr.x),$($wr.y) $($wr.w)x$($wr.h)"
  CloseSettings
}
if ($Part -match 'O') { PartO }

# ---- U: 检查更新 shows a ring while it runs
function PartU() {
  trap { PfTrap $_; continue }
  Case 'U: a ring turns beside 正在检查更新…'
  # A feed that takes the connection and never answers: the check runs until it gives up.
  CloseSettings; PfQuit; Start-Sleep -Seconds 2
  $dead = New-Object Net.Sockets.TcpListener ([Net.IPAddress]::Loopback), 43591
  $dead.Start()
  $env:PECOFENCE_UPDATE_URL = 'http://127.0.0.1:43591/repos/DayuanJiang/PecoFence/releases/latest'
  $env:PECOFENCE_UPDATE_SITE_URL = 'http://127.0.0.1:43591/update.json'
  PfStart -Lang zh-CN
  Pf settings set theme light | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse
  Check 'U settings open over CDP' (PfSettings)
  PageClick 'nav a[data-page=about]'; Start-Sleep -Milliseconds 800
  $b = PageRect '#updateAction'; $card = PageRect '#updateCard'
  Check 'U the update card is on the page' ($null -ne $card -and $null -ne $b) ''
  if ($b) {
    [G]::MoveTo($b.cx, $b.cy); Start-Sleep -Milliseconds 400
    # The card from the click on, then what the page says while the check still runs.
    [Px]::Burst($card.x, $card.y, $card.w, $card.h, 1500, 1)
    $seen = Js "(() => { const s = document.getElementById('updateStatus'); return s.textContent + ' | ring animations: ' + document.getAnimations().filter(a => a.animationName === 'ring-turn').length + ' | width of the ring: ' + getComputedStyle(s, '::before').width; })()"
    [Px]::Strip("$PfShots\update-check-strip.png", 3)
    # The left end of the status line, where the ring turns: how many different pictures in
    # the last second (the line has faded in by then).
    $st = PageRect '#updateStatus'
    $x0 = [math]::Max(0, $st.x - $card.x - 1); $y0 = [math]::Max(0, $st.y - $card.y - 1)
    $pics = @{}; $late = 0
    for ($i = 0; $i -lt [Px]::Frames.Count; $i++) {
      if ([Px]::Times[$i] -lt 500) { continue }
      $late++
      $crop = [Px]::Frames[$i].Clone((New-Object Drawing.Rectangle $x0, $y0, 16, ([math]::Min(18, $card.h - $y0))), [Px]::Frames[$i].PixelFormat)
      $pics[(([Px]::Pixels($crop)) -join ',')] = 1; $crop.Dispose()
    }
    Check 'U while the check runs the line has a ring that turns' ($seen -match '^正在检查更新… \| ring animations: 1 \| width of the ring: 12px' -and $pics.Count -ge 3) "1.5 s after the click the page says: $seen; $($pics.Count) different pictures of the ring's place in the $late frames from 0.5 s on"
  }
  CloseSettings; PfQuit
  $dead.Stop()
  Remove-Item Env:\PECOFENCE_UPDATE_URL, Env:\PECOFENCE_UPDATE_SITE_URL -ErrorAction SilentlyContinue
  PfStart -Lang zh-CN
  Start-Sleep -Seconds 2
}
if ($Part -match 'U') { PartU }

# ---- X: a window that is minimised comes back
function PartX() {
  trap { PfTrap $_; continue }
  Case 'X: the settings window comes back from the taskbar'
  Check 'X settings open' (PfSettings)
  $h = SettingsWin; $was = Rect $h
  [WinPx]::ShowWindow($h, 6) | Out-Null; Start-Sleep -Milliseconds 800
  Pf settings open-ui | Out-Null; Start-Sleep -Milliseconds 1200
  $now = Rect $h
  Check 'X minimised: 设置… restores it to where it was, in front' (-not [WinPx]::IsIconic($h) -and [G]::GetForegroundWindow() -eq $h -and $now.L -eq $was.L -and $now.T -eq $was.T -and $now.R -eq $was.R) "iconic $([WinPx]::IsIconic($h)); foreground $(Fg); was $($was.L),$($was.T), now $($now.L),$($now.T)"
  # The X: the frames of the closing window are kept for a look.
  $r = Rect $h
  [G]::MoveTo($r.R - 24, $r.T + 16); Start-Sleep -Milliseconds 500
  [Px]::Burst($r.L + 240, $r.T, 700, 400, 700, 1); [Px]::Strip("$PfShots\close-settings-strip.png", 1)
  Start-Sleep -Milliseconds 500
  Check 'X the X closes the window' ((SettingsWin) -eq [IntPtr]::Zero) ''
  CloseWindows
}
if ($Part -match 'X') { PartX }

# ---- C: closed before the page is there
function PartC() {
  trap { PfTrap $_; continue }
  Case 'C: a window closed before its page arrives is gone'
  CloseSettings
  # (The browser of the window before has gone by now: its processes end with its window.)
  Start-Sleep -Seconds 4
  $sw = [Diagnostics.Stopwatch]::StartNew()
  Start-Process "$PfApp\pecofence-cli.exe" -ArgumentList 'settings', 'open-ui' -WindowStyle Hidden | Out-Null
  $h = [IntPtr]::Zero
  while ($sw.ElapsedMilliseconds -lt 10000 -and ($h -eq [IntPtr]::Zero -or -not [G]::IsWindowVisible($h))) { $h = SettingsWin; Start-Sleep -Milliseconds 2 }
  $vis = $sw.ElapsedMilliseconds; $r = [IntPtr]::Zero
  [G]::SendMessageTimeoutW($h, 0x10, [IntPtr]0, [IntPtr]0, 2, 5000, [ref]$r) | Out-Null
  $sent = $sw.ElapsedMilliseconds
  Start-Sleep -Seconds 3
  $said = @(PfLogAll | Where-Object { $_ -match 'settings: window closed (before|while) the browser started' }).Count
  $failed = @(PfLogAll | Where-Object { $_ -match 'settings window failed|WebView2 environment failed' }).Count
  Check 'C the window is gone and the app reports no failure' ((SettingsWin) -eq [IntPtr]::Zero -and $failed -eq 0) "window visible $vis ms after the CLI started, closed at $sent ms; log lines 'window closed before / while the browser started': $said; failure lines: $failed"
  Check 'C the next open shows the page' (PfSettings) ''
  CloseSettings
}
if ($Part -match 'C') { PartC }

Case 'cleanup'
CloseSettings
if ((EditorCount) -gt 0) { [G]::Press(0x1B); Start-Sleep -Milliseconds 500 }
CloseWindows
PfQuit
