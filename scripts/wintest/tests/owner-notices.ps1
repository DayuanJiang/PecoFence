# Owner round 2026-11-12, package notices. Runs on a 1600x900 box at 100 %.
#   run.sh --var Part=F ...   first run: one gap between the default fences (FR-1); 「下一条」 keeps
#                             the tip card on screen (NT-2)
#   run.sh --var Part=G ...   a tool, a fence and a tray fence made from a menu and a fence drawn
#                             beside another one keep the snapping gap, edges in line; the example
#                             script fence is titled as its menu line (FR-1, TL-4)
#   run.sh --var Part=N ...   a notification names its source 「PecoFence」 (NT-1). Needs a build
#                             with a version resource (CI, or PECOFENCE_RC=/usr/bin/llvm-rc on the
#                             dev box); with --extra <dir holding old\pecofence.exe> it also
#                             updates that older exe in place.
# Without --var all three run.
if (-not $Part) { $Part = 'FGN' }
$OnGap = 8   # snapping.gapPx by default, at 100 %
$OnFiles = @('预算表 2026.xlsx', '会议纪要 11月.txt', '年度报告 2025 董事会审阅版（含附录）.pdf', '报告.txt', 'a.md')
$OnDirs = @('项目资料', '报告归档')
$OnLinks = @('记事本.lnk', '计算器（常用）.lnk')
function OnClean() {
  foreach ($n in @($OnFiles) + @($OnDirs) + @($OnLinks) + @('on-link.lnk')) {
    $p = Join-Path $PfDesk $n
    if (Test-Path -LiteralPath $p) { Remove-Item -LiteralPath $p -Recurse -Force -ErrorAction SilentlyContinue }
  }
}
function OnSeed() {
  OnClean
  foreach ($n in $OnFiles) { DeskFile $n | Out-Null }
  # WScript.Shell cannot save a shortcut under a Chinese name: make it, then rename it.
  foreach ($n in $OnLinks) { $tmp = DeskLink 'on-link.lnk'; Rename-Item -LiteralPath $tmp -NewName $n }
  foreach ($n in $OnDirs) { New-Item -ItemType Directory -Force (Join-Path $PfDesk $n) | Out-Null }
}
function OnNotice() { $n = [G]::FindWindowW('PecoFence.Notice', [NullString]::Value); if ($n -ne [IntPtr]::Zero -and [G]::IsWindowVisible($n)) { return $n }; return [IntPtr]::Zero }
function OnFences() { return @(@(PfJson fence list) | Where-Object { $_ -and $_.title -and $_.windowRect }) }
function OnFence($title) { return (OnFences | Where-Object { $_.title -eq $title } | Select-Object -First 1) }
function OnRectText($f) { if (-not $f) { return 'none' }; $w = $f.windowRect; return "'$($f.title)' $($w.x),$($w.y),$($w.w)x$($w.h)" }
# How `b` sits beside `a`: the gap between the facing edges and how far the edges across are off
# (a fence right of or left of `a`: tops; one below or above: left edges).
function OnBeside($a, $b) {
  $p = $a.windowRect; $q = $b.windowRect
  $dx = [Math]::Max($q.x - ($p.x + $p.w), $p.x - ($q.x + $q.w))
  $dy = [Math]::Max($q.y - ($p.y + $p.h), $p.y - ($q.y + $q.h))
  if ($dx -ge 0 -and $dx -ge $dy) { return @{ gap = $dx; off = [Math]::Abs($q.y - $p.y); side = 'beside' } }
  return @{ gap = $dy; off = [Math]::Abs($q.x - $p.x); side = 'below or above' }
}
# The fence menu of `f` (a right-click on its title row), then 添加到桌面 and one of its lines.
function OnAddToDesktop($f, $line, $sub = '') {
  [G]::RClick($f.windowRect.x + 120, $f.windowRect.y + 14); Start-Sleep -Milliseconds 900
  $m = MenuWin 4
  $p = MenuPoint $m '^(Add to desktop|添加到桌面)'
  if (-not $p) { Log "no 添加到桌面 in: $(MenuTexts $m)"; CloseMenus; return $null }
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900
  $menus = @([W]::Tops('#32768'))
  if ($sub) {
    $s = $menus | Where-Object { @(MenuRows $_ | Where-Object { $_.text -match $sub }).Count -gt 0 } | Select-Object -First 1
    $p = if ($s) { MenuPoint $s $sub } else { $null }
    if (-not $p) { Log "no $sub in the open menus"; CloseMenus; return $null }
    [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900
    $menus = @([W]::Tops('#32768'))
  }
  $s = $menus | Where-Object { @(MenuRows $_ | Where-Object { $_.text -match $line }).Count -gt 0 } | Select-Object -First 1
  if (-not $s) { Log "no $line in the open menus"; CloseMenus; return $null }
  $picked = MenuPick $line $s
  Start-Sleep -Milliseconds 1500
  # A new fence opens its name box: Enter keeps the name.
  if ([G]::FindWindowW('PecoFence.Rename', [NullString]::Value) -ne [IntPtr]::Zero) { [G]::Press(0x0D); Start-Sleep -Milliseconds 600 }
  return $picked
}
# The fence that was not there before.
function OnNewFence($before) { $ids = @($before | ForEach-Object { $_.id }); return (OnFences | Where-Object { $ids -notcontains $_.id } | Select-Object -First 1) }

function OnPartF() {
  trap { PfTrap $_; continue }
  Case 'first run: one gap between the default fences'
  OnSeed
  $anim = [Anim]::Get(); [Anim]::Set($true)
  if (PfAlive) { PfQuit }
  Remove-Item -Recurse -Force "$PfApp\config", "$PfApp\notices.json" -ErrorAction SilentlyContinue
  if (-not (Test-Path "$PfApp\pecofence.exe")) { New-Item -ItemType Directory -Force $PfApp | Out-Null; Copy-Item "$PfBin\*" $PfApp -Force }
  $env:PECOFENCE_INSTANCE = $PfInst
  $env:RUST_LOG = 'pecofence=debug'
  [G]::MoveTo(700, 450)
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $PfApp
  WaitWin 'PecoFence.Control' 20 -Hidden | Out-Null
  $n = WaitWin 'PecoFence.Notice' 30
  Start-Sleep -Seconds 4
  Shot 'f-first-run'
  $wa = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea
  $col = @(OnFences | Where-Object { $_.kind -ne 'inbox' } | Sort-Object { $_.windowRect.y })
  Log "first run fences: $(($col | ForEach-Object { OnRectText $_ }) -join ' | ')"
  $gaps = @(); for ($i = 1; $i -lt $col.Count; $i++) { $gaps += $col[$i].windowRect.y - $col[$i - 1].windowRect.y - $col[$i - 1].windowRect.h }
  $right = if ($col.Count) { $wa.Right - $col[0].windowRect.x - $col[0].windowRect.w } else { -1 }
  $top = if ($col.Count) { $col[0].windowRect.y - $wa.Top } else { -1 }
  $oneColumn = @($col | ForEach-Object { $_.windowRect.x } | Select-Object -Unique).Count -eq 1
  Check 'the three default fences stand in one column' ($col.Count -eq 3 -and $oneColumn) "fences=$($col.Count) $(($col | ForEach-Object { OnRectText $_ }) -join ' | ')"
  Check "the default fences are $OnGap px apart (the snapping gap)" ($gaps.Count -eq 2 -and @($gaps | Where-Object { $_ -ne $OnGap }).Count -eq 0) "gaps=$($gaps -join ',')"
  Check "the column is $OnGap px from the top and the right edge of the work area" ($right -eq $OnGap -and $top -eq $OnGap) "right=$right top=$top"
  if ($col.Count -ge 2) { Crop 'f-first-run-gap' ($col[0].windowRect.x - 40) ($col[0].windowRect.y + $col[0].windowRect.h - 40) 200 100 | Out-Null }

  Case 'tips: the card stays through Next'
  $n = OnNotice
  Check 'the first tip shows' ($n -ne [IntPtr]::Zero) "notice=$n"
  $rows = @()
  $same = 0; $missing = 0; $bottoms = 0; $titles = @([L]::Text($n))
  foreach ($k in 1..3) {
    $n0 = OnNotice; $r0 = Rect $n0
    # The pointer rests on the button; a point of the card's plate near its bottom-left corner.
    [G]::MoveTo($r0.R - 46, $r0.B - 32); Start-Sleep -Milliseconds 500
    [Px]::Burst($r0.L + 10, $r0.B - 14, 6, 6, 700, 1)
    $first = [Px]::Frames[0].GetPixel(3, 3)
    $gone = 0; $goneAt = @()
    for ($i = 1; $i -lt [Px]::Frames.Count; $i++) {
      $c = [Px]::Frames[$i].GetPixel(3, 3)
      $d = [Math]::Max([Math]::Abs($c.R - $first.R), [Math]::Max([Math]::Abs($c.G - $first.G), [Math]::Abs($c.B - $first.B)))
      if ($d -gt 10) { $gone++; $goneAt += [Px]::Times[$i] }
    }
    Start-Sleep -Milliseconds 500
    $n1 = OnNotice; $r1 = Rect $n1
    $titles += [L]::Text($n1)
    if ($n1 -eq $n0) { $same++ }
    if ($gone -gt 0) { $missing++ }
    if ($r1.B -eq $r0.B -and $r1.T -eq $r0.T) { $bottoms++ }
    $rows += "next ${k}: window $n0 -> $n1, top $($r0.T) -> $($r1.T), bottom $($r0.B) -> $($r1.B), frames $([Px]::Frames.Count), plate changed in $gone ($(($goneAt | Select-Object -First 1)) to $(($goneAt | Select-Object -Last 1)) ms)"
    Shot "f-tip-$($k + 1)"
  }
  Log ("tips:`n" + ($rows -join "`n"))
  Log "tip titles: $($titles -join ' | ')"
  Check 'Next shows the next tip (four different titles)' (@($titles | Select-Object -Unique).Count -eq 4) ($titles -join ' | ')
  Check 'the same card window stays through three Next clicks' ($same -eq 3) ($rows -join ' | ')
  Check 'the card''s plate never leaves the screen during a Next (700 ms of frames each)' ($missing -eq 0) ($rows -join ' | ')
  Check 'the card keeps its place and its size (one height for the four tips)' ($bottoms -eq 3) ($rows -join ' | ')
  $heights = @($rows | ForEach-Object { if ($_ -match 'top (\d+) -> (\d+)') { [int]$Matches[2] - [int]$Matches[1] } })
  Log "top edge moved by: $($heights -join ',') px"
  Check 'the app log names the swap' (@(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match 'next tip on the same card' }).Count -eq 3) ''
  # The last tip's button opens the manual in a browser: close the card with its x instead.
  $r = Rect (OnNotice); [G]::Click($r.R - 24, $r.T + 24); Start-Sleep -Milliseconds 300; [G]::MoveTo(700, 450)
  Check 'x closes the last tip' (WaitGoneWin 'PecoFence.Notice' 3) ''
  [Anim]::Set($anim)
  PfQuit
}

function OnPartG() {
  trap { PfTrap $_; continue }
  Case 'made from a fence menu: the snapping gap, edges in line'
  OnSeed
  PfStart
  ClearFences | Out-Null
  Pf fence create --title 'Files' --rect '360,400,300,200' | Out-Null
  Pf fence create --title 'Notes' --rect '360,100,300,200' | Out-Null
  Start-Sleep -Seconds 1
  $files = OnFence 'Files'; $notes = OnFence 'Notes'
  Log "sources: $(OnRectText $files) | $(OnRectText $notes)"

  $before = OnFences
  $picked = OnAddToDesktop $files '^(PC status|电脑状态)' '^(New tool|新建小工具)'
  $tool = OnNewFence $before
  $b = if ($tool) { OnBeside $files $tool } else { @{ gap = -1; off = -1; side = '' } }
  Shot 'g-tool-from-menu'
  Check 'a tool from a fence menu: PC status made' ($null -ne $tool -and $tool.tool -eq 'script') "picked=$picked $(OnRectText $tool) tool=$($tool.tool)"
  Check "the tool stands $OnGap px beside the fence it was made from, tops in line" ($b.gap -eq $OnGap -and $b.off -eq 0) "gap=$($b.gap) off=$($b.off) $($b.side); $(OnRectText $files) | $(OnRectText $tool)"
  Check 'the example is titled as its menu line, not as its file' ($tool -and $tool.title -eq 'PC status') "title='$($tool.title)'"
  if ($tool) { Crop 'g-tool-gap' ($files.windowRect.x + $files.windowRect.w - 150) ($files.windowRect.y - 30) 360 130 | Out-Null; Pf fence delete $tool.id | Out-Null; Start-Sleep -Milliseconds 800 }

  $before = OnFences
  $picked = OnAddToDesktop $files '^(New fence|新建栅栏)'
  $made = OnNewFence $before
  $b = if ($made) { OnBeside $files $made } else { @{ gap = -1; off = -1; side = '' } }
  Shot 'g-fence-from-menu'
  Check "a fence from a fence menu stands $OnGap px beside its source, tops in line" ($b.gap -eq $OnGap -and $b.off -eq 0) "picked=$picked gap=$($b.gap) off=$($b.off) $($b.side); $(OnRectText $files) | $(OnRectText $made)"
  if ($made) { Crop 'g-fence-gap' ($files.windowRect.x + $files.windowRect.w - 150) ($files.windowRect.y - 30) 360 130 | Out-Null }

  Case 'drawn beside a fence: the snapping gap, tops in line'
  $before = OnFences
  $x = $notes.windowRect.x + $notes.windowRect.w + 3; $y = $notes.windowRect.y + 5
  DragTo $x $y ($x + 260) ($y + 170) 12
  $o = [IntPtr]::Zero
  for ($i = 0; $i -lt 40 -and $o -eq [IntPtr]::Zero; $i++) { $w = [G]::FindWindowW('PecoFence.MarqueeOffer', [NullString]::Value); if ($w -ne [IntPtr]::Zero -and [G]::IsWindowVisible($w)) { $o = $w } else { Start-Sleep -Milliseconds 100 } }
  if ($o -ne [IntPtr]::Zero) {
    $r = Rect $o; $bx = $r.R - 30; $by = [int](($r.T + $r.B) / 2)
    Glide ($bx - 50) ($by + 30) $bx $by 8 25; Start-Sleep -Milliseconds 200
    [G]::Click($bx, $by); Start-Sleep -Milliseconds 1500
    [G]::Press(0x0D); Start-Sleep -Milliseconds 700
  }
  $drawn = OnNewFence $before
  $b = if ($drawn) { OnBeside $notes $drawn } else { @{ gap = -1; off = -1; side = '' } }
  Shot 'g-drawn-fence'
  Check 'the rectangle drawn 3 px beside a fence offered a new fence' ($o -ne [IntPtr]::Zero -and $null -ne $drawn) "offer=$o drawn at $x,$y; $(OnRectText $drawn)"
  Check "the drawn fence stands $OnGap px beside that fence, tops in line" ($b.gap -eq $OnGap -and $b.off -eq 0) "gap=$($b.gap) off=$($b.off) $($b.side); $(OnRectText $notes) | $(OnRectText $drawn)"
  if ($drawn) { Crop 'g-drawn-gap' ($notes.windowRect.x + $notes.windowRect.w - 150) ($notes.windowRect.y - 30) 360 130 | Out-Null }

  Case 'made from the tray twice: the second one beside the first'
  $before = OnFences
  $p1 = TrayPick '^(New fence|新建栅栏)$'
  Start-Sleep -Milliseconds 1500; [G]::Press(0x0D); Start-Sleep -Milliseconds 700
  $t1 = OnNewFence $before
  $before = OnFences
  $p2 = TrayPick '^(New fence|新建栅栏)$'
  Start-Sleep -Milliseconds 1500; [G]::Press(0x0D); Start-Sleep -Milliseconds 700
  $t2 = OnNewFence $before
  $b = if ($t1 -and $t2) { OnBeside $t1 $t2 } else { @{ gap = -1; off = -1; side = '' } }
  Shot 'g-tray-fences'
  Check 'two fences made from the tray menu' ($null -ne $t1 -and $null -ne $t2) "picked=$p1,$p2 $(OnRectText $t1) | $(OnRectText $t2)"
  Check "the second tray fence stands $OnGap px from the first, an edge in line" ($b.gap -eq $OnGap -and $b.off -eq 0) "gap=$($b.gap) off=$($b.off) $($b.side); $(OnRectText $t1) | $(OnRectText $t2)"
  Check 'the app log names how each new fence was placed' (@(PfLogAll | Where-Object { $_ -match 'new fence placed' }).Count -ge 4) ''

  Case 'the gap follows the setting'
  if ($made) { Pf fence delete $made.id | Out-Null; Start-Sleep -Milliseconds 800 }
  Pf settings set snapping.gapPx 16 | Out-Null
  Start-Sleep -Milliseconds 600
  $before = OnFences
  $picked = OnAddToDesktop $files '^(New fence|新建栅栏)'
  $wide = OnNewFence $before
  $b = if ($wide) { OnBeside $files $wide } else { @{ gap = -1; off = -1; side = '' } }
  Check 'with the gap set to 16, a fence from a fence menu stands 16 px beside its source' ($b.gap -eq 16 -and $b.off -eq 0) "picked=$picked gap=$($b.gap) off=$($b.off) $($b.side); $(OnRectText $files) | $(OnRectText $wide)"
  Pf settings set snapping.gapPx 8 | Out-Null
  PfQuit
}

function OnPartN() {
  trap { PfTrap $_; continue }
  Case 'a notification names its source PecoFence'
  # Windows files a tray icon's notices under an app id it makes for the exe's path
  # (HKCU\Software\Classes\AppUserModelId\NotifyIconGeneratedAumid_<n>, DisplayName = the exe's
  # file description, or its file name without one).
  function OnAumids() {
    $out = @{}
    foreach ($k in @(Get-ChildItem 'HKCU:\Software\Classes\AppUserModelId' -ErrorAction SilentlyContinue)) {
      $out[$k.PSChildName] = "$((Get-ItemProperty $k.PSPath -ErrorAction SilentlyContinue).DisplayName)"
    }
    return $out
  }
  # Starts the exe in $PfApp on a configuration it cannot read: its first notification says so.
  function OnStartUnreadable() {
    if (PfAlive) { PfQuit }
    Remove-Item -Recurse -Force "$PfApp\config" -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force "$PfApp\config" | Out-Null
    [IO.File]::WriteAllText("$PfApp\config\config.json", '{ "layouts": this is not json')
    [G]::MoveTo(700, 450)
    PfStart
  }
  # The names Windows files PecoFence's notices under now, but for the ones in $known.
  function OnNewNames($known) { $now = OnAumids; return @($now.Keys | Where-Object { -not $known.ContainsKey($_) -or $known[$_] -ne $now[$_] } | ForEach-Object { "$_=$($now[$_])" }) }
  $desc = (Get-Item "$PfBin\pecofence.exe").VersionInfo.FileDescription
  Log "build under test: FileDescription='$desc' ProductName='$((Get-Item "$PfBin\pecofence.exe").VersionInfo.ProductName)'"
  Check 'the build carries a version resource (a CI build, or PECOFENCE_RC on the dev box)' ([bool]$desc) "FileDescription='$desc'"
  Check 'the exe''s file description is PecoFence' ($desc -ceq 'PecoFence') "FileDescription='$desc'"
  $wd = (Get-Item "$PfBin\pecofence-watchdog.exe").VersionInfo.FileDescription
  Check 'the watchdog keeps a description that tells it apart' ($wd -match 'watchdog') "FileDescription='$wd'"

  # App ids of earlier jobs' test folders go first (Settings lists every one of them).
  function OnDropAumids() {
    foreach ($k in @(Get-ChildItem 'HKCU:\Software\Classes\AppUserModelId' -ErrorAction SilentlyContinue)) {
      $name = "$((Get-ItemProperty $k.PSPath -ErrorAction SilentlyContinue).DisplayName)"
      if ($k.PSChildName -like 'NotifyIconGeneratedAumid_*' -and $name -match '^PecoFence|^pecofence\.exe$') { Remove-Item $k.PSPath -Recurse -Force; Log "app id $($k.PSChildName) ($name) removed" }
    }
  }
  ClearPfNotices
  OnDropAumids
  # A folder Windows has not seen an exe in.
  $script:PfApp = "$PfRun\app-n$(Get-Random -Minimum 1000 -Maximum 9999)"
  New-Item -ItemType Directory -Force $PfApp | Out-Null
  Copy-Item "$PfBin\*" $PfApp -Force
  $known = OnAumids
  OnStartUnreadable
  $toast = ToastOcr 'PecoFence' 10
  Crop 'n-toast-new-folder' 1180 620 420 232 | Out-Null
  Start-Sleep -Seconds 1
  $names = OnNewNames $known
  Log "new folder: toast OCR: $($toast -replace '\s+', ' ')"
  Log "new folder: app ids: $($names -join ' | ')"
  Check 'the app showed its notification (the configuration file could not be read)' (@(PfLogAll | Where-Object { $_ -match 'unreadable|could not be read|moved' }).Count -ge 1 -and $toast -match 'PecoFence') "toast: $($toast -replace '\s+', ' ')"
  Check 'Windows files the notices under the name PecoFence' (@($names | Where-Object { $_ -cmatch '=PecoFence$' }).Count -eq 1 -and @($names | Where-Object { $_ -match 'group your desktop' }).Count -eq 0) ($names -join ' | ')
  Check 'the notification on screen does not show the cut-off sentence' ($toast -notmatch 'group your|desktop icons into') "toast: $($toast -replace '\s+', ' ')"
  # Settings > System > Notifications lists the senders under the same name. (The notification
  # centre keeps no tray notices: nothing to read there.)
  CloseToast
  Start-Process 'ms-settings:notifications'; Start-Sleep -Seconds 7
  $st = OcrText 0 0 1600 860 'n-settings'
  Shot 'n-settings-notifications'
  Get-Process SystemSettings -ErrorAction SilentlyContinue | Stop-Process -Force
  Log "Settings > Notifications OCR: $(($st -replace '\s+', ' ') -replace '^(.{0,900}).*', '$1')"
  Check 'Settings > Notifications lists the sender as PecoFence' ($st -match 'PecoFence' -and $st -notmatch 'group your|desktop icons into') "$(($st -replace '\s+', ' ') -replace '^.*?(.{0,80}PecoFence.{0,80}).*$', '$1')"
  PfQuit

  $old = "$PfExtra\old\pecofence.exe"
  if (Test-Path $old) {
    Case 'an update in place: the older exe, then this build in the same folder'
    $script:PfApp = "$PfRun\app-u$(Get-Random -Minimum 1000 -Maximum 9999)"
    New-Item -ItemType Directory -Force $PfApp | Out-Null
    Copy-Item "$PfBin\*" $PfApp -Force
    Copy-Item $old "$PfApp\pecofence.exe" -Force
    Log "older exe: FileDescription='$((Get-Item "$PfApp\pecofence.exe").VersionInfo.FileDescription)'"
    $known = OnAumids
    OnStartUnreadable
    $toastOld = ToastOcr 'PecoFence' 10
    Crop 'n-toast-older-exe' 1180 620 420 232 | Out-Null
    Start-Sleep -Seconds 1
    $namesOld = OnNewNames $known
    Log "older exe: toast OCR: $($toastOld -replace '\s+', ' ')"
    Log "older exe: app ids: $($namesOld -join ' | ')"
    Check 'the older exe''s notices are filed under its sentence' (@($namesOld | Where-Object { $_ -match 'group your desktop' }).Count -eq 1) ($namesOld -join ' | ')
    CloseToast
    PfQuit
    ClearPfNotices
    Copy-Item "$PfBin\pecofence.exe" "$PfApp\pecofence.exe" -Force
    OnStartUnreadable
    $toastNew = ToastOcr 'PecoFence' 10
    Crop 'n-toast-updated-exe' 1180 620 420 232 | Out-Null
    Start-Sleep -Seconds 1
    $id = @($namesOld | Where-Object { $_ -match 'group your desktop' } | ForEach-Object { ($_ -split '=', 2)[0] }) | Select-Object -First 1
    $now = OnAumids
    Log "updated exe: toast OCR: $($toastNew -replace '\s+', ' ')"
    Log "updated exe: app id $id is now '$($now[$id])'; changed since: $((OnNewNames $known) -join ' | ')"
    Check 'after the update Windows files the notices under PecoFence' (@((OnNewNames $known) | Where-Object { $_ -match 'group your desktop' }).Count -eq 0 -and @((OnNewNames $known) | Where-Object { $_ -cmatch '=PecoFence$' }).Count -ge 1) ((OnNewNames $known) -join ' | ')
    Check 'the notification after the update does not show the cut-off sentence' ($toastNew -match 'PecoFence' -and $toastNew -notmatch 'group your|desktop icons into') "toast: $($toastNew -replace '\s+', ' ')"
    CloseToast
    PfQuit
  }
  ClearPfNotices
  OnDropAumids
}

OnClean
if ($Part -match 'F') { OnPartF }
if ($Part -match 'G') { OnPartG }
if ($Part -match 'N') { OnPartN }

Case 'cleanup'
if (PfAlive) { PfQuit }
OnClean
Check 'test files removed from the desktop' (@(@($OnFiles) + @($OnDirs) + @($OnLinks) | Where-Object { Test-Path -LiteralPath (Join-Path $PfDesk $_) }).Count -eq 0) ''
