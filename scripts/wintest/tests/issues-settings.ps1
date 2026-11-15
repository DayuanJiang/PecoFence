# 菜单颜色 (GitHub #227) and 应用到所有栅栏 (feedback #135, #159).
# A: the popup menus follow 颜色模式 unless 菜单颜色 picks light or dark (the item menu with its
#    Windows entries and the fence menu, judged by their mean colour; the command line and the
#    settings page set it).
# B: the end of 布局 and of 外观 on the 栅栏 page gives one fence's options to every other fence,
#    leaves 「桌面」 arranged like the desktop alone, and the notice's 撤销 sets every fence back.
# A debug build (the page is read over CDP, the clicks are real):
#   run.sh --bin target/wintest-bin-debug scripts/wintest/tests/issues-settings.ps1
# On main's debug build the page has neither and the command line refuses menuColor.

Add-Type 'using System; using System.Runtime.InteropServices; public static class SW { [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int c); [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h, uint m, IntPtr w, IntPtr l); }'
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
$script:MenuN = 0
# The open popup menu's mean colour: 'dark', 'light' or 'none', with the crop saved.
function IsMenu() {
  $m = MenuWin 6
  if ($m -eq [IntPtr]::Zero) { return 'none' }
  Start-Sleep -Milliseconds 450
  $r = Rect $m
  $script:MenuN++
  $rgb = MeanRgb (Crop "menu-$($script:MenuN)" $r.L $r.T ($r.R - $r.L) ($r.B - $r.T))
  $mean = ($rgb[0] + $rgb[1] + $rgb[2]) / 3
  Log "menu $($script:MenuN): mean $($rgb -join ' ') at $($r.L),$($r.T) $($r.R - $r.L)x$($r.B - $r.T)"
  CloseMenus
  if ($mean -lt 110) { return 'dark' }
  if ($mean -gt 170) { return 'light' }
  return "between ($([int]$mean))"
}
# The item menu (with its Windows part) and the fence menu of the fence that holds the test file.
function ItemMenu() {
  $p = ItemPt $script:Fence $script:Item
  if (-not $p) { return 'no item' }
  ParkMouse
  [G]::RClick($p[0], $p[1])
  return (IsMenu)
}
function FenceMenu() {
  $f = FenceGet $script:Fence
  ParkMouse
  [G]::RClick([int]($f.windowRect.x + 40), [int]($f.windowRect.y + 14))
  return (IsMenu)
}
# A picture of the fence with its item menu open: the fence and the menu beside it.
# (The command line gets no answer while a menu is open: the fence is read first.)
function MenuScene($name) {
  $p = ItemPt $script:Fence $script:Item
  $f = FenceGet $script:Fence
  ParkMouse
  [G]::RClick($p[0], $p[1])
  $m = MenuWin 6
  Start-Sleep -Milliseconds 450
  $r = Rect $m
  $x = [math]::Max(0, [math]::Min($f.windowRect.x, $r.L) - 20); $y = [math]::Max(0, [math]::Min($f.windowRect.y, $r.T) - 20)
  $x2 = [math]::Min(1600, [math]::Max($f.windowRect.x + $f.windowRect.w, $r.R) + 20); $y2 = [math]::Min(900, [math]::Max($f.windowRect.y + $f.windowRect.h, $r.B) + 20)
  Crop $name $x $y ($x2 - $x) ($y2 - $y) | Out-Null
  CloseMenus
}
function Setting($name) { return "$(Pf settings get $name)".Trim().Trim('"') }
function NoHash($v) { return "$v".TrimStart('#').ToUpperInvariant() }

if (-not $Part -or $Part -match 'A') {
  Case 'menu colours follow the colour mode or 菜单颜色'
  PfStart -Lang zh-CN
  $script:FileName = 'menu-colour-check.txt'
  $p = DeskFile $script:FileName
  Start-Sleep -Seconds 3
  $it = @(TJson item list) | Where-Object { $_ -and $_.fileName -eq $script:FileName } | Select-Object -First 1
  $script:Item = $it.name; $script:Fence = $it.fenceTitle
  Check 'the test file is in a fence' ([bool]$script:Fence) "item=$($script:Item) fence=$($script:Fence)"
  Pf settings set theme dark | Out-Null
  Start-Sleep -Seconds 2
  $m = ItemMenu
  Check 'dark mode: the item menu is dark' ($m -eq 'dark') $m
  MenuScene 'scene-dark-mode-default'
  $out = "$(Pf settings set menuColor light 2>&1)"
  Check 'the command line takes menuColor light' ((Setting menuColor) -eq 'light') $out
  Start-Sleep -Milliseconds 800
  $m = ItemMenu
  Check 'dark mode with 菜单颜色 浅色: the item menu is light' ($m -eq 'light') $m
  $m = FenceMenu
  Check 'dark mode with 菜单颜色 浅色: the fence menu is light' ($m -eq 'light') $m
  MenuScene 'scene-dark-mode-light-menus'
  Pf settings set theme light | Out-Null
  Start-Sleep -Seconds 2
  $m = ItemMenu
  Check 'light mode with 菜单颜色 浅色: the item menu stays light' ($m -eq 'light') $m
  Pf settings set menuColor dark | Out-Null
  Start-Sleep -Milliseconds 800
  $m = ItemMenu
  Check 'light mode with 菜单颜色 深色: the item menu is dark' ($m -eq 'dark') $m
  Pf settings set menuColor followMode | Out-Null
  Start-Sleep -Milliseconds 800
  $m = ItemMenu
  Check 'followMode in light mode: the item menu is light again' ($m -eq 'light') $m
  $line = @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match 'menu colours set' }) | Select-Object -Last 1
  Check 'the log says how the menus were set' ([bool]$line) "$line"

  Case 'the settings page offers 菜单颜色'
  Pf settings set theme dark | Out-Null
  $ok = PfSettings
  Check 'settings page open' $ok ''
  $there = Js "!!document.querySelector('[data-bind=menuColor]')"
  Check 'Settings › 常规 › 外观 has 菜单颜色' ([bool]$there) ''
  $r = PageRect "[data-bind=theme]"
  if ($r) { Crop 'page-appearance' ($r.x - 560) ($r.y - 40) 760 220 | Out-Null }
  $opts = Js "[...document.querySelector('[data-bind=menuColor]').options].map(o => o.value + '=' + o.textContent).join(' | ')"
  Check 'its choices' ("$opts" -match '^followMode=跟随颜色模式 \| light=浅色 \| dark=深色$') "$opts"
  Js "(() => { const s = document.querySelector('[data-bind=menuColor]'); s.value = 'light'; s.dispatchEvent(new Event('change')); return true; })()" | Out-Null
  Start-Sleep -Milliseconds 800
  Check 'choosing 浅色 on the page saves it' ((Setting menuColor) -eq 'light') (Setting menuColor)
  $r = PageRect "[data-bind=menuColor]"
  if ($r) { Crop 'page-appearance-after' ($r.x - 560) ($r.y - 110) 760 220 | Out-Null }
  [SW]::PostMessageW([G]::FindWindowW('PecoFence.Settings', [NullString]::Value), 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
  Start-Sleep -Seconds 1
  $m = ItemMenu
  Check 'after the page: dark mode, light item menu' ($m -eq 'light') $m
  Remove-Item -LiteralPath $p -ErrorAction SilentlyContinue
}

if (-not $Part -or $Part -match 'B') {
  Case '应用到所有栅栏 on the 栅栏 page'
  PfStart -Lang zh-CN
  # Icons in every fence, so the pictures show what changed: files and folders the first-run
  # rules sort into 文件与文档 and 文件夹.
  $made = @()
  foreach ($n in @('季度报告.txt', '会议记录 2026.txt', 'a.txt', '项目计划书最终版.txt', '清单.txt')) { $made += DeskFile $n }
  foreach ($n in @('工作', '旧文件备份', '下载的安装包')) { $d = Join-Path $PfDesk $n; New-Item -ItemType Directory -Force $d | Out-Null; $made += $d }
  Start-Sleep -Seconds 4
  $all = @(TJson fence list) | Where-Object { $_ -and $_.id }
  $regular = @($all | Where-Object { $_.kind -ne 'inbox' -and -not $_.tool })
  $inbox = @($all | Where-Object { $_.kind -eq 'inbox' }) | Select-Object -First 1
  Check 'three regular fences and 「桌面」 arranged like the desktop' ($regular.Count -ge 3 -and $inbox -and $inbox.desktopLayout) "regular=$($regular.Count) inboxDesktopLayout=$($inbox.desktopLayout)"
  $src = $regular[0]; $others = @($regular | Select-Object -Skip 1)
  Pf fence set "$($src.id)" iconAlign center | Out-Null
  Pf fence set "$($src.id)" iconSize 64 | Out-Null
  Pf fence set "$($src.id)" tint '#E74856' | Out-Null
  Pf fence set "$($src.id)" opacity clear | Out-Null
  # The source's new height glides for 250 ms; its height is read once it is there.
  Start-Sleep -Seconds 1
  $before = @{}
  foreach ($o in $others) { $before["$($o.id)"] = FenceGet "$($o.id)" }
  $inboxBefore = FenceGet "$($inbox.id)"
  $srcH = (FenceGet "$($src.id)").windowRect.h
  ParkMouse
  Shot 'desktop-before'
  $ok = PfSettings
  Check 'settings page open' $ok ''
  Js "(() => { document.querySelector('[data-page=fences]').click(); const s = document.getElementById('fenceSel'); s.value = '$($src.id)'; s.dispatchEvent(new Event('change')); return true; })()" | Out-Null
  Start-Sleep -Milliseconds 900
  $card = PageRect '#fenceLayoutAll'
  Check 'the 布局 group ends with 应用到所有栅栏' ([bool]$card -and (Js "!document.getElementById('fenceLayoutAll').hidden")) ''
  $anchor = PageRect '[data-fence=iconAlign]'
  if ($anchor) { Crop 'page-layout-end' ($anchor.x - 640) ($anchor.y - 30) 860 200 | Out-Null }
  if ($card) {
    PageClick '#fenceLayoutAllBtn'
    Start-Sleep -Milliseconds 1200
    $t = PageToast
    Check 'the notice says how many fences took it and offers 撤销' ($t.on -and "$($t.text)" -match "$($others.Count)" -and (Js "!document.getElementById('toastUndo').hidden")) "$($t.text)"
    Crop 'page-layout-applied' ($anchor.x - 640) ($anchor.y - 30) 860 200 | Out-Null
    $sw = Rect ([G]::FindWindowW('PecoFence.Settings', [NullString]::Value))
    Crop 'page-window-applied' $sw.L $sw.T ($sw.R - $sw.L) ($sw.B - $sw.T) | Out-Null
    $bad = @()
    foreach ($o in $others) {
      $g = FenceGet "$($o.id)"
      if ($g.iconAlign -ne 'center' -or $g.iconSize -ne 64) { $bad += "$($o.title): $($g.iconAlign) $($g.iconSize)" }
      if ((NoHash $g.tint) -ne (NoHash $before["$($o.id)"].tint) -or $g.opacity -ne $before["$($o.id)"].opacity) { $bad += "$($o.title) looks changed: $($g.tint) $($g.opacity)" }
    }
    Check 'every other fence took the layout and kept its looks' ($bad.Count -eq 0) ($bad -join '; ')
    $ib = FenceGet "$($inbox.id)"
    Check '「桌面」 arranged like the desktop kept its own' ($ib.iconSize -eq $inboxBefore.iconSize -and $ib.iconAlign -eq $inboxBefore.iconAlign) "$($ib.iconSize) $($ib.iconAlign)"
    $h = (FenceGet "$($src.id)").windowRect.h
    Check 'the fence the options came from keeps its height (the last one in the column is kept short, as one change at a time does)' ($h -eq $srcH) "before $srcH, after $h"
    [SW]::ShowWindow([G]::FindWindowW('PecoFence.Settings', [NullString]::Value), 6) | Out-Null
    Start-Sleep -Milliseconds 900
    Shot 'desktop-layout-applied'
    [SW]::ShowWindow([G]::FindWindowW('PecoFence.Settings', [NullString]::Value), 9) | Out-Null
    Start-Sleep -Milliseconds 900
    Js "(() => { document.getElementById('toastUndo').click(); return true; })()" | Out-Null
    Start-Sleep -Milliseconds 1200
    $bad = @()
    foreach ($o in $others) {
      $g = FenceGet "$($o.id)"; $b = $before["$($o.id)"]
      if ($g.iconAlign -ne $b.iconAlign -or $g.iconSize -ne $b.iconSize) { $bad += "$($o.title): $($g.iconAlign) $($g.iconSize) (was $($b.iconAlign) $($b.iconSize))" }
    }
    Check '撤销 sets every fence back to its own layout' ($bad.Count -eq 0) ($bad -join '; ')
  }
  $card = PageRect '#fenceLooksAll'
  Check 'the 外观 group ends with 应用到所有栅栏' ([bool]$card -and (Js "!document.getElementById('fenceLooksAll').hidden")) ''
  if ($card) {
    PageClick '#fenceLooksAllBtn'
    Start-Sleep -Milliseconds 1200
    $bad = @()
    foreach ($o in $others) {
      $g = FenceGet "$($o.id)"
      if ((NoHash $g.tint) -ne 'E74856' -or $g.opacity -ne 'clear') { $bad += "$($o.title): $($g.tint) $($g.opacity)" }
      if ($g.iconSize -ne $before["$($o.id)"].iconSize) { $bad += "$($o.title) layout changed: $($g.iconSize)" }
    }
    Check 'every other fence took the looks and kept its layout' ($bad.Count -eq 0) ($bad -join '; ')
    $ib = FenceGet "$($inbox.id)"
    Check '「桌面」 arranged like the desktop kept its looks' ((NoHash $ib.tint) -eq (NoHash $inboxBefore.tint) -and $ib.opacity -eq $inboxBefore.opacity) "$($ib.tint) $($ib.opacity)"
    PageClick '#fenceLooksAllBtn'
    Start-Sleep -Milliseconds 900
    $t = PageToast
    Check 'a second click says the others are already so' ($t.on -and (Js "document.getElementById('toastUndo').hidden")) "$($t.text)"
    [SW]::ShowWindow([G]::FindWindowW('PecoFence.Settings', [NullString]::Value), 6) | Out-Null
    Start-Sleep -Milliseconds 900
    Shot 'desktop-looks-applied'
  }
  $line = @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match 'options set on several fences' }) | Select-Object -Last 1
  Check 'the log says what was set' ([bool]$line) "$line"
  foreach ($m in $made) { Remove-Item -LiteralPath $m -Recurse -Force -ErrorAction SilentlyContinue }
}

# C: what a review of the merged round found. 撤销 gives every fence back its place and height
# (spacing and name lines fit a fence's height to its icons, a cell snap can move it left);
# with 「桌面」 arranged like the desktop picked, the two rows are not offered; a tab whose title
# follows the tint keeps following it when its window takes the looks.
function RectText($f) { $r = $f.rect; return "$($r.x),$($r.y),$($r.w),$($r.h)" }
if (-not $Part -or $Part -match 'C') {
  Case '撤销 puts every fence back where it was and as high as it was'
  PfStart -Lang zh-CN
  $made = @()
  foreach ($n in @('季度报告.txt', '会议记录 2026.txt', 'a.txt', '项目计划书最终版.txt', '清单.txt')) { $made += DeskFile $n }
  foreach ($n in @('工作', '旧文件备份', '下载的安装包')) { $d = Join-Path $PfDesk $n; New-Item -ItemType Directory -Force $d | Out-Null; $made += $d }
  Start-Sleep -Seconds 4
  $all = @(TJson fence list) | Where-Object { $_ -and $_.id }
  $regular = @($all | Where-Object { $_.kind -ne 'inbox' -and -not $_.tool })
  $inbox = @($all | Where-Object { $_.kind -eq 'inbox' }) | Select-Object -First 1
  $a = $regular[0]; $b = $regular[1]; $c = $regular[2]
  Pf settings set snapping.sizeToCells true | Out-Null
  # B sized by hand: taller than its icons need, a width off the cells, at the right edge.
  $gb = FenceGet "$($b.id)"
  Pf fence move "$($b.id)" --rect "1330,$($gb.rect.y),262,$($gb.rect.h + 150)" | Out-Null
  Start-Sleep -Seconds 1
  Pf fence set "$($a.id)" spacingPx -12 | Out-Null
  Pf fence set "$($a.id)" iconSize 64 | Out-Null
  Pf fence set "$($a.id)" labelLines 1 | Out-Null
  Start-Sleep -Seconds 1
  $before = @{}
  foreach ($o in @($b, $c)) { $before["$($o.id)"] = RectText (FenceGet "$($o.id)") }
  Log "before: B $($before["$($b.id)"]) C $($before["$($c.id)"])"
  $ok = PfSettings
  Check 'settings page open' $ok ''
  Js "(() => { document.querySelector('[data-page=fences]').click(); const s = document.getElementById('fenceSel'); s.value = '$($a.id)'; s.dispatchEvent(new Event('change')); return true; })()" | Out-Null
  Start-Sleep -Milliseconds 900
  PageClick '#fenceLayoutAllBtn'
  Start-Sleep -Milliseconds 1500
  $mid = @{}
  foreach ($o in @($b, $c)) { $mid["$($o.id)"] = RectText (FenceGet "$($o.id)") }
  Log "after apply: B $($mid["$($b.id)"]) C $($mid["$($c.id)"])"
  Check 'B took the layout' ((FenceGet "$($b.id)").spacingPx -eq -12) ''
  Js "(() => { document.getElementById('toastUndo').click(); return true; })()" | Out-Null
  Start-Sleep -Milliseconds 1500
  $bad = @()
  foreach ($o in @($b, $c)) { $now = RectText (FenceGet "$($o.id)"); if ($now -ne $before["$($o.id)"]) { $bad += "$($o.title): $now, was $($before["$($o.id)"]) (after apply $($mid["$($o.id)"]))" } }
  Check '撤销 puts every fence back at its place and height (B sized by hand too)' ($bad.Count -eq 0) ($bad -join '; ')

  Case 'with 「桌面」 arranged like the desktop picked, 应用到所有栅栏 is not offered'
  Js "(() => { const s = document.getElementById('fenceSel'); s.value = '$($inbox.id)'; s.dispatchEvent(new Event('change')); return true; })()" | Out-Null
  Start-Sleep -Milliseconds 900
  $hidden = Js "document.getElementById('fenceLooksAll').hidden && document.getElementById('fenceLayoutAll').hidden"
  Check 'both rows hidden for 「桌面」 arranged like the desktop' ([bool]$hidden) ''
  [SW]::PostMessageW([G]::FindWindowW('PecoFence.Settings', [NullString]::Value), 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
  Start-Sleep -Seconds 1

  Case 'a tab whose title follows the tint keeps following it'
  Pf settings set snapping.sizeToCells false | Out-Null
  Pf fence merge "$($c.id)" --into "$($b.id)" | Out-Null
  Start-Sleep -Seconds 1
  Pf fence set "$($b.id)" tint '#0078D4' | Out-Null
  Pf fence set "$($c.id)" titleColor tint | Out-Null
  Pf fence set "$($a.id)" tint '#E74856' | Out-Null
  Pf fence set "$($a.id)" titleColor tint | Out-Null
  Start-Sleep -Milliseconds 600
  Check 'the tab follows the tint before' ((FenceGet "$($c.id)").titleColor -eq 'tint') "$((FenceGet "$($c.id)").titleColor)"
  $ok = PfSettings
  Js "(() => { document.querySelector('[data-page=fences]').click(); const s = document.getElementById('fenceSel'); s.value = '$($a.id)'; s.dispatchEvent(new Event('change')); return true; })()" | Out-Null
  Start-Sleep -Milliseconds 900
  PageClick '#fenceLooksAllBtn'
  Start-Sleep -Milliseconds 1500
  $gc = FenceGet "$($c.id)"
  Check 'the window took the new tint' ((NoHash $gc.tint) -eq 'E74856') "$($gc.tint)"
  Check 'the tab title still follows the tint (not the old blue)' ($gc.titleColor -eq 'tint') "$($gc.titleColor)"
  foreach ($m in $made) { Remove-Item -LiteralPath $m -Recurse -Force -ErrorAction SilentlyContinue }
}
