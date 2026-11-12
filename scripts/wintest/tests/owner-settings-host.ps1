# The settings window as a window (owner round 2026-11-12, review of the settings window):
#   O  it appears with the page: no black frame, no dark page in light mode, and seldom a frame
#      without the state; how long an open takes (any build)
#   K  the browser's own keys (F5, Ctrl+R, F12, Ctrl+Shift+I, F3, F7, Ctrl+J, Ctrl+U) do nothing;
#      Ctrl+F still goes to 查找设置 and the text keys still edit (any build; more on a debug one)
#   R  a reopen in the same run comes back on the page, the scroll offset, the size and the place
#      it was closed with; 栅栏选项… and a tip's 打开设置 still go where they point (debug build)
#   D  deleting a rule or a snapshot shows a notice with 撤销 that puts it back (debug build)
# run.sh --var Part=O|K|R|D (default: all). --var Rec=<name> records the before/after animations:
# with Part=V an open in light and in dark mode and four of the keys (any build), with Part=R the
# reopen, with Part=D the rule delete and its undo.
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'OKRD' }
if (-not (Get-Variable Rec -ErrorAction SilentlyContinue)) { $Rec = '' }
if ($Rec) { . C:\pf\kit\rec.ps1 }

Add-Type @'
public static class OwnerPx {
  // How many pixels of two same-size grabs (Px.Pixels) differ by more than thr in a channel,
  // inside x0..x1, y0..y1.
  public static int Differ(int[] a, int[] b, int w, int x0, int y0, int x1, int y1, int thr) {
    int n = 0;
    for (int y = y0; y < y1; y++) for (int x = x0; x < x1; x++) {
      int p = a[y * w + x], q = b[y * w + x];
      if (System.Math.Abs(((p >> 16) & 255) - ((q >> 16) & 255)) > thr || System.Math.Abs(((p >> 8) & 255) - ((q >> 8) & 255)) > thr || System.Math.Abs((p & 255) - (q & 255)) > thr) n++;
    }
    return n;
  }
  // How many pixels inside x0..x1, y0..y1 differ from one colour by more than thr in a channel.
  public static int OffColour(int[] a, int w, int x0, int y0, int x1, int y1, int colour, int thr) {
    int n = 0;
    for (int y = y0; y < y1; y++) for (int x = x0; x < x1; x++) {
      int p = a[y * w + x];
      if (System.Math.Abs(((p >> 16) & 255) - ((colour >> 16) & 255)) > thr || System.Math.Abs(((p >> 8) & 255) - ((colour >> 8) & 255)) > thr || System.Math.Abs((p & 255) - (colour & 255)) > thr) n++;
    }
    return n;
  }
}
'@

function SettingsWin() { return [G]::FindWindowW('PecoFence.Settings', [NullString]::Value) }
function SettingsShown() { $h = SettingsWin; return ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) }
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

# ---- O: the first frames of the window, and how long an open takes
# Opens the settings and grabs the window's rectangle as fast as the box allows from the moment
# the window is visible. Every grab is the desktop as it was (the window is not on screen yet),
# the finished window (what it shows 2 s later), the finished page fading in (the navigation
# column, which does not fade and ends with the fence and item counts of the state, is the
# finished one already), a frame without the state in the right colours (the page's background
# colour alone, or the page as it is before the state arrives: what WebView2 had drawn last when
# its window was shown; counted, a failure only when it happens more than once in the 7 opens),
# or something else: a black client area, the page in the other colour mode. Returns how many
# grabs are something else, how many have no state yet, when the right page was first on screen
# and when it had settled (ms after the CLI started), and the app's own numbers.
function OpenFrames($tag) {
  $sw = [Diagnostics.Stopwatch]::StartNew()
  Start-Process "$PfApp\pecofence-cli.exe" -ArgumentList 'settings', 'open-ui' -WindowStyle Hidden | Out-Null
  $h = [IntPtr]::Zero
  while ($sw.ElapsedMilliseconds -lt 20000 -and $h -eq [IntPtr]::Zero) { $h = SettingsWin; if ($h -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 4 } }
  $r = Rect $h
  $w = $r.R - $r.L; $hh = $r.B - $r.T
  $desk = [Px]::Pixels([Px]::Grab($r.L, $r.T, $w, $hh))
  while ($sw.ElapsedMilliseconds -lt 20000 -and -not [G]::IsWindowVisible($h)) { Start-Sleep -Milliseconds 2 }
  $vis = $sw.ElapsedMilliseconds
  [Px]::Burst($r.L, $r.T, $w, $hh, 450, 0)
  Start-Sleep -Milliseconds 2000
  $final = [Px]::Pixels([Px]::Grab($r.L, $r.T, $w, $hh))
  # The client area without the window's edges and the scrollbar.
  $x0 = 12; $y0 = 40; $x1 = $w - 30; $y1 = $hh - 12
  # The page's background colour: an empty spot at the top left of the navigation column.
  $sheet = $final[($y0 + 2) * $w + $x0 + 2]
  $kinds = @(); $wrong = 0; $plain = 0; $firstFinal = -1; $firstPage = -1
  for ($i = 0; $i -lt [Px]::Frames.Count; $i++) {
    $px = [Px]::Pixels([Px]::Frames[$i])
    $dFinal = [OwnerPx]::Differ($px, $final, $w, $x0, $y0, $x1, $y1, 24)
    $dDesk = [OwnerPx]::Differ($px, $desk, $w, $x0, $y0, $x1, $y1, 24)
    $dNav = [OwnerPx]::Differ($px, $final, $w, $x0, $y0, 236, $y1, 24)
    if ($dFinal -le 60) { $kinds += "$([Px]::Times[$i]):finished"; if ($firstFinal -lt 0) { $firstFinal = $vis + [Px]::Times[$i] } }
    elseif ($dDesk -le 60) { $kinds += "$([Px]::Times[$i]):desktop" }
    elseif ($dNav -le 40) { $kinds += "$([Px]::Times[$i]):fading in ($dFinal px to go)" }
    elseif ([OwnerPx]::OffColour($px, $w, $x0, $y0, $x1, $y1, $sheet, 6) -le 60) { $plain++; $kinds += "$([Px]::Times[$i]):background colour only" }
    elseif ($dNav -le 2000) { $plain++; $kinds += "$([Px]::Times[$i]):the page before it has the state (navigation column differs in $dNav px)"; if ($plain -le 2) { [Px]::Frames[$i].Save("$PfShots\open-$tag-no-state-$plain.png") } }
    else {
      $wrong++
      $m = MeanRgb ([Px]::Frames[$i])
      $kinds += "$([Px]::Times[$i]):OTHER(differs from the finished window in $dFinal px, its navigation column in $dNav px, mean colour $($m -join ','))"
      if ($wrong -le 3) { [Px]::Frames[$i].Save("$PfShots\open-$tag-other-$wrong.png") }
    }
    if ($firstPage -lt 0 -and $kinds[-1] -match ':(finished|fading)') { $firstPage = $vis + [Px]::Times[$i] }
  }
  [Px]::Strip("$PfShots\open-$tag-strip.png", 1)
  $ready = LastLogMs 'settings: page ready'
  $drawn = LastLogMs 'settings: page drawn, window shown'
  Log "open ${tag}: visible after $vis ms, the right page on screen after $firstPage ms, settled after $firstFinal ms (cli start included); app log: page ready $ready ms, window shown $drawn ms; frames: $($kinds -join ' | ')"
  "$tag`t$vis`t$firstPage`t$firstFinal`t$ready`t$drawn`t$wrong`t$plain" | Add-Content "$PfShots\open-times.tsv" -Encoding UTF8
  return @{ wrong = $wrong; plain = $plain; vis = $vis; page = $firstPage; final = $firstFinal; ready = $ready; drawn = $drawn; frames = ($kinds -join ' | ') }
}
function PartO() {
  trap { PfTrap $_; continue }
  Case 'O: the window appears with the finished page'
  CloseWindows
  PfStart -Lang 'zh-CN'
  ParkMouse
  $early = 0
  foreach ($mode in 'light', 'dark') {
    Pf settings set theme $mode | Out-Null
    Start-Sleep -Seconds 2
    foreach ($n in 1, 2, 3) {
      $o = OpenFrames "$mode-$n"
      if ($o.plain -gt 0) { $early++ }
      Check "O $mode mode, open ${n}: no frame of the window is black or in the other colour mode" ($o.wrong -eq 0 -and $o.page -ge 0) "$($o.wrong) other frames, $($o.plain) without the state yet; right page after $($o.page) ms; $($o.frames)"
      CloseSettings; Start-Sleep -Seconds 2
    }
  }
  # After an app restart (the browser processes are gone): once more.
  PfQuit; Start-Sleep -Seconds 3
  PfStart -Lang 'zh-CN'
  ParkMouse
  $o = OpenFrames 'restart'
  Check 'O after a restart: no frame of the window is black or in the other colour mode' ($o.wrong -eq 0 -and $o.page -ge 0) "$($o.wrong) other frames, $($o.plain) without the state yet; right page after $($o.page) ms; $($o.frames)"
  if ($o.plain -gt 0) { $early++ }
  # Seen in every dark mode open when the window was shown on `ready`, in about one open in
  # eight on the software-rendered box when the page said `drawn` two frames after the state,
  # and in none of 28 opens with three frames.
  Check 'O at most 1 of the 7 opens starts with a frame that has no state yet' ($early -le 1) "$early of 7"
  Check 'O the window shows within 3 s' ($o.vis -lt 3000) "visible after $($o.vis) ms"
  CloseSettings
  (PfLogAll | Where-Object { $_ -match 'settings: ' } | Select-Object -Last 14) | Set-Content "$PfShots\open-log.txt" -Encoding UTF8
}
if ($Part -match 'O') { PartO }

# ---- K: the browser's own keys
# The window's pixels (edges and scrollbar left out) and how many differ between two grabs.
function WinGrab() { $r = Rect (SettingsWin); return [Px]::Pixels([Px]::Grab($r.L, $r.T, $r.R - $r.L, $r.B - $r.T)) }
function GrabDiff($a, $b) { $r = Rect (SettingsWin); $w = $r.R - $r.L; return [OwnerPx]::Differ($a, $b, $w, 12, 40, $w - 30, $r.B - $r.T - 12, 24) }
# Windows of the browser besides the settings window (DevTools, the page source, a find bar).
function BrowserWindows() { return (([L]::Tops('') -split "`n") | Where-Object { $_ -and $_ -notmatch 'PecoFence\.|Shell_TrayWnd|Progman|WorkerW' -and $_ -match 'msedge|webview|DevTools|Chrome_WidgetWin' } | ForEach-Object { if ($_.Length -gt 90) { $_.Substring(0, 90) } else { $_ } }) -join ' ;; ' }
# Away from 常规 with the page's own keys: Ctrl+F is 查找设置, Enter opens the first result
# (把当前布局存为快照 on 布局与备份).
function GoToSnapshots() {
  [G]::SetForegroundWindow((SettingsWin)) | Out-Null; Start-Sleep -Milliseconds 300
  Chord 0x11 0x46; Start-Sleep -Milliseconds 400
  Chord 0x11 0x41; [G]::Type('快照'); Start-Sleep -Milliseconds 900
  [G]::Press(0x0D); Start-Sleep -Milliseconds 3000
}
function PartK() {
  trap { PfTrap $_; continue }
  Case 'K: the browser keys do nothing in the settings window'
  CloseWindows
  if ($PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
  PfStart -Lang 'zh-CN'
  ParkMouse
  $ok = PfSettings
  Check 'K the settings window opens' $ok "debug=$PfDebugBuild"
  [G]::SetForegroundWindow((SettingsWin)) | Out-Null; Start-Sleep -Milliseconds 500
  $general = WinGrab
  $base = BrowserWindows
  Log "browser windows before: $base"
  if ($PfDebugBuild) { Js 'window.__ownerMark = 1' | Out-Null }
  GoToSnapshots
  $d = GrabDiff (WinGrab) $general
  Check 'K Ctrl+F is 查找设置 and Enter goes to the result on another page' ($d -gt 20000) "the window differs from 常规 in $d px"
  $keys = @(
    @('F5', { [G]::Press(0x74) }), @('Ctrl+R', { Chord 0x11 0x52 }), @('F12', { [G]::Press(0x7B) }),
    @('Ctrl+Shift+I', { [G]::KeyDown(0x11); [G]::KeyDown(0x10); [G]::Press(0x49); [G]::KeyUp(0x10); [G]::KeyUp(0x11) }),
    @('F3', { [G]::Press(0x72) }), @('Ctrl+G', { Chord 0x11 0x47 }), @('F7', { [G]::Press(0x76) }),
    @('Ctrl+J', { Chord 0x11 0x4A }), @('Ctrl+U', { Chord 0x11 0x55 }), @('Ctrl+P', { Chord 0x11 0x50 })
  )
  foreach ($k in $keys) {
    # A key before this one reloaded the page (back on 常规): away from it again.
    if ((GrabDiff (WinGrab) $general) -le 60) { GoToSnapshots }
    [G]::SetForegroundWindow((SettingsWin)) | Out-Null; Start-Sleep -Milliseconds 300
    $before = WinGrab
    & $k[1]
    Start-Sleep -Milliseconds 1800
    $d = GrabDiff $before (WinGrab)
    $others = BrowserWindows; $fg = Fg
    if ($fg.Length -gt 80) { $fg = $fg.Substring(0, 80) }
    Check "K $($k[0]) does nothing" ($d -le 60 -and $others -eq $base -and $fg -match 'PecoFence.Settings') "the window changed in $d px; foreground $fg; browser windows: $others"
    # Close whatever the key opened.
    [G]::Press(0x1B); Start-Sleep -Milliseconds 400
    if ((Fg) -notmatch 'PecoFence.Settings') { Chord 0x12 0x73; Start-Sleep -Milliseconds 800 }
  }
  Check 'K the settings window is still open' (SettingsShown) (Fg)
  if ($PfDebugBuild) {
    Check 'K the page was never loaded again' ((Js 'window.__ownerMark === 1') -eq $true) "mark: $(Js 'String(window.__ownerMark)')"
    Check 'K a debug build started with PECOFENCE_WEBVIEW_ARGS still answers DevTools' ((Js 'document.title') -match 'PecoFence') "title: $(Js 'document.title')"
    # The text keys still edit: select all, copy, paste twice, undo, cut.
    [G]::SetForegroundWindow((SettingsWin)) | Out-Null; Start-Sleep -Milliseconds 300
    Chord 0x11 0x46; Start-Sleep -Milliseconds 400
    Check 'K Ctrl+F puts the focus in 查找设置' ((Js 'document.activeElement.id') -eq 'settingsSearch') "focus: $(Js 'document.activeElement.id')"
    Chord 0x11 0x41; [G]::Type('abc'); Start-Sleep -Milliseconds 300
    Chord 0x11 0x41; Chord 0x11 0x43; Start-Sleep -Milliseconds 200
    [G]::Press(0x23); Chord 0x11 0x56; Start-Sleep -Milliseconds 300
    $pasted = Js "document.getElementById('settingsSearch').value"
    Chord 0x11 0x5A; Start-Sleep -Milliseconds 300
    $undone = Js "document.getElementById('settingsSearch').value"
    Chord 0x11 0x41; Chord 0x11 0x58; Start-Sleep -Milliseconds 300
    $cut = Js "document.getElementById('settingsSearch').value"
    Check 'K Ctrl+A, Ctrl+C, Ctrl+V, Ctrl+Z and Ctrl+X still edit a text box' ($pasted -eq 'abcabc' -and $undone -eq 'abc' -and -not $cut) "after paste '$pasted', after undo '$undone', after cut '$cut'"
  }
  Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  CloseSettings
  PfQuit
}
if ($Part -match 'K') { PartK }

# ---- R: a reopen in the same run (debug build: the page is read over CDP)
function PageView() { return Js "({ page: document.querySelector('nav a.sel').dataset.page, scroll: Math.round(document.querySelector('main').scrollTop), fence: document.getElementById('fenceSel').value })" }
function RectText($r) { return "$($r.L),$($r.T) $($r.R - $r.L)x$($r.B - $r.T)" }
function PartR() {
  trap { PfTrap $_; continue }
  Case 'R: a reopen comes back where the window was closed'
  CloseWindows
  Remove-Item -LiteralPath (Join-Path $PfDesk '季度销售报告.pdf') -Force -ErrorAction SilentlyContinue
  $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
  PfStart -Lang 'zh-CN'
  ParkMouse
  $ok = PfSettings
  Check 'R the settings open over CDP (debug build)' $ok "debug=$PfDebugBuild"
  $h = SettingsWin; $first = Rect $h
  # Left as a user would leave it: moved, resized, on 速览与搜索, scrolled down.
  if ($Rec) { RecStart "$Rec-reopen" -X 0 -Y 0 -W 1600 -H 860 -Fps 8; Start-Sleep -Milliseconds 600; RecMark 'move the window, open another page, scroll' }
  [L]::MoveWindow($h, 60, 40, 1200, 780, $true) | Out-Null; Start-Sleep -Milliseconds 500
  PageClick 'nav a[data-page=peek]'
  Js "document.querySelector('main').scrollTop = 300" | Out-Null
  Start-Sleep -Milliseconds 700
  $left = PageView; $leftRect = Rect $h
  Shot 'r-closed-like-this'
  if ($Rec) { Start-Sleep -Milliseconds 800; RecMark 'close the window' }
  CloseSettings; Start-Sleep -Seconds 1
  if ($Rec) { RecMark 'open the settings again' }
  $ok = PfSettings
  $h = SettingsWin; $again = Rect $h; $now = PageView
  Shot 'r-reopened'
  if ($Rec) { Start-Sleep -Milliseconds 1500; RecStop }
  Check 'R reopened at the place and size it was closed with' ((RectText $again) -eq (RectText $leftRect) -and (RectText $leftRect) -eq '60,40 1200x780') "closed at $(RectText $leftRect), reopened at $(RectText $again); the first open was at $(RectText $first)"
  Check 'R reopened on the page it was closed on' ($ok -and $now.page -eq 'peek' -and $left.page -eq 'peek') "closed on $($left.page), reopened on $($now.page)"
  Check 'R reopened scrolled as it was' ($left.scroll -ge 200 -and [math]::Abs($now.scroll - $left.scroll) -le 2) "closed at scroll $($left.scroll), reopened at $($now.scroll)"

  Case 'R: 栅栏选项… still shows that fence, beside it'
  CloseSettings; Start-Sleep -Seconds 1
  $fence = @(PfJson fence list | Where-Object { $_ -and $_.id -and $_.kind -ne 'inbox' })[0]
  $fr = (FenceGet $fence.title).windowRect
  $m = FenceMenu $fence.title
  $picked = MenuPick '^栅栏选项' $m
  CloseMenus
  $h = WaitWin 'PecoFence.Settings' 20
  $ok = (CdpOpen) -and [bool](JsWait "document.documentElement.classList.contains('ready')" 20)
  Start-Sleep -Milliseconds 800
  $now = PageView; $sr = Rect $h
  $overlap = -not ($sr.R -le $fr.x -or $sr.L -ge $fr.x + $fr.w -or $sr.B -le $fr.y -or $sr.T -ge $fr.y + $fr.h)
  Shot 'r-fence-options'
  Check 'R 栅栏选项…: the 栅栏 page with that fence chosen' ($ok -and $now.page -eq 'fences' -and $now.fence -eq "$($fence.id)") "picked '$picked'; page $($now.page), fence chosen $($now.fence), asked for $($fence.id) ($($fence.title))"
  Check 'R 栅栏选项…: the window stands clear of the fence, on screen' ($h -ne [IntPtr]::Zero -and -not $overlap -and $sr.L -ge 0 -and $sr.T -ge 0 -and $sr.R -le 1600 -and $sr.B -le 900) "settings $(RectText $sr); fence $($fr.x),$($fr.y) $($fr.w)x$($fr.h)"

  Case 'R: a card that points at a settings page still opens that page'
  CloseSettings; Start-Sleep -Seconds 1
  # The item menu's 以后把 .pdf 文件都放到这里 adds a rule and shows a card whose button opens
  # 整理规则 (the last window was closed on 栅栏).
  $pdf = DeskFile '季度销售报告.pdf'
  Start-Sleep -Seconds 3
  $it = @(PfJson item list | Where-Object { $_ -and $_.fileName -eq '季度销售报告.pdf' })[0]
  $m = RClickItem $it.fenceTitle $it.name
  $picked = MenuPick '以后把' $m
  CloseMenus
  $card = WaitWin 'PecoFence.Notice' 8
  $cr = Rect $card
  Shot 'r-rule-card'
  [G]::Click($cr.R - 46, $cr.B - 32); Start-Sleep -Milliseconds 900
  $h = WaitWin 'PecoFence.Settings' 20
  $ok = (CdpOpen) -and [bool](JsWait "document.documentElement.classList.contains('ready')" 20)
  Start-Sleep -Milliseconds 800
  $now = PageView
  Shot 'r-card-opens-rules'
  Check 'R the card''s button opens 整理规则, not the page of the last close' ($ok -and $now.page -eq 'rules') "item in '$($it.fenceTitle)', menu '$picked', card '$([L]::Text($card))'; page $($now.page)"
  CloseSettings
  PfQuit
  Remove-Item -LiteralPath $pdf -Force -ErrorAction SilentlyContinue
}
if ($Part -match 'R') { PartR }

# ---- D: delete with 撤销 (debug build: the page is read over CDP)
# PowerShell 5.1 reads the curly quotes as string delimiters: the notice's quotes by code.
$Lq = [char]0x201C; $Rq = [char]0x201D
function UndoShown() { return Js "(() => { const b = document.getElementById('toastUndo'), bar = document.getElementById('toast'); return !!b && !b.hidden && getComputedStyle(b).visibility === 'visible' && getComputedStyle(bar).pointerEvents === 'auto'; })()" }
function SnapText() { return (@(Snaps) | ForEach-Object { "$($_.id) $($_.name) $($_.ts)" }) -join "`n" }
# A snapshot as the config file holds it (the layouts and all), once the save timer has run.
function SnapInFile($id) {
  Start-Sleep -Milliseconds 1800
  $path = Js "document.getElementById('cfgPath').textContent"
  $cfg = Get-Content -LiteralPath $path -Raw -Encoding UTF8 | ConvertFrom-Json
  $s = @($cfg.snapshots | Where-Object { $_.id -eq $id })[0]
  if (-not $s) { return '' }
  return ($s | ConvertTo-Json -Depth 40 -Compress)
}
function PartD() {
  trap { PfTrap $_; continue }
  Case 'D: a deleted rule comes back with 撤销'
  CloseWindows
  $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
  PfStart -Lang 'zh-CN'
  ParkMouse
  Pf snapshot save '整理之前' | Out-Null
  Pf snapshot save '周末的桌面布局' | Out-Null
  $ok = PfSettings
  Check 'D the settings open over CDP (debug build)' $ok "debug=$PfDebugBuild"
  PageClick 'nav a[data-page=rules]'
  $before = @(Rules); $sigs = ($before | ForEach-Object { RuleSig $_ }) -join "`n"
  $victim = $before[1]
  if ($Rec) { $wr = Rect (SettingsWin); RecStart "$Rec-delete" -X ($wr.L - 20) -Y ($wr.T - 16) -W ($wr.R - $wr.L + 40) -H ($wr.B - $wr.T + 32) -Fps 10; Start-Sleep -Milliseconds 800; RecMark 'click the bin of the second rule' }
  PageClick '#ruleList .card.rule [data-act=del]' 1
  Start-Sleep -Milliseconds 600
  $t = PageToast; $undo = UndoShown
  Shot 'd-rule-deleted'
  if ($Rec) { Start-Sleep -Milliseconds 1200; RecMark 'click Undo on the notice' }
  Check 'D rule: one click deletes it' (@(Rules).Count -eq $before.Count - 1 -and -not (RuleById $victim.id)) "rules $($before.Count) -> $(@(Rules).Count)"
  Check 'D rule: the notice names the rule and offers 撤销' ($t.on -and $t.text -eq "已删除规则$Lq$($victim.name)$Rq" -and $undo) "notice on=$($t.on) text='$($t.text)'; 撤销 shown and clickable: $undo"
  PageClick '#toastUndo'
  Start-Sleep -Milliseconds 900
  $after = (@(Rules) | ForEach-Object { RuleSig $_ }) -join "`n"
  Check 'D rule: 撤销 puts it back at its place with all its fields' ($after -eq $sigs) "before: $sigs || after: $after"
  Check 'D rule: the notice goes away with 撤销' (-not (PageToast).on) "on=$((PageToast).on)"
  Shot 'd-rule-back'
  if ($Rec) { Start-Sleep -Milliseconds 1500; RecStop }
  PageClick '#ruleList .card.rule [data-act=del]' 1
  Start-Sleep -Milliseconds 4000
  $still = (PageToast).on
  Start-Sleep -Milliseconds 3200
  Check 'D rule: left alone, the notice stays about 6 s and the rule stays deleted' ($still -and -not (PageToast).on -and -not (UndoShown) -and -not (RuleById $victim.id)) "on after 4.5 s: $still; on after 7.5 s: $((PageToast).on); rules now $(@(Rules).Count)"
  $second = @(Rules)[0]
  PageClick '#ruleList .card.rule [data-act=del]' 0
  Start-Sleep -Milliseconds 500
  CloseSettings; Start-Sleep -Seconds 1
  Check 'D rule: closing the window keeps the delete' (-not (RuleById $second.id) -and @(Rules).Count -eq $before.Count - 2) "rules now $(@(Rules).Count)"

  Case 'D: a deleted snapshot comes back with 撤销'
  $ok = PfSettings
  PageClick 'nav a[data-page=layout]'
  $snaps = SnapText
  $gone = @(Snaps | Where-Object { $_.name -eq '整理之前' })[0]
  $inFile = SnapInFile $gone.id
  $row = "#snapList .card[data-row-id=`"$($gone.id)`"] [data-act=del]"
  PageClick $row
  Start-Sleep -Milliseconds 600
  $t = PageToast; $undo = UndoShown
  Shot 'd-snapshot-deleted'
  Check 'D snapshot: one click deletes it' (@(Snaps).Count -eq 1 -and -not (@(Snaps) | Where-Object { $_.id -eq $gone.id })) "snapshots now: $(SnapText)"
  Check 'D snapshot: the notice names the snapshot and offers 撤销' ($t.on -and $t.text -eq "已删除快照${Lq}整理之前$Rq" -and $undo) "notice on=$($t.on) text='$($t.text)'; 撤销 shown and clickable: $undo"
  PageClick '#toastUndo'
  Start-Sleep -Milliseconds 900
  $back = SnapInFile $gone.id
  Check 'D snapshot: 撤销 puts it back at its place with its name' ((SnapText) -eq $snaps) "before: $snaps || after: $(SnapText)"
  Check 'D snapshot: its content is the same as before the delete' ($inFile.Length -gt 100 -and $back -eq $inFile) "in the config file: $($inFile.Length) chars before, $($back.Length) after"
  Check 'D snapshot: the row is in the list again' ((Js "document.querySelectorAll('#snapList .card[data-row-id]').length") -eq 2) "rows: $(Js "[...document.querySelectorAll('#snapList .card .t')].map(e => e.textContent).join(' | ')")"
  Shot 'd-snapshot-back'
  PageClick $row
  Start-Sleep -Milliseconds 7200
  Check 'D snapshot: left alone, the notice goes and the snapshot stays deleted' (-not (PageToast).on -and @(Snaps).Count -eq 1) "on=$((PageToast).on); snapshots now: $(SnapText)"
  $last = @(Snaps)[0]
  PageClick "#snapList .card[data-row-id=`"$($last.id)`"] [data-act=del]"
  Start-Sleep -Milliseconds 500
  PfQuit; Start-Sleep -Seconds 2
  PfStart -Lang 'zh-CN'
  Check 'D snapshot: quitting keeps the delete' (@(Snaps).Count -eq 0) "snapshots after a restart: $(SnapText)"
  PfQuit
}
if ($Part -match 'D') { PartD }

# ---- V: recordings for the before/after animations (--var Rec=<name>): an open in light and in
# dark mode at 30 frames per second, and four of the browser keys
function RecOpen($name) {
  RecStart $name -X 300 -Y 76 -W 1000 -H 700 -Fps 30; Start-Sleep -Milliseconds 400
  RecMark 'open the settings'
  Start-Process "$PfApp\pecofence-cli.exe" -ArgumentList 'settings', 'open-ui' -WindowStyle Hidden | Out-Null
  $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.ElapsedMilliseconds -lt 20000 -and -not (SettingsShown)) { Start-Sleep -Milliseconds 2 }
  RecMark 'the window appears'
  Start-Sleep -Milliseconds 500
  RecMark 'the page is there'
  Start-Sleep -Milliseconds 1200
  RecStop
}
function PartV() {
  trap { PfTrap $_; continue }
  Case 'V: recordings'
  CloseWindows
  PfStart -Lang 'zh-CN'
  ParkMouse
  foreach ($mode in 'light', 'dark') {
    Pf settings set theme $mode | Out-Null
    Start-Sleep -Seconds 2
    # Once unrecorded, so that the recorded open is a warm one on every build.
    Pf settings open-ui | Out-Null; WaitWin 'PecoFence.Settings' 20 | Out-Null; Start-Sleep -Seconds 2; CloseSettings; Start-Sleep -Seconds 2
    RecOpen "$Rec-open-$mode"
    Check "V an open in $mode mode is recorded" (SettingsShown) ''
    CloseSettings; Start-Sleep -Seconds 2
  }
  Pf settings set theme light | Out-Null
  Start-Sleep -Seconds 2
  $ok = PfSettings
  [G]::SetForegroundWindow((SettingsWin)) | Out-Null; Start-Sleep -Milliseconds 500
  GoToSnapshots
  RecStart "$Rec-keys" -X 0 -Y 0 -W 1300 -H 860 -Fps 8; Start-Sleep -Milliseconds 800
  foreach ($k in @(@('press F5', { [G]::Press(0x74) }), @('press F12', { [G]::Press(0x7B) }), @('press F3', { [G]::Press(0x72) }), @('press Ctrl+U', { Chord 0x11 0x55 }))) {
    [G]::SetForegroundWindow((SettingsWin)) | Out-Null; Start-Sleep -Milliseconds 300
    RecMark $k[0]
    & $k[1]
    Start-Sleep -Milliseconds 2200
    [G]::Press(0x1B); Start-Sleep -Milliseconds 300
    if ((Fg) -notmatch 'PecoFence.Settings') { Chord 0x12 0x73; Start-Sleep -Milliseconds 700 }
  }
  RecStop
  Check 'V the keys are recorded' (SettingsShown) (Fg)
  Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  CloseSettings
  PfQuit
}
if ($Part -match 'V') { PartV }
