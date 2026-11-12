# 「显示托盘图标」 (Settings › 常规 › 桌面与启动; GitHub #209 and a feedback report): the icon is
# there by default, the switch takes it out and brings it back at once, the choice holds over a
# restart of PecoFence and of Explorer, and everything the icon's menu had stays in reach with
# the icon hidden: a second start opens the settings, the page has 「退出 PecoFence」, a
# notification brings the icon for as long as it shows, and while the item menus wait to get
# their Windows part back (after a crash in a menu extension) the icon stays, because the entry
# that brings it back is in its menu.
# A debug build (build.sh --debug, run.sh --bin target/wintest-bin-debug): the page is read over
# CDP, the clicks are real.
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class TrayAt {
  [StructLayout(LayoutKind.Sequential)] public struct NII { public uint cbSize; public IntPtr hWnd; public uint uID; public Guid guidItem; }
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, R_, B; }
  [DllImport("shell32.dll")] static extern int Shell_NotifyIconGetRect(ref NII id, out R r);
  [DllImport("user32.dll")] public static extern bool RegisterHotKey(IntPtr hWnd, int id, uint mods, uint vk);
  [DllImport("user32.dll")] public static extern bool UnregisterHotKey(IntPtr hWnd, int id);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  // "result left top right bottom" of the icon of this window with this id: result 0 = on the
  // taskbar, 1 = under the "^" button (the rectangle is that button's), negative = no such icon.
  public static string Ask(IntPtr hwnd, uint id) {
    var n = new NII(); n.cbSize = (uint)Marshal.SizeOf(typeof(NII)); n.hWnd = hwnd; n.uID = id;
    R r; int hr = Shell_NotifyIconGetRect(ref n, out r);
    return hr + " " + r.L + " " + r.T + " " + r.R_ + " " + r.B;
  }
}
"@
function TrayAsk() { return @(([TrayAt]::Ask([G]::FindWindowW('PecoFence.Control', [NullString]::Value), 1)) -split ' ' | ForEach-Object { [int]$_ }) }
function TrayThere() { return ((TrayAsk)[0] -ge 0) }
# Waits until the icon is there ($true) or gone ($false); the milliseconds it took, -1 = never.
function TrayWait($want, $sec = 6) {
  $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.Elapsed.TotalSeconds -lt $sec) { if ((TrayThere) -eq $want) { return [int]$sw.ElapsedMilliseconds }; Start-Sleep -Milliseconds 100 }
  return -1
}
function TraySetting() { return "$(Pf settings get showTrayIcon)".Trim() }
function T6Log($regex) { return (@(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match $regex }) | Select-Object -Last 1) }
function T6CloseSettings() {
  $w = [G]::FindWindowW('PecoFence.Settings', [NullString]::Value)
  if ($w -ne [IntPtr]::Zero) { [TrayAt]::PostMessageW($w, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; Start-Sleep -Milliseconds 800 }
}
# A picture of the right end of the taskbar with the list of hidden icons open (the "^" button):
# Windows keeps the icon of a program it has not seen for long in that list.
function T6Corner($name) {
  $ov = [G]::FindWindowW('TopLevelWindowForOverflowXamlIsland', [NullString]::Value)
  if ($ov -eq [IntPtr]::Zero -or -not [G]::IsWindowVisible($ov)) {
    $tray = [Windows.Automation.AutomationElement]::FromHandle([G]::FindWindowW('Shell_TrayWnd', [NullString]::Value))
    $chev = @($tray.FindAll([Windows.Automation.TreeScope]::Descendants, [Windows.Automation.Condition]::TrueCondition)) | Where-Object { $_.Current.Name -match 'hidden icons|chevron' -and (OnScreen $_) } | Select-Object -First 1
    if ($chev) { $r = $chev.Current.BoundingRectangle; [G]::Click([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2)); Start-Sleep -Milliseconds 1200 }
  }
  $w = [G]::GetSystemMetrics(0); $h = [G]::GetSystemMetrics(1)
  Crop $name ($w - 360) ($h - 150) 360 150 | Out-Null
  [G]::Press(0x1B); Start-Sleep -Milliseconds 500
}
# Explorer ends and Windows starts it again: the new taskbar starts without anyone's icons, and
# PecoFence hears of it (its log line); $true when it did.
function T6Taskbars() { return @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match 'TaskbarCreated: adding the tray icon again' }).Count }
function T6RestartExplorer() {
  $n = T6Taskbars
  Get-Process explorer -ErrorAction SilentlyContinue | Stop-Process -Force
  for ($i = 0; $i -lt 80 -and (T6Taskbars) -le $n; $i++) {
    Start-Sleep -Milliseconds 500
    if ($i -eq 12 -and -not (Get-Process explorer -ErrorAction SilentlyContinue)) { Start-Process explorer.exe }
  }
  Start-Sleep -Seconds 4
  return ((T6Taskbars) -gt $n)
}
$T6Marker = "$PfApp\config\windows-menu-off.marker"

Case 'the icon is there by default'
CloseWindows
Remove-Item $T6Marker -ErrorAction SilentlyContinue
Check 'a debug build (the page is reached over CDP)' $PfDebugBuild
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
PfStart -Lang zh-CN
$started = Get-Date
Pf settings set featureTips false | Out-Null
Check 'a new configuration has showTrayIcon on, and settings get lists it' ((TraySetting) -eq 'true' -and "$(Pf settings get)" -match '"showTrayIcon":\s*true') "showTrayIcon=$(TraySetting)"
Check 'the icon is in the notification area' ((TrayWait $true) -ge 0 -and [bool](TrayIconRect)) "Shell_NotifyIconGetRect there=$(TrayThere)"
T6Corner 'corner-on'
# The config schema with the new key, for site/schema/config.json.
$schema = & "$PfApp\pecofence-cli.exe" describe --schema Config --compact | Out-String
[IO.File]::WriteAllText("$PfShots\config-schema.json", $schema.Trim(), (New-Object Text.UTF8Encoding($false)))
Check 'the config schema has showTrayIcon' ($schema -match '"showTrayIcon"') "length=$($schema.Length)"

Case 'the switch takes the icon out at once'
Check 'settings open over CDP' (PfSettings)
$card = Js "(() => { const t = document.querySelector('[data-bind=showTrayIcon]'); if (!t) return null; const c = t.closest('.card'); let h = c.closest('.group'); while (h && h.tagName !== 'H3') h = h.previousElementSibling; return { on: t.getAttribute('aria-checked'), title: c.querySelector('.t').textContent, group: h ? h.textContent : '', quit: document.getElementById('quitCard').hidden }; })()"
Check 'the page has the switch under 桌面与启动, on, without the 退出 card' ($card -and $card.on -eq 'true' -and $card.title -eq '显示托盘图标' -and $card.group -eq '桌面与启动' -and $card.quit -eq $true) "$($card | ConvertTo-Json -Compress)"
PageRect '[data-bind=showTrayIcon]' | Out-Null
Start-Sleep -Milliseconds 400
Shot 'settings-on'
PageClick '[data-bind=showTrayIcon]'
$ms = TrayWait $false
Check 'a click on the switch removes the icon' ($ms -ge 0 -and $ms -lt 1500 -and (TraySetting) -eq 'false') "gone after $ms ms; showTrayIcon=$(TraySetting)"
Check 'the app logged the decision' ([bool](T6Log 'tray icon shown or hidden.*shown=false')) "$(T6Log 'tray icon shown or hidden')"
$quit = JsWait "(() => { const q = document.getElementById('quitCard'); return !q.hidden && q.getBoundingClientRect().height > 20 ? q.innerText : null; })()" 5
Check 'the page shows 退出 PecoFence under the switch now' ("$quit" -match '退出 PecoFence' -and "$quit" -match '从这里退出') "card='$quit'"
Start-Sleep -Milliseconds 600
Shot 'settings-off'
T6Corner 'corner-off'
function T6Find($text) {
  $q = $text | ConvertTo-Json
  return @(Js "(() => { const b = document.getElementById('settingsSearch'); b.focus(); b.value = $q; b.dispatchEvent(new Event('input', { bubbles: true })); return [...document.querySelectorAll('#settingsSearchList [role=option]')].map(o => o.innerText.replace(/\s+/g, ' ')); })()")
}
foreach ($word in 'tray', '托盘', '通知区域', '任务栏图标') {
  $hits = T6Find $word
  Check "查找设置 finds the switch by '$word'" (($hits -join ' | ') -match '显示托盘图标') "$($hits -join ' | ')"
}
T6Find '' | Out-Null
T6CloseSettings

Case 'ways in without the icon'
# A second start opens the settings of the running copy (not in that copy's first 30 s: two
# autostart entries race at logon).
$wait = 32 - ((Get-Date) - $started).TotalSeconds
if ($wait -gt 0) { Start-Sleep -Seconds ([int]$wait + 1) }
Check 'the settings window is closed before the second start' ([G]::FindWindowW('PecoFence.Settings', [NullString]::Value) -eq [IntPtr]::Zero)
Start-Process "$PfApp\pecofence.exe" -WorkingDirectory $PfApp
$w = WaitWin 'PecoFence.Settings' 20
Start-Sleep -Seconds 2
Check 'starting PecoFence again opens the settings window, in one process' ($w -ne [IntPtr]::Zero -and @(Get-Process pecofence).Count -eq 1) "window=$w processes=$(@(Get-Process pecofence).Count)"
Check 'the icon stays hidden' (-not (TrayThere))
T6CloseSettings
Pf settings open-ui | Out-Null
$w = WaitWin 'PecoFence.Settings' 20
Check 'pecofence-cli settings open-ui opens it too' ($w -ne [IntPtr]::Zero)
T6CloseSettings
[G]::MoveTo(700, 450)
PeekKey
Start-Sleep -Milliseconds 1200
$peek = T6Log 'peek'
$dim = [G]::FindWindowW('PecoFence.PeekDimmer', [NullString]::Value)
Check 'the Peek shortcut (Ctrl+Alt+Space) still works' ($dim -ne [IntPtr]::Zero -and [G]::IsWindowVisible($dim)) "peek window=$dim log: $peek"
PeekKey
Start-Sleep -Milliseconds 1200

Case 'the switch brings the icon back at once'
Check 'settings open over CDP' (PfSettings)
PageClick '[data-bind=showTrayIcon]'
$ms = TrayWait $true
$gone = JsWait "document.getElementById('quitCard').hidden" 5
Check 'a click on the switch adds the icon' ($ms -ge 0 -and $ms -lt 1500 -and (TraySetting) -eq 'true') "there after $ms ms; showTrayIcon=$(TraySetting)"
Check 'the 退出 card leaves the page' ([bool]$gone)
Pf settings set showTrayIcon false | Out-Null
$ms = TrayWait $false
$off = JsWait "document.querySelector('[data-bind=showTrayIcon]').getAttribute('aria-checked') === 'false' && !document.getElementById('quitCard').hidden" 5
Check 'pecofence-cli settings set showTrayIcon false removes it and the page follows' ($ms -ge 0 -and [bool]$off) "gone after $ms ms; page follows=$off"
T6CloseSettings

Case 'the choice holds over a restart of PecoFence'
PfQuit
PfStart -Lang zh-CN
$started = Get-Date
Start-Sleep -Seconds 4
Check 'after a restart the icon is still hidden' (-not (TrayThere) -and (TraySetting) -eq 'false' -and [bool](T6Log 'tray icon at start, by the setting.*shown=false')) "there=$(TrayThere) showTrayIcon=$(TraySetting) log: $(T6Log 'tray icon at start')"

Case 'a notification brings the icon for as long as it shows'
# The Peek shortcut is taken (by this job) when PecoFence starts: it takes another one and says
# so in a notification. Windows shows one only for an icon that is in the notification area.
PfQuit
$taken = [TrayAt]::RegisterHotKey([IntPtr]::Zero, 77, 3, 0x20)
Log "Ctrl+Alt+Space registered by the job: $taken"
[G]::MoveTo(700, 450)
PfStart -Lang ''
$up = TrayWait $true 8
Shot 'notice-with-hidden-icon'
T6Corner 'corner-notice'
$said = T6Log 'hidden tray icon shown while a notification is up'
Check 'the notification goes out and the icon is there with it' ($taken -and $up -ge 0 -and [bool]$said -and [bool](T6Log 'peek hotkey taken; using fallback')) "icon after $up ms; log: $said"
$down = TrayWait $false 45
Check 'the icon leaves again when the notification is gone' ($down -ge 0 -and [bool](T6Log 'hidden tray icon taken out again')) "gone after $down ms more; log: $(T6Log 'hidden tray icon taken out again')"
[TrayAt]::UnregisterHotKey([IntPtr]::Zero, 77) | Out-Null
T6Corner 'corner-after-notice'

Case 'the way back after a crash in a menu extension'
# The marker a start writes after such a crash (tests/shellext-fault.ps1 part M makes the crash):
# the notice then says the Windows entries come back from the tray menu, so the icon is there
# until they are back, whatever the switch says.
PfQuit
New-Item -ItemType File -Force $T6Marker | Out-Null
PfStart -Lang zh-CN
Start-Sleep -Seconds 3
Check 'with the Windows part of the item menus off, the icon shows although the switch is off' ((TrayWait $true) -ge 0 -and (TraySetting) -eq 'false' -and [bool](T6Log 'item menus are built without the Windows part')) "there=$(TrayThere) showTrayIcon=$(TraySetting)"
$picked = TrayPick '恢复右键菜单里的 Windows 菜单项'
Check 'its menu has 恢复右键菜单里的 Windows 菜单项' ([bool]$picked) "menu: $script:LastTrayMenu"
[G]::Press(0x1B); Start-Sleep -Milliseconds 300
$ms = TrayWait $false
Check 'after that entry the Windows part is back and the icon is hidden again' ($ms -ge 0 -and -not (Test-Path $T6Marker) -and [bool](T6Log 'the Windows part of item menus turned back on')) "gone after $ms ms; marker left=$(Test-Path $T6Marker)"

Case 'quit from the settings page'
Check 'settings open over CDP' (PfSettings)
PageRect '#quitApp' | Out-Null
Start-Sleep -Milliseconds 300
PageClick '#quitApp'
for ($i = 0; $i -lt 50 -and (PfAlive); $i++) { Start-Sleep -Milliseconds 200 }
$log = @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match 'quitting' }) | Select-Object -Last 1
Check '退出 on the page ends PecoFence the normal way' (-not (PfAlive) -and "$log" -match 'settings page') "alive=$(PfAlive) log: $log"

Case 'the choice holds over a restart of Explorer'
PfStart -Lang zh-CN
Start-Sleep -Seconds 3
Check 'before: the switch is off and the icon hidden' ((TraySetting) -eq 'false' -and -not (TrayThere))
$again = T6RestartExplorer
Check 'Explorer made its taskbar again and the icon stays out' ($again -and -not (TrayThere)) "PecoFence heard of the new taskbar=$again; there=$(TrayThere)"
Pf settings set showTrayIcon true | Out-Null
Check 'switched on, the icon is on the new taskbar' ((TrayWait $true) -ge 0)
$again = T6RestartExplorer
Check 'and Explorer''s next taskbar gets it again' ($again -and (TrayWait $true 20) -ge 0) "PecoFence heard of the new taskbar=$again; there=$(TrayThere)"
Pf settings set showTrayIcon false | Out-Null
Check 'switched off again, it is gone' ((TrayWait $false) -ge 0)

Case 'cleanup'
Remove-Item $T6Marker -ErrorAction SilentlyContinue
[TrayAt]::UnregisterHotKey([IntPtr]::Zero, 77) | Out-Null
CloseWindows
PfQuit
Get-Process pecofence-watchdog -ErrorAction SilentlyContinue | Stop-Process -Force
