# A light plugin's pf.show caption (feedback #175 for the store's 时钟): rolled up, the fence's
# title row shows it in place of the name; 收起时缩到标题的宽度 takes its width, also when the
# caption changes while rolled; opened, the title row shows the name again.
# English UI. Debug build (CDP on the settings page, as plugins.ps1).
$Feed = 'C:\pf\run\pfeed'
$script:Serial = 1
# The plugin: one row; its caption alternates between a short and a long text every 4 s.
$Js = @'
let long = false;
function draw() { pf.show({ rows: [{ text: 'caption test ' + (long ? 'long' : 'short') }], caption: long ? 'Twenty two chars long!' : '9:41' }); }
pf.on('start', draw);
pf.every(4, () => { long = !long; draw(); });
'@
function Publish() {
  $commit = ('{0:x}' -f 7).PadLeft(40, 'a')
  $dir = "$Feed\$commit\plugins\pf-cap"; New-Item -ItemType Directory -Force $dir | Out-Null
  $m = [ordered]@{ id = 'pf-cap'; version = '1.0.0'; api = 1; layer = 'light'; name = @{ en = 'PF Cap' }; description = @{ en = 'A test plugin' }
    author = 'tester'; minPecoFence = '0.4.0'; permissions = @{}; settings = @() }
  [IO.File]::WriteAllText("$dir\plugin.json", ($m | ConvertTo-Json -Depth 6))
  [IO.File]::WriteAllText("$dir\main.js", $Js)
  [IO.File]::WriteAllText("$dir\README.md", "# pf-cap`r`n")
  $m.files = @(foreach ($f in Get-ChildItem $dir -File) { [ordered]@{ name = $f.Name; size = $f.Length; sha256 = (Get-FileHash $f.FullName -Algorithm SHA256).Hash.ToLower() } })
  [IO.File]::WriteAllText("$Feed\index.json", ([ordered]@{ format = 1; serial = $script:Serial; commit = $commit; plugins = @($m) } | ConvertTo-Json -Depth 8))
}
function RowButton($name) { return (Js "[...document.querySelectorAll('#pluginList .pl-row')].findIndex(r => r.textContent.startsWith('$name'))") }
# The text OCR reads at the top of fence window `w` (the title row and, when open, the first row; the chevron left out).
function TitleText($w) { return (OcrText ($w.x + 4) ($w.y + 2) ([Math]::Max(60, $w.w - 40)) 56 'title') }

Case 'setup: the feed, the plugin installed, its fence'
CloseWindows
New-Item -ItemType Directory -Force $Feed | Out-Null
$feedProc = PfFeed 8732 -log 'C:\pf\run\pfeed.log'
Publish
$env:PECOFENCE_PLUGINS_URL = 'http://127.0.0.1:8732/index.json'
if ($PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
PfStart -Lang en
$ok = PfSettings
PageClick 'nav a[data-page=tools]'; Start-Sleep -Seconds 3
$i = [int](RowButton 'PF Cap')
PageClick '#pluginList .pl-row button' $i; Start-Sleep -Milliseconds 600
PageClick '#plOk'; Start-Sleep -Seconds 4
$line = PfWaitLog 'plugins: fence added' 15
$f = @(PluginFences pf-cap)
Check 'the plugin installed and its fence runs' ($ok -and $f.Count -eq 1 -and $line) "fences=$($f.Count) $line"
CloseWindows
$id = $f[0].id
Pf fence move $id --rect '16,120,300,200' | Out-Null
Start-Sleep -Seconds 2

Case 'rolled up, the title row shows the caption; opened, the name'
$open = (FenceGet $id).windowRect
$nameOpen = TitleText $open
Pf fence roll $id | Out-Null
$swap = PfWaitLog 'rolled caption: the title row shows the caption' 5
Start-Sleep -Seconds 1
$w = (FenceGet $id).windowRect
$t1 = TitleText $w
Shot 'rolled-caption'
Check 'open: the title row shows the name' ($nameOpen -match 'PF Cap') "ocr='$nameOpen'"
Check 'rolled: the title row shows the caption, not the name' (($t1 -match '9:41|Twenty') -and $t1 -notmatch 'PF Cap' -and $swap) "ocr='$t1' log=$([bool]$swap)"
Pf fence unroll $id | Out-Null
Start-Sleep -Seconds 1
$t2 = TitleText (FenceGet $id).windowRect
Check 'opened again: the name is back' ($t2 -match 'PF Cap') "ocr='$t2'"
Shot 'opened-name'

Case 'compact rolled rows take the caption''s width, also when it changes'
Pf settings set rollUp.compactRolled true | Out-Null
Start-Sleep -Milliseconds 800
Pf fence roll $id | Out-Null
Start-Sleep -Seconds 1
# The caption alternates every 4 s: the rolled row's width follows it (read for 14 s).
$widths = @{}
$deadline = (Get-Date).AddSeconds(14)
while ((Get-Date) -lt $deadline) {
  $w = (FenceGet $id).windowRect
  if (-not $widths.ContainsKey([int]$w.w)) { $widths[[int]$w.w] = $true; Shot "compact-$($w.w)" }
  Start-Sleep -Milliseconds 500
}
$ws = @($widths.Keys | Sort-Object)
$stale = PfWaitLog 'plugin caption: the rolled row takes its width' 2
Check 'the compact row is narrow for 9:41 and wider for the long caption' ($ws.Count -ge 2 -and $ws[-1] - $ws[0] -gt 40 -and $ws[-1] -lt 300 -and $stale) "widths=$($ws -join ' ') px; log=$([bool]$stale)"
Pf fence unroll $id | Out-Null
Start-Sleep -Seconds 1
$w = (FenceGet $id).windowRect
Check 'opened, the fence has its own width again' ([Math]::Abs($w.w - $open.w) -le 2) "width=$($w.w) was $($open.w)"
Pf settings set rollUp.compactRolled false | Out-Null

# The store's 时钟 plugin itself (run.sh --extra <dir with a clock\ folder: plugin.json, main.js,
# README.md>): rolled up, its title row shows the time from 1.0.1 on, its name before.
if (Test-Path "$PfExtra\clock\plugin.json") {
  Case 'the store clock plugin rolled up'
  $cm = Get-Content "$PfExtra\clock\plugin.json" -Raw -Encoding UTF8 | ConvertFrom-Json
  $commit = ('{0:x}' -f 8).PadLeft(40, 'a')
  $dir = "$Feed\$commit\plugins\clock"; New-Item -ItemType Directory -Force $dir | Out-Null
  Copy-Item "$PfExtra\clock\plugin.json", "$PfExtra\clock\main.js", "$PfExtra\clock\README.md" $dir
  $m = Get-Content "$dir\plugin.json" -Raw -Encoding UTF8 | ConvertFrom-Json
  $m | Add-Member -NotePropertyName files -NotePropertyValue @(foreach ($f in Get-ChildItem $dir -File) { [ordered]@{ name = $f.Name; size = $f.Length; sha256 = (Get-FileHash $f.FullName -Algorithm SHA256).Hash.ToLower() } })
  $script:Serial = 2
  [IO.File]::WriteAllText("$Feed\index.json", ([ordered]@{ format = 1; serial = $script:Serial; commit = $commit; plugins = @($m) } | ConvertTo-Json -Depth 10))
  $ok = PfSettings
  PageClick 'nav a[data-page=tools]'; Start-Sleep -Seconds 1
  Js "window.chrome.webview.postMessage({ type: 'action', name: 'pluginsRefresh' })" | Out-Null; Start-Sleep -Seconds 3
  $i = [int](RowButton 'Clock')
  PageClick '#pluginList .pl-row button' $i; Start-Sleep -Milliseconds 600
  PageClick '#plOk'; Start-Sleep -Seconds 4
  CloseWindows
  $c = @(PluginFences clock)
  Check "the clock plugin $($cm.version) installed, its fence runs" ($c.Count -eq 1) "fences=$($c.Count)"
  $cid = $c[0].id
  Pf fence move $cid --rect '16,560,300,200' | Out-Null
  Start-Sleep -Seconds 2
  Pf fence roll $cid | Out-Null
  Start-Sleep -Seconds 2
  $w = (FenceGet $cid).windowRect
  $t = TitleText $w
  Shot "clock-$($cm.version)-rolled"
  $time = $t -match '\d{1,2}[:.]\d{2}'
  if ($cm.version -eq '1.0.0') {
    Check 'clock 1.0.0 rolled up: its title row shows its name' (-not $time) "ocr='$t'"
  } else {
    Check "clock $($cm.version) rolled up: its title row shows the time" ($time -and $t -notmatch 'Clock') "ocr='$t'"
  }
  Pf fence unroll $cid | Out-Null
}

Case 'end'
if ($feedProc) { Stop-Process -Id $feedProc.Id -Force -ErrorAction SilentlyContinue }
$bad = @(PfLogAll | Where-Object { $_ -match ' ERROR |panicked' })
Check 'app log has no ERROR or panic line' ($bad.Count -eq 0) ($bad | Select-Object -First 2)
