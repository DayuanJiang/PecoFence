# 插件的轻量层 (sample): the clock plugin from a local test index (PECOFENCE_PLUGINS_URL) to a
# running plugin fence beside the native clock: the install into plugins\clock\<version>, the
# time it shows, its seconds on the second, its 显示秒 setting, no ticks while it is hidden, the
# short form in a narrow fence, and quit ending the plugin thread. Debug build (CDP on the page);
# --extra a folder with clock\plugin.json, main.js, README.md. `--var Part=Ma|Mb|Mc` runs
# measure.ps1 of that folder instead (memory and CPU of one native clock, one plugin clock or six
# plugin clocks, release build).
if ($Part -match '^M([abc])$') {
  Case "memory and CPU, scenario $($Matches[1])"
  & powershell -NoProfile -ExecutionPolicy Bypass -File "$PfExtra\measure.ps1" -Scenario $Matches[1] -Minutes $(if ($Minutes) { $Minutes } else { 5 }) -Root "$PfRun\m" -Out "$PfShots\measure"
  $rows = @(Get-Content "$PfShots\measure\samples.csv" -ErrorAction SilentlyContinue)
  Check 'a sample every 30 s' ($rows.Count -ge 3) ((Get-Content "$PfShots\measure\log.txt") -join ' / ')
  return
}
$Feed = 'C:\pf\run\pfeed'
function ClockFence() { return @(@((PfJson fence list)) | Where-Object { $_ -and $_.script -and $_.script.path -eq 'plugin:clock' })[0] }
function Shows() { return (ClockFence).script.output }
function Rows() { return (Js "[...document.querySelectorAll('#pluginList .pl-row')].map(r => r.textContent).join(' / ')") }

Case 'setup: a test index with the clock plugin'
CloseWindows
$commit = 'c10c'.PadLeft(40, 'a')
$dir = "$Feed\$commit\plugins\clock"; New-Item -ItemType Directory -Force $dir | Out-Null
Copy-Item "$PfExtra\clock\*" $dir
$m = Get-Content "$dir\plugin.json" -Raw -Encoding UTF8 | ConvertFrom-Json
$m | Add-Member files @(foreach ($f in Get-ChildItem $dir -File) { [ordered]@{ name = $f.Name; size = $f.Length; sha256 = (Get-FileHash $f.FullName -Algorithm SHA256).Hash.ToLower() } })
[IO.File]::WriteAllText("$Feed\index.json", ([ordered]@{ format = 1; commit = $commit; plugins = @($m) } | ConvertTo-Json -Depth 8))
$server = @'
$l = New-Object System.Net.Sockets.TcpListener([System.Net.IPAddress]::Parse('127.0.0.1'), 8731); $l.Start()
while ($true) { $c = $l.AcceptTcpClient(); try { $s = $c.GetStream(); $r = New-Object IO.StreamReader($s); $line = $r.ReadLine(); while ($r.ReadLine()) {}
  $path = 'C:\pf\run\pfeed' + (($line -split ' ')[1] -replace '/', '\')
  if (Test-Path -LiteralPath $path -PathType Leaf) { $b = [IO.File]::ReadAllBytes($path); $h = "HTTP/1.1 200 OK`r`nContent-Length: $($b.Length)`r`nConnection: close`r`n`r`n" } else { $b = @(); $h = "HTTP/1.1 404 Not Found`r`nContent-Length: 0`r`nConnection: close`r`n`r`n" }
  $hb = [Text.Encoding]::ASCII.GetBytes($h); $s.Write($hb, 0, $hb.Length); if ($b.Length) { $s.Write($b, 0, $b.Length) }; $s.Flush() } catch {} finally { $c.Close() } }
'@
Set-Content 'C:\pf\run\pfeed-serve.ps1' $server
$feedProc = Start-Process powershell.exe -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', 'C:\pf\run\pfeed-serve.ps1' -WindowStyle Hidden -PassThru
$env:PECOFENCE_PLUGINS_URL = 'http://127.0.0.1:8731/index.json'
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
PfStart -Lang en
$script:Config = Split-Path (PfJson status).configPath
$ok = PfSettings
PageClick 'nav a[data-page=tools]'; Start-Sleep -Seconds 3
Check 'the tools page offers Clock' ($ok -and (Rows) -match 'Clock.*Install') (Rows)

Case 'install: the plugin fence runs main.js'
PageClick '#pluginList .pl-row button' ([int](Js "[...document.querySelectorAll('#pluginList .pl-row')].findIndex(r => r.textContent.startsWith('Clock'))"))
Start-Sleep -Milliseconds 600; PageClick '#plOk'
$line = PfWaitLog 'plugin fence starts its main.js' 20
Start-Sleep -Seconds 2
Check 'installed in plugins\clock\1.0.0, current.txt names it' ((Test-Path "$script:Config\plugins\clock\1.0.0\main.js") -and (Get-Content "$script:Config\plugins\clock\current.txt") -eq '1.0.0') $line
Check 'the fence shows the time without seconds and the date' ((Shows) -match '^\d{1,2}:\d{2}( [AP]M)?\n.*\d') (Shows)
[L]::PostMessageW((WaitWin 'PecoFence.Settings' 2), 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
Pf settings set tools.clockSeconds true | Out-Null
Pf fence create --tool clock --title 'Native' --rect '300,150,260,150' | Out-Null
Pf fence move (ClockFence).id --rect '600,150,260,150' | Out-Null
Start-Sleep -Seconds 2

Case 'the 显示秒 setting in the fence menu'
$r = (ClockFence).windowRect
[G]::RClick($r.x + 120, $r.y + 14); $menu = MenuWin 5
$texts = MenuTexts $menu
MenuPick '^Show seconds$' $menu | Out-Null; Start-Sleep -Seconds 2
$saved = (Get-Content "$script:Config\config.json" -Raw -Encoding UTF8 | ConvertFrom-Json).scripts.((ClockFence).id).settings.seconds
Check 'the menu offers Show seconds; on, the time has seconds and the config keeps it' ($texts -match 'Show seconds' -and (Shows) -match '^\d{1,2}:\d{2}:\d{2}' -and $saved -eq $true) "menu=$texts shows=$(Shows) saved=$saved"
ParkMouse
Shot 'side-by-side-native-left-plugin-right'

Case 'seconds change on the second, as the native clock''s do'
Add-Type -ReferencedAssemblies System.Drawing @'
using System; using System.Drawing; using System.Drawing.Imaging; using System.Collections.Generic; using System.Runtime.InteropServices;
public static class Ticks {
  [DllImport("kernel32.dll")] static extern void GetSystemTimePreciseAsFileTime(out long t);
  static long Ms() { long t; GetSystemTimePreciseAsFileTime(out t); return t / 10000 - 11644473600000L; }
  static long Hash(Bitmap b) {
    var d = b.LockBits(new Rectangle(0, 0, b.Width, b.Height), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
    var px = new int[b.Width * b.Height]; Marshal.Copy(d.Scan0, px, 0, px.Length); b.UnlockBits(d);
    long h = 1469598103934665603L; foreach (var p in px) h = (h ^ p) * 1099511628211L; return h;
  }
  // When each of the boxes at (x0, y) and (x1, y) changed, as unix ms, for `ms`: "box,ms" lines.
  public static string Watch(int x0, int x1, int y, int w, int h, int ms) {
    var log = new List<string>(); var last = new long[2]; long end = Ms() + ms;
    using (var b = new Bitmap(w, h)) using (var g = Graphics.FromImage(b))
      while (Ms() < end) for (int k = 0; k < 2; k++) {
        g.CopyFromScreen(k == 0 ? x0 : x1, y, 0, 0, new Size(w, h)); long v = Hash(b), at = Ms();
        if (last[k] != 0 && v != last[k]) log.Add(k + "," + at); last[k] = v;
      }
    return string.Join(";", log);
  }
}
'@
$nat = (PfFence 'Native').windowRect; $plg = (ClockFence).windowRect
$seen = [Ticks]::Watch($nat.x + 8, $plg.x + 8, $nat.y + 40, 240, 60, 15000) -split ';' | Where-Object { $_ } | ForEach-Object { $a = $_ -split ','; [pscustomobject]@{ box = [int]$a[0]; ms = [long]$a[1] } }
$n = @($seen | Where-Object box -eq 0); $p = @($seen | Where-Object box -eq 1)
$gaps = @(foreach ($t in $n) { $q = $p | Sort-Object { [math]::Abs($_.ms - $t.ms) } | Select-Object -First 1; if ($q) { $q.ms - $t.ms } })
$late = @($p | ForEach-Object { $_.ms % 1000 })
Log "native at ms $(($n | ForEach-Object { $_.ms % 1000 }) -join ','); plugin at ms $($late -join ','); plugin minus native $($gaps -join ',')"
Check 'the plugin clock changed about once a second' ($p.Count -ge 13 -and $p.Count -le 16) "plugin=$($p.Count) native=$($n.Count)"
Check 'each change within 50 ms of the native clock''s' ($gaps.Count -ge 13 -and @($gaps | Where-Object { [math]::Abs($_) -gt 50 }).Count -eq 0) "gaps=$($gaps -join ',')"
. C:\pf\kit\rec.ps1
RecStart 'tick-both' -X $nat.x -Y $nat.y -W ($plg.x + $plg.w - $nat.x) -H $nat.h -Fps 20
RecMark '看秒数跳动'; Start-Sleep -Seconds 6; RecStop

Case 'hidden: no ticks while rolled up, while all fences are hidden, on another virtual desktop'
$id = (ClockFence).id
function Paused($hide, $show) {
  & $hide; Start-Sleep -Seconds 1; $a = Shows; Start-Sleep -Seconds 3; $b = Shows
  & $show; Start-Sleep -Milliseconds 1500; $c = Shows
  return @(($a -eq $b -and $c -ne $a), "hidden=$a later=$b back=$c")
}
$r = Paused { Pf fence roll $id | Out-Null } { Pf fence unroll $id | Out-Null }
Check 'rolled up: the same second for 3 s, a new one when it opens' $r[0] $r[1]
$r = Paused { Pf fence hide-all | Out-Null } { Pf fence show-all | Out-Null }
Check 'all fences hidden: the same' $r[0] $r[1]
. C:\pf\kit\vdesk.ps1
VdOnlyOne; VdNew; VdLeft
Pf fence set $id virtualDesktops 1 | Out-Null
$r = Paused { VdRight } { VdLeft }
Check 'another virtual desktop: the same' $r[0] $r[1]
Check 'the log says the plugin was told' ([bool](PfWaitLog 'plugin shown or hidden, its timers follow.*shown=false' 2)) ''
VdOnlyOne

Case 'a fence too narrow for the seconds shows the short form'
Log "resize: $(Pf fence resize (ClockFence).id --w 70) $(Pf fence resize Native --w 70)"
Start-Sleep -Seconds 2
$nat = (PfFence 'Native').windowRect; $plg = (ClockFence).windowRect
$tn = OcrText $nat.x ($nat.y + 30) $nat.w 70 'narrow-native'; $tp = OcrText $plg.x ($plg.y + 30) $plg.w 70 'narrow-plugin'
Shot 'narrow-native-left-plugin-right'
Check 'both show the time without seconds, the same text' ($tp -eq $tn -and $tp -match '\d{1,2}.?\d{2}' -and $tp -notmatch '\d:\d{2}:\d{2}' -and $tn -notmatch '\d:\d{2}:\d{2}') "plugin=$tp native=$tn view=$(Shows)"

Case 'quit stops the plugins and the plugin thread'
PfQuit
Check 'the log says every plugin stopped and the thread ended' ([bool](PfWaitLog 'plugin thread: every plugin stops' 5) -and [bool](PfWaitLog 'plugin thread ended' 5)) ''
if ($feedProc) { Stop-Process -Id $feedProc.Id -Force -ErrorAction SilentlyContinue }
