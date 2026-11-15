# 0.3.0 小工具 rework: one switch for all tools (on by default), the 新建小工具 submenu, one clock,
# 显示秒 in the clock's menu, the settings page list (新建便签, 添加, examples, 写法说明), the
# 速览与搜索 page with Peek, Everything and dialog jump, and an import that holds scripts back.
# Needs a debug build (CDP on the settings page).
Case 'setup'
CloseWindows
Set-Culture en-US
if ($PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
PfStart -Lang en -Script @('sleep 3600000')
Start-Sleep -Seconds 3
function ToolFences($kind) { return ,@(@((PfJson fence list)) | Where-Object { $_ -and $_.tool -eq $kind }) }
function FenceWindows() { return @(([L]::Tops('pecofence') -split "`n") | Where-Object { $_ -match '\| PecoFence\.Fence \|' }) }
function Tools() { return (PfJson settings get).tools }
function PlainFence() { return (@(@((PfJson fence list)) | Where-Object { $_ -and -not $_.tool }) | Select-Object -First 1) }
function FenceMenu($f) {
  $wr = $f.windowRect
  [G]::RClick([int]($wr.x + $wr.w / 2), [int]($wr.y + 14)); Start-Sleep -Milliseconds 700
  return (MenuWin 5)
}
# Opens 添加到桌面 of the open fence menu (新建便签 and 新建小工具 live there).
function AddToDesktop($h) {
  $p = MenuPoint $h '^Add to desktop'
  if (-not $p) { return $null }
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
  foreach ($m in @([W]::Tops('#32768'))) { if ((MenuTexts $m) -match '^New fence') { return $m } }
  return $null
}
# Opens the 新建小工具 submenu of the open 添加到桌面 submenu; its texts.
function ToolSubmenu($h) {
  $p = MenuPoint $h '^New tool'
  if (-not $p) { return $null }
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
  foreach ($m in @([W]::Tops('#32768'))) { if ((MenuTexts $m) -match 'Weather') { return $m } }
  return $null
}

Case 'defaults'
$t = Tools
Check 'tools.enabled is on by default' ($t.enabled -eq $true) ($t | ConvertTo-Json -Compress)
Check 'the old per-tool switches are gone' ($null -eq $t.notes -and $null -eq $t.clock -and $null -eq $t.scripts) ($t | ConvertTo-Json -Compress)

Case 'fence menu: 新建便签 and 新建小工具'
$f = PlainFence
$h = AddToDesktop (FenceMenu $f)
$texts = if ($h) { MenuTexts $h } else { '' }
Check 'the fence menu has New note and New tool (in Add to desktop)' ($texts -match 'New note' -and $texts -match 'New tool') $texts
Check 'no old New clock / New script fence items' ($texts -notmatch 'New clock' -and $texts -notmatch 'New script fence') $texts
$sub = ToolSubmenu $h
$subTexts = if ($sub) { MenuTexts $sub } else { '' }
Check 'New tool lists Clock, the three examples and Your own script' ($subTexts -match '^Clock \| Weather \| GitHub repository activity \| PC status \| Your own script') $subTexts
Check 'no To-dos (todo.txt) example' ($subTexts -notmatch 'todo') $subTexts
Shot 't01-submenu'
if ($sub) { MenuPick '^Clock$' $sub | Out-Null }
Start-Sleep -Seconds 2
Check 'Clock from the submenu makes a clock' ((ToolFences 'clock').Count -eq 1) "clocks=$((ToolFences 'clock').Count)"
CloseMenus

Case 'one clock'
$h = AddToDesktop (FenceMenu (PlainFence))
$sub = ToolSubmenu $h
$subTexts = if ($sub) { MenuTexts $sub } else { '' }
Check 'with a clock on the desktop, New tool has no Clock' ($subTexts -notmatch '^Clock' -and $subTexts -match 'Weather') $subTexts
CloseMenus
$r = & "$PfBin\pecofence-cli.exe" fence create --tool clock --title Second 2>&1 | Out-String
$code = $LASTEXITCODE
Check 'a second clock from the CLI is refused (limitReached)' ($code -ne 0 -and $r -match 'limitReached|clock already') "exit=$code $r"
Check 'still one clock' ((ToolFences 'clock').Count -eq 1)

Case 'clock menu: Show seconds'
$c = (ToolFences 'clock')[0]
$h = FenceMenu $c
$rows = @(MenuRows $h)
$sec = $rows | Where-Object { $_.text -match '^Show seconds' } | Select-Object -First 1
Check 'the clock menu has Show seconds, unticked' ($sec -and -not ($sec.state -band 8)) (MenuTexts $h)
MenuPick '^Show seconds' $h | Out-Null
Start-Sleep -Seconds 1
Check 'Show seconds turns tools.clockSeconds on' ((Tools).clockSeconds -eq $true)
$h = FenceMenu $c
# An entry with an icon shows no tick: with the seconds on it reads what a click does now.
$sec = @(MenuRows $h) | Where-Object { $_.text -match '^Hide seconds' } | Select-Object -First 1
Check 'reopened, the entry reads Hide seconds' ($null -ne $sec) (MenuTexts $h)
Shot 't02-clock-menu'
CloseMenus

Case 'settings: 小工具 page'
$ok = PfSettings
Check 'settings open over CDP' $ok
PageClick 'nav a[data-page=tools]'; Start-Sleep -Milliseconds 600
$rows = Js "[...document.querySelectorAll('#page-tools .card.child .t')].map(t => t.textContent).join(' / ')"
Check 'the list: notes, clock, weather, GitHub, PC status, own script' ($rows -eq 'Notes and to-dos / Clock / Weather / GitHub repository activity / PC status / Your own script') $rows
Check 'one switch on the page' ((Js "document.querySelectorAll('#page-tools .toggle').length") -eq 1)
$clock = Js "(() => { const b = document.getElementById('newClock'); return { disabled: b.disabled, text: b.textContent }; })()"
Check 'with a clock, its button is greyed: On the desktop' ($clock.disabled -and $clock.text -eq 'On the desktop') ($clock | ConvertTo-Json -Compress)
Shot 't03-tools-page'
PageClick '#addStatus'; Start-Sleep -Seconds 6
$st = @(ToolFences 'script' | Where-Object { $_.title -match 'status' })
Check 'Add on PC status makes its script fence' ($st.Count -eq 1) ((ToolFences 'script' | ForEach-Object { $_.title }) -join ', ')
if ($st.Count) {
  $g = PfJson fence get $st[0].id
  $s = if ($g.fence) { $g.fence.script } else { $g.script }
  Check 'PC status keeps running and prints its rows' ($s.running -and "$($s.output)" -match 'CPU') "running=$($s.running) output=$(("$($s.output)" -split "`n")[0])"
}
PageClick '#newNote'
$ed = WaitWin 'PecoFence.NoteEdit' 10
Check 'New note opens the note editor' ($ed -ne [IntPtr]::Zero)
[G]::Type('from the settings page'); [G]::Press(0x1B); Start-Sleep -Milliseconds 800
Check 'the note is made' ((ToolFences 'note').Count -eq 1)
PageClick '#scriptGuide'; Start-Sleep -Seconds 2
Check 'Write-up opens the manual lesson' ((PfLogAll) -match 'opening the manual url=https://pecofence\.jiang\.jp/manual/#write-scripts') ([regex]::Match((PfLogAll), '.*opening the manual.*').Value)
# All of Edge: on a new box its first-run window (another title) covers the settings page.
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

Case 'the one switch hides all tools and stops scripts'
$before = (FenceWindows).Count
PageClick '#page-tools .toggle[data-bind="tools.enabled"]'; Start-Sleep -Seconds 2
$off = (FenceWindows).Count
$g = PfJson fence get $st[0].id
$s = if ($g.fence) { $g.fence.script } else { $g.script }
Check 'off: the clock, the note and the script fence go' ($off -eq $before - 3) "windows before=$before off=$off"
Check 'off: the streaming script stops' (-not $s.running) "running=$($s.running)"
$h = AddToDesktop (FenceMenu (PlainFence))
$texts = if ($h) { MenuTexts $h } else { '' }
Check 'off: the fence menu has no New note / New tool' ($texts -match '^New fence' -and $texts -notmatch 'New note' -and $texts -notmatch 'New tool') $texts
CloseMenus
PageClick '#page-tools .toggle[data-bind="tools.enabled"]'; Start-Sleep -Seconds 4
Check 'on again: the three come back' ((FenceWindows).Count -eq $before) "windows=$((FenceWindows).Count)"

Case 'settings: 速览与搜索 page'
PageClick 'nav a[data-page=peek]'; Start-Sleep -Milliseconds 600
$nav = Js "[...document.querySelectorAll('nav a[data-page] .nav-text')].map(a => a.textContent.trim()).join(' / ')"
Check 'Peek and search sits right below General' ($nav -like 'General / Peek and search / *') $nav
$heads = Js "[...document.querySelectorAll('#page-peek h3')].map(h => h.textContent).join(' / ')"
Check 'its sections: Peek, Search, File dialogs' ($heads -eq 'Peek / Search / File dialogs') $heads
$has = Js "['peek.enabled','peek.dim','tools.everything','tools.dialogJump'].every(b => document.querySelector('#page-peek [data-bind=\'' + b + '\']')) && !!document.querySelector('#page-peek #dialogJumpDemo')"
Check 'Peek, dim, Everything, dialog jump and its demo are on it' $has
Shot 't04-peek-page'
PageClick '#page-peek .toggle[data-bind="tools.dialogJump"]'; Start-Sleep -Seconds 1
Check 'the dialog jump switch works from its new page' ((Tools).dialogJump -eq $true)

Case 'import holds the scripts back'
$file = 'C:\pf\run\export.json'
Pf config export $file | Out-Null
Pf config import $file | Out-Null; Start-Sleep -Seconds 3
$t = Tools
Check 'an imported config with script fences switches 小工具 off' ($t.enabled -eq $false) ($t | ConvertTo-Json -Compress)
$toast = PageToast
Check 'the notice says how to turn it back on' ($toast.text -match 'Tools switch') ($toast | ConvertTo-Json -Compress)
Pf settings set tools.enabled true | Out-Null; Start-Sleep -Seconds 2
Check 'switched on again' ((Tools).enabled -eq $true)
PfQuit
