# Owner round, third batch: texts of notices, of one settings card and of an empty fence.
# --var Part=N|W|J (default all).
#   N: after a crash inside ExplorerPatcher while a menu was built, the notice names the Windows
#      menu entries that are off now and the tray menu that brings them back (any build).
#   W: Settings › 布局与备份 in Simplified Chinese at the window's default size: the description of
#      the 导出 / 导入配置文件 card is one line (it broke inside the word 电脑). Debug build: the
#      page is read over CDP.
#   J: the text of an empty fence 150 px wide in Japanese is one line (the last character stood
#      alone on a second line).
trap { PfTrap $_; continue }
if (-not $Part) { $Part = 'NWJ' }

if ($Part -match 'N') {
  Case 'N a crash inside ExplorerPatcher while a menu was built'
  PfStart
  PfQuit
  # A crashed run's shareable log as crashlog leaves it: the CRASH line names ExplorerPatcher's
  # DLL as the faulting module and the menu that was being built (feedback.rs crashed_building_menu).
  $t = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ss.ffffffZ')
  Add-Content (ShareLog) "$t ERROR pecofence_platform::crashlog: CRASH (unhandled exception) code=0xc0000005 at=ExplorerPatcher.amd64.dll+0xc0c1 module_owner=other module_version=26100.8457.70.3 command=ItemMenu access=write stack=  ExplorerPatcher.amd64.dll+0xc0c1" -Encoding UTF8
  PfStart
  $toast = ToastOcr 'ExplorerPatcher' 8
  Crop 'n-notice' 1180 600 420 260 | Out-Null
  $ep = PfWaitLog 'the crash was inside ExplorerPatcher' 5
  $off = PfWaitLog 'item menus are built without the Windows part' 2
  Check 'N the start sees the crash inside ExplorerPatcher and builds menus without the Windows part' ([bool]$ep -and [bool]$off) "ep: $ep | off: $off"
  Check 'N the notice names the setting and the tray menu' ($toast -match 'Register as shell extension' -and $toast -match 'tray menu') "toast: '$toast'"
  PfQuit
}

if ($Part -match 'W') {
  Case 'W the description of the import card is one line'
  if (-not (IsDebugBuild)) {
    Check 'W needs a debug build' $false 'release build'
  } else {
    CloseWindows
    $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
    PfStart -Lang zh-CN
    Pf settings set theme light | Out-Null
    Check 'W settings open over CDP' (PfSettings)
    Js "(document.querySelector('nav a[data-page=layout]').click(), true)" | Out-Null
    Start-Sleep -Seconds 1
    Js "(document.getElementById('cfgExport').closest('.card').scrollIntoView({ block: 'center', behavior: 'instant' }), true)" | Out-Null
    Start-Sleep -Seconds 1
    # The lines of the description: the distinct tops of its text boxes.
    $lines = Js "(() => { const e = document.getElementById('cfgExport').closest('.card').querySelector('.s'); const r = document.createRange(); r.selectNodeContents(e); const tops = new Set(); for (const b of r.getClientRects()) tops.add(Math.round(b.top / 4)); return tops.size; })()"
    $text = Js "document.getElementById('cfgExport').closest('.card').querySelector('.s').textContent"
    $h = WaitWin 'PecoFence.Settings' 5
    $r = Rect $h
    Crop 'w-layout' $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) | Out-Null
    Check 'W the description is on one line' ([int]$lines -eq 1) "lines=$lines text=$text"
    CloseWindows
    PfQuit
  }
}

if ($Part -match 'J') {
  Case 'J the text of a narrow empty fence in Japanese'
  CloseWindows
  PfStart -Lang ja
  Pf settings set theme light | Out-Null
  Pf fence create --title 'Narrow' --rect '40,250,150,150' | Out-Null
  Start-Sleep -Seconds 3
  ParkMouse
  $b = Crop 'j-narrow' 40 250 150 150
  # The lines of text under the title: runs of pixel rows that hold dark pixels (a gap of fewer
  # than 4 rows is inside one line: the marks over a kana).
  $runs = 0; $blank = 9
  for ($y = 44; $y -lt 130; $y++) {
    $dark = 0
    for ($x = 8; $x -lt 142; $x++) { $c = $b.GetPixel($x, $y); if ($c.R -lt 140 -and $c.G -lt 140 -and $c.B -lt 150) { $dark++ } }
    if ($dark -ge 3) { if ($blank -ge 4) { $runs++ }; $blank = 0 } else { $blank++ }
  }
  Check 'J the text is one line' ($runs -eq 1) "lines=$runs"
  Pf fence delete 'Narrow' | Out-Null
  PfQuit
}
