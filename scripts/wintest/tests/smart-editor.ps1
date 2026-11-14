# 智能栅栏编辑器 (the smart fence editor, a small window of its own) on a debug build (build.sh
# --debug, run.sh --bin target/wintest-bin-debug: its page is reached over CDP) with Everything
# (see smart.ps1). Part A: every way to open it (新建智能栅栏… in the tray menu and in a fence's
# menu, a smart fence's 编辑条件…, the button on Settings › 栅栏) x every way to end it (创建 / 好,
# Enter, 取消, Esc, the close box), each on a fence of its own, read back with fence get. Part B:
# one editor at a time, Settings following a save, the live count (Everything, about a number,
# the folder read, Everything needed and started), 任一 with two folders from the folder
# picker, every kind of row, the window's size and place. --var Part=A or Part=B runs one part.
if (-not $Part) { $Part = 'AB' }

Case 'setup'
CloseWindows
DownloadsScene
foreach ($n in @('weekly.docx', 'budget.xlsx')) { AgedFile $PfDocs $n 1 | Out-Null }
Check 'a debug build (the page is reached over CDP)' $PfDebugBuild
Check 'Everything runs' (EvStart)
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
PfStart -Lang en
Start-Sleep -Seconds 2
$Dl = 'Downloads of the last 7 days'
# The app's process: the same one must still run after every save (a save that closed the
# window once crashed it: the browser's close re-entered the window's state).
$AppPid = (Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1).Id

# Opens the editor by $entry: 'tray' and 'fence menu' (新建智能栅栏…, a new fence; in the fence menu it is in 添加到桌面), 'edit menu'
# (编辑条件… of $fence) and 'settings' (the button on Settings › 栅栏 for $fence). The editor's
# window with its page connected, or zero.
function OpenEditor($entry, $fence) {
  switch ($entry) {
    'tray' { $m = TrayMenu; MenuPick '^New smart fence' $m | Out-Null }
    'fence menu' { $m = FenceMenuOf 'Programs'; PickNewSmart $m | Out-Null }
    'edit menu' { $m = FenceMenuOf $fence; MenuPick '^Edit conditions' $m | Out-Null }
    'settings' {
      if ([G]::FindWindowW('PecoFence.Settings', [NullString]::Value) -eq [IntPtr]::Zero) { PfSettings | Out-Null } else { CdpOpen | Out-Null }
      PageClick 'nav a[data-page=fences]'; Start-Sleep -Milliseconds 400
      $id = (FenceGet $fence).id
      Js "(() => { const e = document.getElementById('fenceSel'); e.value = '$id'; e.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
      Start-Sleep -Milliseconds 500
      PageClick '#smartEdit'
    }
  }
  return (EditorWin)
}
# Ends the editor by $how: 'ok' (创建 / 好), 'enter' (in the name box), 'cancel' (取消), 'esc' or
# 'close' (the close box, a real click on it). True when the window went.
function EndEditor($how) {
  switch ($how) {
    'ok' { PageClick '#seWindow .se-ok' }
    'enter' { PageClick '#seWindow .se-name'; Start-Sleep -Milliseconds 200; [G]::Press(0x0D) }
    'cancel' { PageClick '#seWindow .se-cancel' }
    'esc' { PageClick '#seWindow .se-name'; Start-Sleep -Milliseconds 200; [G]::Press(0x1B) }
    'close' {
      $h = [G]::FindWindowW('PecoFence.SmartEditor', [NullString]::Value); $r = Rect $h
      $x = $r.R - 23; $y = $r.T + 15
      Log "close box at $x,$y hit=$(NcHit $h $x $y)"
      [G]::Click($x, $y)
    }
  }
  return (WaitGoneWin 'PecoFence.SmartEditor' 10)
}
function EdState() { return Js "(() => { const w = document.getElementById('seWindow'); return { title: w.querySelector('h2').textContent, name: w.querySelector('.se-name').value, rows: [...w.querySelectorAll('.se-row')].map(r => r.dataset.kind).join(), count: (w.querySelector('.se-count') || {}).textContent || '', msg: w.querySelector('.se-msg').textContent, busy: w.querySelector('.se-count').classList.contains('busy') }; })()" }
# Waits until the count line matches $re (not while a count is on its way).
function WaitCount($re, $sec = 15) {
  for ($i = 0; $i -lt $sec * 4; $i++) { $s = EdState; if (-not $s.busy -and $s.count -match $re) { return $true }; Start-Sleep -Milliseconds 250 }
  return $false
}
# Closes the settings window like its close box (WM_CLOSE).
function CloseSettings() {
  $h = [G]::FindWindowW('PecoFence.Settings', [NullString]::Value)
  if ($h -ne [IntPtr]::Zero) { [L]::PostMessageW($h, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; WaitGoneWin 'PecoFence.Settings' 10 | Out-Null }
}
# Types $path into the folder picker's Folder: box (it has the keyboard) and presses Enter.
# Enter there opens the folder; then a click on Select Folder picks it.
function PickIn($dlg, $path) {
  [G]::SetForegroundWindow($dlg) | Out-Null; Start-Sleep -Milliseconds 300
  Chord 0x11 0x41; Start-Sleep -Milliseconds 80
  [G]::Type($path); Start-Sleep -Milliseconds 200
  [G]::Press(0x0D); Start-Sleep -Milliseconds 1200
  $b = @(([W]::Buttons($dlg) -split "`n") | Where-Object { $_ -match "`tSelect Folder`t" }) | Select-Object -First 1
  Log "folder picker: $path, button $b"
  if ($b) { $r = ($b -split "`t")[3] -split ','; [G]::Click([int](([int]$r[0] + [int]$r[2]) / 2), [int](([int]$r[1] + [int]$r[3]) / 2)) }
  WaitGoneWin '#32770' 5 | Out-Null
}
function SmartTitles() { return @(@(PfJson fence list) | Where-Object { $_ -and $_.smart } | ForEach-Object { $_.title }) }

if ($Part -match 'A') {
  $n = 0
  foreach ($entry in @('tray', 'fence menu', 'edit menu', 'settings')) {
    foreach ($how in @('ok', 'enter', 'cancel', 'esc', 'close')) {
      $n++
      $saves = $how -eq 'ok' -or $how -eq 'enter'
      Case "$entry x $how"
      $isNew = $entry -eq 'tray' -or $entry -eq 'fence menu'
      $name = "Ed $n $how"
      if (-not $isNew) {
        $fence = "Old $n"
        Pf fence create --smart downloads --title $fence --rect "$(40 + ($n % 4) * 30),$(60 + ($n % 4) * 30),420,260" | Out-Null
        WaitSmart $fence { $s.source -eq 'everything' } 20 | Out-Null
      }
      $before = SmartTitles
      $h = OpenEditor $entry $fence
      Check 'the editor opens' ($h -ne [IntPtr]::Zero -and (EditorCount) -eq 1) "h=$h count=$(EditorCount)"
      if ($h -eq [IntPtr]::Zero) { CloseWindows; continue }
      $st = EdState
      if ($isNew) {
        Check 'a new fence: 新建智能栅栏, every drive and an empty name row, the cursor in the name' ($st.title -eq 'New smart fence' -and $st.rows -eq 'where,name' -and (Js "document.activeElement.classList.contains('se-name')")) ($st | ConvertTo-Json -Compress)
        PageClick '#seWindow [data-preset=psd]'; Start-Sleep -Milliseconds 400
        PageType '#seWindow .se-name' $name
      } else {
        Check 'the fence''s search: its name and rows' ($st.title -eq 'Edit smart fence' -and $st.name -eq $fence -and $st.rows -eq 'where,sub,time,type') ($st | ConvertTo-Json -Compress)
        PageType '#seWindow .se-name' $name
        PagePick '#seWindow .se-limit select' '50'
      }
      Check 'the count follows the change' (WaitCount 'Files found now: \d+' 15) (EdState).count
      Check "$how closes the editor" (EndEditor $how) "count=$(EditorCount)"
      Start-Sleep -Milliseconds 800
      $alive = [bool](Get-Process -Id $AppPid -ErrorAction SilentlyContinue)
      $panic = @(PfLogErrors) -match 'PANIC|CRASH'
      Check 'the app keeps running, the same process, no panic' ($alive -and -not $panic) "pid=$AppPid alive=$alive $($panic -join ' | ')"
      $after = SmartTitles
      if ($isNew -and $saves) {
        $s = SmartOf $name
        Check 'a new smart fence with the name and the .psd search over every drive' ($s -and $s.spec.template -eq 'psd' -and @($s.folders).Count -eq 0 -and ((@($s.spec.allOf) | Where-Object { $_.cond -eq 'ext' }).value -join ',') -eq '.psd') "$($s.spec | ConvertTo-Json -Compress -Depth 4)"
        Check 'one fence more, nothing else' ($after.Count -eq $before.Count + 1) "$($before.Count) -> $($after.Count)"
      } elseif ($isNew) {
        Check 'nothing made' ($after.Count -eq $before.Count -and -not ($after -contains $name)) ($after -join ', ')
      } elseif ($saves) {
        $s = SmartOf $name
        Check 'the fence renamed and its search saved (limit 50), the template kept' ($s -and $s.spec.limit -eq 50 -and $s.spec.template -eq 'downloads' -and -not ($after -contains $fence)) "$($s.spec | ConvertTo-Json -Compress -Depth 4)"
      } else {
        $s = SmartOf $fence
        Check 'nothing changed' ($s -and $s.spec.limit -eq 200 -and -not ($after -contains $name)) "$($s.spec | ConvertTo-Json -Compress -Depth 4)"
      }
      $log = (PfLogAll) -join "`n"
      $want = if ($saves) { 'smart fence editor closed.*why="saved"' } elseif ($how -eq 'close') { 'smart fence editor closed by its window' } else { 'smart fence editor closed.*why="cancelled"' }
      Check 'the log says how it ended' ($log -match $want) $want
      if ($n -eq 1 -or $n -eq 13) { Shot "a$n-$($entry -replace ' ', '-')" }
    }
  }
  CloseSettings
  $log = (PfLogAll) -join "`n"
  Check 'log: opened from each entry point' ($log -match 'smart fence editor opened by="tray menu"' -and $log -match 'smart fence editor opened by="fence menu"' -and $log -match 'smart fence editor opened by="settings page"') ''
}

if ($Part -match 'B') {
  Case 'one editor at a time'
  # Part A's fences go: the menus below need the fences' title rows in the clear.
  foreach ($t in @(SmartTitles)) { Pf fence delete $t | Out-Null }
  Start-Sleep -Seconds 1
  Pf fence create --smart downloads --title 'One' --rect '40,60,420,260' | Out-Null
  Pf fence create --smart downloads --title 'Two' --rect '480,60,420,260' | Out-Null
  Start-Sleep -Seconds 2
  $h = OpenEditor 'edit menu' 'One'
  Check 'One in the editor' ((EdState).name -eq 'One') ''
  $h2 = OpenEditor 'edit menu' 'Two'
  Check 'nothing unsaved: the same window takes Two' ($h2 -eq $h -and (EditorCount) -eq 1 -and (EdState).name -eq 'Two') "h=$h h2=$h2 name=$((EdState).name)"
  PageType '#seWindow .se-name' 'Two changed'
  Start-Sleep -Milliseconds 300
  $m = FenceMenuOf 'One'; MenuPick '^Edit conditions' $m | Out-Null
  Start-Sleep -Seconds 1
  CdpOpen -Editor | Out-Null
  Check 'something unsaved: the editor keeps Two and comes to the front' ((EditorCount) -eq 1 -and (EdState).name -eq 'Two changed' -and (Fg) -match 'PecoFence.SmartEditor') "fg=$(Fg) name=$((EdState).name)"
  $m = TrayMenu; MenuPick '^New smart fence' $m | Out-Null
  Start-Sleep -Seconds 1
  CdpOpen -Editor | Out-Null
  Check 'New smart fence… too: still Two, one window' ((EditorCount) -eq 1 -and (EdState).name -eq 'Two changed') ''
  $log = (PfLogAll) -join "`n"
  Check 'log: took the other fence, then kept the unsaved one' ($log -match 'takes the other fence \(nothing unsaved\)' -and $log -match 'already open, brought to the front.*unsaved=true') ''
  EndEditor 'esc' | Out-Null
  Check 'Esc: Two keeps its name' ((SmartTitles) -contains 'Two') ((SmartTitles) -join ', ')

  Case 'the fence is deleted while its editor is open: OK makes it anew'
  Pf fence create --smart downloads --title 'Gone' --rect '480,400,420,260' | Out-Null
  Start-Sleep -Seconds 1
  $h = OpenEditor 'edit menu' 'Gone'
  PagePick '#seWindow .se-limit select' '50'
  Pf fence delete 'Gone' | Out-Null
  Start-Sleep -Milliseconds 500
  EndEditor 'ok' | Out-Null
  Start-Sleep -Seconds 1
  $s = SmartOf 'Gone'
  Check 'a new smart fence with the edited search' ($s -and $s.spec.limit -eq 50 -and @($s.folders).Count -eq 1) "$($s.spec | ConvertTo-Json -Compress -Depth 4)"
  Check 'log: deleted meanwhile, made anew' (((PfLogAll) -join "`n") -match 'the fence was deleted meanwhile; a new one is made') ''
  Pf fence delete 'Gone' | Out-Null

  Case 'a change made with the CLI while the editor is open'
  Pf fence create --smart downloads --title 'Both' --rect '480,400,420,260' | Out-Null
  Start-Sleep -Seconds 1
  $h = OpenEditor 'edit menu' 'Both'
  Pf fence set 'Both' smartWithinDays 30 | Out-Null
  $d = JsWait "(() => { const s = document.querySelector('#seWindow .se-days'); return s && s.value === '30' ? s.value : null; })()" 10
  Check 'nothing unsaved: the editor shows the new search' ($d -eq '30' -and (EdState).msg -eq '') "days=$d msg=$((EdState).msg)"
  Pf fence rename 'Both' 'Both by CLI' | Out-Null
  $nm = JsWait "(() => { const v = document.querySelector('#seWindow .se-name').value; return v === 'Both by CLI' ? v : null; })()" 10
  Check 'nothing unsaved: a rename by the CLI shows in the name box' ($nm -eq 'Both by CLI') "name=$((EdState).name)"
  PageType '#seWindow .se-name' 'Both renamed'
  Start-Sleep -Milliseconds 400
  Pf fence set 'Both by CLI' smartWithinDays 90 | Out-Null
  $m = JsWait "(() => { const t = document.querySelector('#seWindow .se-msg').textContent; return t || null; })()" 10
  Check 'something unsaved: the editor says OK would replace that change, and keeps its rows' ($m -match 'changed elsewhere' -and (Js "document.querySelector('#seWindow .se-days').value") -eq '30') "msg=$m"
  Pf fence rename 'Both by CLI' 'Both again' | Out-Null
  Start-Sleep -Milliseconds 800
  Check 'something unsaved: a rename by the CLI does not touch the name box' ((EdState).name -eq 'Both renamed') "name=$((EdState).name)"
  PagePick '#seWindow .se-limit select' '50'
  Start-Sleep -Milliseconds 500
  Check 'the notice stays after another edit' ((EdState).msg -match 'changed elsewhere') "msg=$((EdState).msg)"
  EndEditor 'ok' | Out-Null
  $s = SmartOf 'Both renamed'
  Check 'OK: the editor''s search and name' ($s -and $s.spec.withinDays -eq 30 -and $s.spec.limit -eq 50) "$($s.spec | ConvertTo-Json -Compress -Depth 4)"
  $log = (PfLogAll) -join "`n"
  Check 'log: shown again, then the notice' ($log -match 'the search changed elsewhere; shown again' -and $log -match 'the search changed elsewhere; OK would replace it') ''
  Pf fence delete 'Both renamed' | Out-Null

  Case 'no room for another fence: the editor stays open with its rows'
  $have = @(@(PfJson fence list) | Where-Object { $_ -and $_.id }).Count
  for ($i = $have; $i -lt 64; $i++) { & "$PfApp\pecofence-cli.exe" fence create --title "Fill $i" --rect "$(20 + ($i % 8) * 10),$(20 + ($i % 8) * 10),200,120" 2>&1 | Out-Null }
  Check '64 fences' (@(@(PfJson fence list) | Where-Object { $_ -and $_.id }).Count -eq 64) ''
  $h = OpenEditor 'tray'
  PageClick '#seWindow [data-preset=psd]'; Start-Sleep -Milliseconds 300
  PageClick '#seWindow .se-ok'; Start-Sleep -Seconds 1
  $st = EdState
  Check 'Create at the limit: the editor stays, says why, keeps the rows' ((EditorCount) -eq 1 -and $st.msg -match '64' -and $st.rows -eq 'where,type,ext') ($st | ConvertTo-Json -Compress)
  for ($i = $have; $i -lt 64; $i++) { & "$PfApp\pecofence-cli.exe" fence delete "Fill $i" 2>&1 | Out-Null }
  PageClick '#seWindow .se-ok'
  Check 'room again: Create makes it' ((WaitGoneWin 'PecoFence.SmartEditor' 10) -and (SmartOf 'All .psd files')) ''
  Check 'log: refused for the limit' (((PfLogAll) -join "`n") -match 'smart fence editor: not saved.*no room for another fence') ''
  Pf fence delete 'All .psd files' | Out-Null

  Case 'the window: size, place, title'
  $h = OpenEditor 'edit menu' 'One'
  $r = Rect $h
  $wa = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea
  Check 'about 880 x 660, centred on the work area' ([math]::Abs(($r.R - $r.L) - 880) -le 2 -and [math]::Abs(($r.B - $r.T) - 660) -le 2 -and [math]::Abs(($r.L + $r.R) / 2 - ($wa.Left + $wa.Width / 2)) -le 2) "rect=$($r | ConvertTo-Json -Compress) work=$($wa.Width)x$($wa.Height)"
  Check 'the caption says 编辑智能栅栏' ([W]::Title($h) -eq 'Edit smart fence') ([W]::Title($h))
  Shot 'b01-edit'

  Case 'Settings follow a save in the editor'
  EndEditor 'cancel' | Out-Null
  $h = OpenEditor 'settings' 'One'
  CdpOpen | Out-Null
  $sum0 = Js "document.getElementById('smartSummary').textContent"
  CdpOpen -Editor | Out-Null
  Js "(() => { const s = document.querySelector('#seWindow .se-row[data-kind=time] .se-days'); s.value = '30'; s.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
  Check 'the count follows' (WaitCount 'Files found now: \d+' 15) (EdState).count
  EndEditor 'ok' | Out-Null
  CdpOpen | Out-Null
  $sum1 = JsWait "(() => { const t = document.getElementById('smartSummary').textContent; return t.includes('30') ? t : null; })()" 10
  Check 'the summary on Settings › Fences changes from 7 to 30 days' ($sum0 -match 'Last 7 days' -and $sum1 -match 'Last 30 days') "before=$sum0 after=$sum1"
  Shot 'b02-settings'
  CloseSettings

  Case 'the live count'
  Pf fence create --smart downloads --rect '420,140,560,330' | Out-Null
  WaitSmart $Dl { $s.source -eq 'everything' } 20 | Out-Null
  $h = OpenEditor 'edit menu' $Dl
  Check 'Everything''s count: the six files of the week' (WaitCount 'Files found now: 6(?!\d).*Searched by Everything' 15) (EdState).count
  $asked0 = @(@(PfLogAll) | Where-Object { $_ -match 'smart fence editor: count asked' }).Count
  PageClick '#seWindow .se-row:last-child .se-plus'; Start-Sleep -Milliseconds 300
  [G]::Type('quarterly'); Start-Sleep -Milliseconds 200
  Check 'typing a name: one file' (WaitCount 'Files found now: 1(?!\d)' 15) (EdState).count
  $asked = @(@(PfLogAll) | Where-Object { $_ -match 'smart fence editor: count asked' }).Count - $asked0
  Check 'the + and nine letters typed: one or two counts (a moment after the last key)' ($asked -ge 1 -and $asked -le 2) "requests=$asked"
  Js "(() => { const s = document.querySelector('#seWindow .se-row[data-kind=name] .se-op'); s.value = 'notContains'; s.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
  Check 'does not contain: five' (WaitCount 'Files found now: 5(?!\d)' 15) (EdState).count
  EndEditor 'esc' | Out-Null
  # Idle days in a folder: the count is the files the fence then shows.
  Pf fence create --title 'Idle' --smart-folder $PfDl --rect '480,400,420,260' | Out-Null
  $h = OpenEditor 'edit menu' 'Idle'
  PageClick '#seWindow .se-row:last-child .se-plus'; Start-Sleep -Milliseconds 300
  Js "(() => { const s = [...document.querySelectorAll('#seWindow .se-kind')].at(-1); s.value = 'idle'; s.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
  Check 'idle 30 days in Downloads: the four old files (40 to 200 days)' (WaitCount 'Files found now: 4(?!\d)' 20) (EdState).count
  EndEditor 'ok' | Out-Null
  Check 'the fence shows the same four, nothing newer' (WaitSmart 'Idle' { $s.total -eq 4 -and $s.shown -eq 4 -and ($n -match 'annual') -and ($n -match 'readme') -and -not ($n -match 'quarterly|wallpaper|old contract') } 20) (SmartLine 'Idle')
  Check 'Everything narrows idle days to files written before that day' ((SmartOf 'Idle').query -match 'dm:<=\d{4}-\d\d-\d\d') (SmartOf 'Idle').query
  Pf fence delete 'Idle' | Out-Null
  # The whole computer with idle days: the count says when only the newest 5000 were checked,
  # and it is what the fence then shows.
  $h = OpenEditor 'tray'
  PageType '#seWindow .se-name' 'Idle everywhere'
  Js "(() => { const s = [...document.querySelectorAll('#seWindow .se-kind')].at(-1); s.value = 'idle'; s.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
  Check 'the whole computer with idle days: a count, and whether only the newest 5000 were checked' (WaitCount 'Files found now: \d+' 30) (EdState).count
  $line = (EdState).count
  $edN = [int]([regex]::Match($line, 'Files found now: (\d+)').Groups[1].Value)
  Check 'more than 5000 old files: it says only the newest 5000 were checked' ($edN -lt 5000 -or $line -match 'only the newest 5000 were checked') $line
  Shot 'b03-newest'
  EndEditor 'ok' | Out-Null
  Check 'the fence''s total is the same number' (WaitSmart 'Idle everywhere' { $s.source -eq 'everything' -and [math]::Abs($s.total - $edN) -le 5 } 30) "editor=$edN $(SmartLine 'Idle everywhere' | ForEach-Object { $_.Substring(0, [math]::Min(200, $_.Length)) })"
  Pf fence delete 'Idle everywhere' | Out-Null
  $h = OpenEditor 'tray'
  Js "(() => { const s = [...document.querySelectorAll('#seWindow .se-kind')].at(-1); s.value = 'idle'; s.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
  WaitCount 'Files found now: \d+' 30 | Out-Null
  Check 'Everything stops' (EvStop)
  PageType '#seWindow .se-row[data-kind=idle] .se-n1' '31'
  Check 'the whole computer without Everything: it says so and offers to start it' ((WaitCount 'needs Everything|Everything first' 20) -and (Js "!!document.querySelector('#seWindow .se-count button')")) (EdState).count
  Shot 'b04-needs-everything'
  PageClick '#seWindow .se-count button'
  for ($i = 0; $i -lt 30 -and -not (EvRunning); $i++) { Start-Sleep -Milliseconds 500 }
  Check 'the button starts Everything and the count comes' ((EvRunning) -and (WaitCount 'Files found now: \d+' 40)) (EdState).count
  EndEditor 'cancel' | Out-Null
  Check 'Everything stops again' (EvStop)
  $h = OpenEditor 'edit menu' $Dl
  Check 'a folder without Everything: PecoFence reads it' (WaitCount 'Files found now: 6(?!\d).*PecoFence reads the folders' 20) (EdState).count
  EndEditor 'esc' | Out-Null
  Check 'Everything runs again' (EvStart)
  $log = (PfLogAll) -join "`n"
  Check 'log: counted by Everything, on a worker and from the folder' ($log -match 'smart fence editor: counted by Everything' -and $log -match 'smart fence editor: counted source="everything"' -and $log -match 'smart fence editor: counted source="folder"' -and $log -match 'count shown.*newest=5000|count shown.*n=\d{1,3} ') ''

  Case '任一 over two folders from the folder picker'
  $h = OpenEditor 'tray'
  PageType '#seWindow .se-name' 'Any two'
  # 位置 › Folder opens the folder picker at once.
  Js "(() => { const s = document.querySelector('#seWindow .se-where'); s.value = 'folder'; s.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
  $dlg = $null
  for ($i = 0; $i -lt 20 -and -not $dlg; $i++) { Start-Sleep -Milliseconds 300; $dlg = @([W]::Tops('#32770') | Where-Object { [L]::Text($_) -match 'smart fence searches' }) | Select-Object -First 1 }
  Check 'Folder opens the folder picker' ([bool]$dlg) (([W]::Tops('#32770') | ForEach-Object { [L]::Text($_) }) -join ' | ')
  if ($dlg) { PickIn $dlg $PfDl }
  CdpOpen -Editor | Out-Null
  $p1 = JsWait "(() => { const p = document.querySelector('#seWindow .se-row[data-kind=where] .se-path'); return p && !p.classList.contains('none') ? p.textContent : null; })()" 10
  Check 'the row shows the picked folder' ($p1 -eq $PfDl) "$p1"
  # A second 位置 row: + then its kind, then 选择….
  PageClick '#seWindow .se-row[data-kind=where] .se-plus'; Start-Sleep -Milliseconds 300
  Js "(() => { const s = document.querySelectorAll('#seWindow .se-kind')[1]; s.value = 'where'; s.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
  PageClick '#seWindow .se-row[data-kind=where] .se-pick' 1
  $dlg = $null
  for ($i = 0; $i -lt 20 -and -not $dlg; $i++) { Start-Sleep -Milliseconds 300; $dlg = @([W]::Tops('#32770') | Where-Object { [L]::Text($_) -match 'smart fence searches' }) | Select-Object -First 1 }
  if ($dlg) { PickIn $dlg $PfDocs }
  CdpOpen -Editor | Out-Null
  JsWait "document.querySelectorAll('#seWindow .se-row[data-kind=where] .se-path:not(.none)').length === 2" 10 | Out-Null
  # The name row of the new editor: weekly; another row: extension pdf; then 任一.
  PageType '#seWindow .se-row[data-kind=name] .se-text' 'weekly'
  PageClick '#seWindow .se-row[data-kind=name] .se-plus'; Start-Sleep -Milliseconds 300
  Js "(() => { const s = [...document.querySelectorAll('#seWindow .se-kind')].at(-1); s.value = 'ext'; s.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
  PageType '#seWindow .se-row[data-kind=ext] .se-text' 'pdf'
  Check 'all: nothing is a pdf named weekly' (WaitCount 'Files found now: 0(?!\d)' 15) (EdState).count
  Js "(() => { const s = document.querySelector('#seWindow .se-all'); s.value = 'any'; s.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
  Check 'any: the PDFs of Downloads and weekly.docx of Documents' (WaitCount 'Files found now: ([3-9]|\d\d+)(?!\d)' 15) (EdState).count
  Shot 'b05-any-two-folders'
  EndEditor 'ok' | Out-Null
  $s = SmartOf 'Any two'
  Check 'saved: two folders, any, two conditions' ($s -and @($s.folders).Count -eq 2 -and $s.match -eq 'any' -and @($s.spec.allOf).Count -eq 2 -and $s.spec.match -eq 'any') ($s | ConvertTo-Json -Compress -Depth 5)
  Check 'the fence shows the PDFs and weekly.docx, nothing else' (WaitSmart 'Any two' { ($n -match 'weekly') -and ($n -match 'quarterly') -and ($n -match 'old contract') -and -not ($n -match 'budget|meeting|wallpaper') } 20) (SmartLine 'Any two')
  $q = $s.query
  Check 'one Everything search: both folders, the rows OR''ed' ($q -match '^<"[^"]+Downloads\\"\|"[^"]+Documents\\">' -and $q -match '<<"weekly">\|<ext:pdf>>') $q

  Case 'every kind of row saves as fence set reads it'
  $h = OpenEditor 'edit menu' 'Any two'
  $rows = Js "[...document.querySelectorAll('#seWindow .se-row')].map(r => r.dataset.kind).join()"
  Check 'the rows of the saved search' ($rows -eq 'where,where,sub,name,ext') $rows
  foreach ($k in @('time', 'type', 'size', 'created', 'weekday', 'target', 'idle', 'everything')) {
    PageClick '#seWindow .se-row:last-child .se-plus'; Start-Sleep -Milliseconds 200
    Js "(() => { const s = [...document.querySelectorAll('#seWindow .se-kind')].at(-1); s.value = '$k'; s.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
  }
  Js "(() => { const r = document.querySelector('#seWindow .se-row[data-kind=type] .se-cats input[value=documents]'); r.checked = true; r.dispatchEvent(new Event('change', { bubbles: true })); const w = document.querySelector('#seWindow .se-row[data-kind=weekday] .se-week input[value=\'5\']'); w.checked = true; w.dispatchEvent(new Event('change', { bubbles: true })); document.querySelector('#seWindow .se-row[data-kind=size] .se-n1').value = '2'; document.querySelector('#seWindow .se-row[data-kind=target] .se-text').value = 'steam'; document.querySelector('#seWindow .se-row[data-kind=everything] .se-text').value = 'report'; return true; })()" | Out-Null
  # 任一 with Everything syntax and rows only PecoFence tests: refused, the row is marked.
  PageClick '#seWindow .se-ok'; Start-Sleep -Milliseconds 600
  $st = EdState
  Check 'any with Everything syntax and a row PecoFence tests: not saved, the row named' ((EditorCount) -eq 1 -and $st.msg -match 'Everything syntax cannot go with this row' -and (Js "['created', 'weekday', 'target', 'idle'].includes((document.querySelector('#seWindow .se-row.bad') || {}).dataset?.kind)")) ($st | ConvertTo-Json -Compress)
  Js "(() => { const s = document.querySelector('#seWindow .se-all'); s.value = 'all'; s.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
  EndEditor 'ok' | Out-Null
  $s = SmartOf 'Any two'
  $conds = (@($s.spec.allOf) | ForEach-Object { $_.cond }) -join ','
  Check 'all: fence get has every condition, the window and the Everything text' ($conds -eq 'name,ext,type,sizeMb,createdTime,createdWeekday,shortcutTarget,idleDays' -and $s.spec.withinDays -eq 7 -and $s.spec.everything -eq 'report' -and $s.match -eq 'all') "$conds days=$($s.spec.withinDays) ev=$($s.spec.everything)"
  $log = (PfLogAll) -join "`n"
  Check 'log: the whole search saved' ($log -match 'smart fence search changed.*what="the whole search".*matching="all"') ''
  Pf fence delete 'Any two' | Out-Null
}

Case 'cleanup'
PfQuit
foreach ($n in @('weekly.docx', 'budget.xlsx')) { Remove-Item -LiteralPath (Join-Path $PfDocs $n) -ErrorAction SilentlyContinue }
ClearDownloads
Check 'Everything still runs for the next job' (EvStart) ''
