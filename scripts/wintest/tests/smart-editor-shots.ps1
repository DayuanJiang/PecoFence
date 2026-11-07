# Shots of the three smart fence editor samples (A small window, B in 设置 › 栅栏, C a 智能栅栏
# page) over the real desktop: PecoFence runs in Chinese and light theme with 最近 7 天下载 (30
# files) and 所有 .psd 文件; each sample page (C:\pf\extra\alt-*.html, the settings page with
# the harness's mock host) opens in an Edge app window of the size the real window would have.
# Make the pages first (dev box): PECOFENCE_UI_TEST_PORT=43291 node scripts/test-settings-ui.mjs,
# then for a in A B C, s in new edit: curl "http://127.0.0.1:43291/preview?mode=light&sample=$a&shot=$s"
# -o <dir>/alt-$a-$s.html, copy ui/i18n.js into <dir>, and run.sh --extra <dir> this file.
Case 'setup'
CloseWindows
ClearDownloads
New-Item -ItemType Directory -Force $PfDl | Out-Null
$exts = @('pdf', 'docx', 'xlsx', 'png', 'jpg', 'zip', 'txt', 'mp3', 'msi', 'pptx')
for ($i = 0; $i -lt 30; $i++) { AgedFile $PfDl ('download {0:D2}.{1}' -f $i, $exts[$i % 10]) (0.05 + $i * 0.2) | Out-Null }
AgedFile $PfDocs 'poster design.psd' 30 | Out-Null; AgedFile $PfDocs 'cover.psd' 3 | Out-Null
Check 'Everything runs' (EvStart) ''
PfStart -Lang zh-CN
Pf settings set theme light | Out-Null
foreach ($r in @(@('Programs', '程序'), @('Folders', '文件夹'), @('Files and documents', '文件与文档'))) { Pf fence set $r[0] title $r[1] | Out-Null }
Pf fence create --smart downloads --rect '24,40,420,300' | Out-Null
Pf fence create --smart psd --rect '24,360,420,200' | Out-Null
Start-Sleep -Seconds 4
$edge = @("${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe", "$env:ProgramFiles\Microsoft\Edge\Application\msedge.exe") | Where-Object { Test-Path $_ } | Select-Object -First 1
Check 'Edge is installed' ([bool]$edge) "$edge"

function EdgeShot($page, $w, $h, $x, $y) {
  $dir = "$PfRun\edge-$page"
  $p = Start-Process $edge -ArgumentList "--app=file:///C:/pf/extra/$page.html", "--window-size=$w,$h", "--window-position=$x,$y", "--user-data-dir=$dir", '--no-first-run', '--no-default-browser-check', '--disable-features=msEdgeSidebarV2,EdgeWelcomePage,Translate,msEdgeTranslate', '--lang=zh-CN', '--force-device-scale-factor=1' -PassThru
  Start-Sleep -Seconds 6
  [G]::MoveTo(1250, 780); Start-Sleep -Milliseconds 800
  Shot $page
  $procs = Get-CimInstance Win32_Process -Filter "Name = 'msedge.exe'" | Where-Object { $_.CommandLine -like "*$dir*" }
  Log "edge $page pid=$($p.Id) processes=$(@($procs).Count)"
  $procs | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
  Start-Sleep -Seconds 2
}

Case 'shots'
[G]::MoveTo(1200, 700); Start-Sleep -Seconds 3
foreach ($s in @('new', 'edit')) {
  EdgeShot "alt-A-$s" 680 560 460 130
  EdgeShot "alt-B-$s" 960 700 320 76
  EdgeShot "alt-C-$s" 960 700 320 76
}
Check 'six shots' (@(Get-ChildItem "$PfShots\alt-*.png").Count -eq 6) ((Get-ChildItem "$PfShots\alt-*.png" | ForEach-Object { $_.Name }) -join ', ')

Case 'cleanup'
Remove-Item -LiteralPath (Join-Path $PfDocs 'poster design.psd'), (Join-Path $PfDocs 'cover.psd') -ErrorAction SilentlyContinue
PfQuit
