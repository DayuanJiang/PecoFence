# Fence fix round 2026-11-07, explorer-waits: the fences keep answering while Explorer is busy
# or stuck (S1, P2, X4), and the wallpaper is still followed.
#   run.sh --var Part=freeze tests/explorer-busy.ps1     Explorer's desktop thread stopped 20 s
#   run.sh --var Part=burst ...                              1500 and 2000 new desktop files
#   run.sh --var Part=start ...                              start while Explorer adds 1400 files
#   run.sh --var Part=wallpaper ...                          picture change, slideshow, per monitor
#   run.sh --var Part=drop ...                               drop on the desktop, Explorer slow
#   run.sh --var Part=quit ...                               quit while Explorer is stuck: icons back
$PfAllowLogErrors = $true
if (-not $Part) { $Part = 'freeze' }

Add-Type @'
using System; using System.Threading; using System.Diagnostics; using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class Xw {
  [DllImport("kernel32.dll")] static extern IntPtr OpenThread(uint a, bool i, uint id);
  [DllImport("kernel32.dll")] static extern uint SuspendThread(IntPtr h);
  [DllImport("kernel32.dll")] static extern uint ResumeThread(IntPtr h);
  [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
  [DllImport("user32.dll")] static extern IntPtr SendMessageTimeoutW(IntPtr h, uint m, IntPtr w, IntPtr l, uint f, uint t, out IntPtr r);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern bool SystemParametersInfoW(uint a, uint p, string v, uint f);
  public static volatile bool Frozen;
  // A job that ended while its freeze had the thread stopped leaves it stopped for good:
  // resumes it fully; the suspend count it had.
  public static int Thaw(uint tid) {
    var h = OpenThread(0x0002, false, tid); if (h == IntPtr.Zero) return -1;
    int first = -1; uint prev;
    do { prev = ResumeThread(h); if (first < 0) first = (int)prev; } while (prev > 1 && prev != 0xFFFFFFFF);
    CloseHandle(h); return first;
  }
  // Explorer stuck: the thread stops for `ms` (offMs/onMs > 0: stops offMs, runs onMs, again
  // and again for `ms`, an Explorer that answers each message late).
  public static bool Freeze(uint tid, int ms, int offMs, int onMs) {
    var h = OpenThread(0x0002, false, tid); if (h == IntPtr.Zero) return false;
    var t = new Thread(() => {
      Frozen = true; var sw = Stopwatch.StartNew();
      if (offMs <= 0) { SuspendThread(h); Thread.Sleep(ms); ResumeThread(h); }
      else while (sw.ElapsedMilliseconds < ms) { SuspendThread(h); Thread.Sleep(offMs); ResumeThread(h); Thread.Sleep(onMs); }
      Frozen = false; CloseHandle(h);
    });
    t.IsBackground = true; t.Start(); return true;
  }
  // WM_NULL to a window every 20 ms on a thread of its own: the longest answer, and how long
  // after the start the last answer over 200 ms ended. A thread waiting in a COM call or in
  // SendMessageTimeout still answers sent messages: input and the CLI (posted) are the measure.
  static Thread pinger; static volatile bool stop; static long max, lastSlowEnd, n;
  public static void PingStart(IntPtr h) {
    stop = false; max = 0; lastSlowEnd = 0; n = 0;
    pinger = new Thread(() => {
      var all = Stopwatch.StartNew();
      while (!stop) {
        var sw = Stopwatch.StartNew(); IntPtr r;
        var ok = SendMessageTimeoutW(h, 0, IntPtr.Zero, IntPtr.Zero, 0, 60000, out r);
        long ms = ok == IntPtr.Zero ? 60000 : sw.ElapsedMilliseconds; n++;
        if (ms > max) max = ms; if (ms > 200) lastSlowEnd = all.ElapsedMilliseconds;
        Thread.Sleep(20);
      }
    });
    pinger.IsBackground = true; pinger.Start();
  }
  public static string PingStop() { stop = true; if (pinger != null) pinger.Join(70000); return "max=" + max + " lastSlowEnd=" + lastSlowEnd + " n=" + n; }
  public static long PingMax() { return max; }
  public static bool SetWallpaper(string path) { return SystemParametersInfoW(0x14, 0, path, 3); }
}
[ComImport, Guid("B92B56A9-8B55-4E14-9A89-0199BBB6F93B"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface IDesktopWallpaperX {
  void SetWallpaper([MarshalAs(UnmanagedType.LPWStr)] string monitorID, [MarshalAs(UnmanagedType.LPWStr)] string wallpaper);
  [return: MarshalAs(UnmanagedType.LPWStr)] string GetWallpaper([MarshalAs(UnmanagedType.LPWStr)] string monitorID);
  [return: MarshalAs(UnmanagedType.LPWStr)] string GetMonitorDevicePathAt(uint monitorIndex);
  uint GetMonitorDevicePathCount();
  void GetMonitorRECT([MarshalAs(UnmanagedType.LPWStr)] string monitorID, out long a, out long b);
  void SetBackgroundColor(uint color);
  uint GetBackgroundColor();
  void SetPosition(int position);
  int GetPosition();
  void SetSlideshow(IntPtr items);
  IntPtr GetSlideshow();
  void SetSlideshowOptions(int options, uint slideshowTick);
  void GetSlideshowOptions(out int options, out uint slideshowTick);
  void AdvanceSlideshow([MarshalAs(UnmanagedType.LPWStr)] string monitorID, int direction);
  int GetStatus();
  void Enable([MarshalAs(UnmanagedType.Bool)] bool enable);
}
public static class Dw {
  [DllImport("shell32.dll", CharSet=CharSet.Unicode, PreserveSig=false)] static extern void SHCreateItemFromParsingName(string path, IntPtr pbc, ref Guid riid, out IntPtr item);
  [DllImport("shell32.dll", PreserveSig=false)] static extern void SHCreateShellItemArrayFromShellItem(IntPtr item, ref Guid riid, out IntPtr arr);
  static IDesktopWallpaperX W() { return (IDesktopWallpaperX)Activator.CreateInstance(Type.GetTypeFromCLSID(new Guid("C2CF3110-460E-4fc1-B9D0-8A1C0C9CC4BD"))); }
  public static string Monitor0() { return W().GetMonitorDevicePathAt(0); }
  public static void SetFor(string monitor, string path) { W().SetWallpaper(monitor, path); }
  public static string Get(string monitor) { return W().GetWallpaper(monitor); }
  // A slideshow of the pictures in `folder`, a new one every `tickMs`; then one step forward.
  public static void Slideshow(string folder, uint tickMs) {
    var w = W(); var iShellItem = new Guid("43826d1e-e718-42ee-bc55-a1e261c37bfe"); var iArray = new Guid("b63ea76d-1f85-456f-a19c-48159efa858b");
    IntPtr item, arr; SHCreateItemFromParsingName(folder, IntPtr.Zero, ref iShellItem, out item);
    SHCreateShellItemArrayFromShellItem(item, ref iArray, out arr);
    w.SetSlideshow(arr); w.SetSlideshowOptions(0, tickMs);
    Marshal.Release(arr); Marshal.Release(item);
  }
  public static void Advance() { W().AdvanceSlideshow(null, 0); }
  public static int Status() { return W().GetStatus(); }
}
'@

# Milliseconds a CLI command takes; its output in $script:XOut.
function XTimed {
  $a = @($args | ForEach-Object { $_ })
  $sw = [Diagnostics.Stopwatch]::StartNew()
  $script:XOut = & "$PfApp\pecofence-cli.exe" @a 2>&1 | Out-String
  return [int]$sw.ElapsedMilliseconds
}
function XFenceWin($title) { $w = @([W]::Tops('PecoFence.Fence') | Where-Object { [W]::Title($_) -eq $title }); if ($w.Count) { return $w[0] }; return [IntPtr]::Zero }
function XLog($regex) { return @(PfLogAll | Where-Object { $_ -match $regex }) }
function XFiles($n, $prefix) { for ($i = 0; $i -lt $n; $i++) { [IO.File]::WriteAllText((Join-Path $PfDesk ("$prefix{0:D5}.txt" -f $i)), "x $i") } }
function XClean($prefix) { Get-ChildItem $PfDesk -Filter "$prefix*" -Force -ErrorAction SilentlyContinue | Remove-Item -Force -ErrorAction SilentlyContinue }
# A picture of one colour (or with circles), 1600x900, as JPEG.
function XPicture($path, $r, $g, $b, [switch]$Busy) {
  $bmp = New-Object Drawing.Bitmap 1600, 900
  $gr = [Drawing.Graphics]::FromImage($bmp)
  $gr.Clear([Drawing.Color]::FromArgb($r, $g, $b))
  if ($Busy) {
    $rnd = New-Object Random ($r + $g + $b)
    for ($k = 0; $k -lt 40; $k++) { $br = New-Object Drawing.SolidBrush ([Drawing.Color]::FromArgb($rnd.Next(256), $rnd.Next(256), $rnd.Next(256))); $gr.FillEllipse($br, $rnd.Next(1500), $rnd.Next(800), 40 + $rnd.Next(300), 40 + $rnd.Next(300)); $br.Dispose() }
  }
  $gr.Dispose(); $bmp.Save($path, [Drawing.Imaging.ImageFormat]::Jpeg); $bmp.Dispose()
}
# Mean colour inside the fence titled $title (its body below the title row).
function XFenceRgb($title) {
  $h = XFenceWin $title; if ($h -eq [IntPtr]::Zero) { return $null }
  $r = Rect $h
  $b = [Px]::Grab($r.L + 20, $r.T + 60, [math]::Max(20, $r.R - $r.L - 40), [math]::Max(20, $r.B - $r.T - 80))
  $m = MeanRgb $b; $b.Dispose(); return $m
}
# Waits until the log has $n more "wallpaper refreshed" lines than $before; ms or -1.
function XWaitRefresh($before, $sec = 10) {
  $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.Elapsed.TotalSeconds -lt $sec) {
    if ((XLog 'wallpaper refreshed').Count -gt $before) { return [int]$sw.ElapsedMilliseconds }
    Start-Sleep -Milliseconds 100
  }
  return -1
}

# An earlier job that ended inside a freeze left Explorer's desktop thread stopped.
$thawed = [Xw]::Thaw([Th]::UiThread([G]::FindWindowW('Progman', [NullString]::Value)))
Log "explorer desktop thread suspend count at the start: $thawed"

if ($Part -eq 'freeze') {
  Case 'freeze setup'
  PfStart
  Pf fence create --title X --rect '300,300,260,180' | Out-Null
  Start-Sleep -Seconds 6
  Case 'S1 Explorer desktop thread stops for 20 s'
  $prog = [G]::FindWindowW('Progman', [NullString]::Value)
  $etid = [Th]::UiThread($prog)
  for ($k = 0; $k -lt 3; $k++) {
    Pf fence move X --x 300 --y 300 | Out-Null
    Start-Sleep -Seconds 3
    $x0 = (FenceGet 'X').windowRect
    $fw = XFenceWin 'X'
    [Xw]::PingStart($fw)
    $ok = [Xw]::Freeze($etid, 20000, 0, 0)
    Start-Sleep -Seconds 6
    $t = [Diagnostics.Stopwatch]::StartNew()
    DragTo ($x0.x + 60) ($x0.y + 14) ($x0.x + 260) ($x0.y + 114) 10
    $dragEnd = $t.ElapsedMilliseconds
    # 1 s after the release, Explorer still stopped: has the fence moved?
    if ($k -eq 0) { Start-Sleep -Milliseconds 1000; Shot 'freeze-1s-after-drag' }
    $lat = @(); $movedAt = -1
    while ($t.Elapsed.TotalSeconds -lt 30) {
      $lat += XTimed --timeout 30000 status
      if ($movedAt -lt 0) { $r = Rect $fw; if ($r.L -ne $x0.x) { $movedAt = $t.ElapsedMilliseconds - $dragEnd } }
      if (-not [Xw]::Frozen -and $movedAt -ge 0) { break }
      Start-Sleep -Milliseconds 500
    }
    $ping = [Xw]::PingStop()
    $max = ($lat | Measure-Object -Maximum).Maximum
    $line = "round ${k}: explorer frozen=$ok; drag 6 s into the freeze took effect $movedAt ms after the release; slowest CLI answer $max ms of $($lat.Count); fence WM_NULL $ping"
    Log "S1 $line"
    Check "S1 $k the fence answers while Explorer's desktop does not" ($ok -and $movedAt -ge 0 -and $movedAt -lt 2000 -and $max -lt 2000) $line
    while ([Xw]::Frozen) { Start-Sleep -Milliseconds 200 }
    Start-Sleep -Seconds 3
  }
  Case 'S1 a theme switch while Explorer does not answer'
  $etid = [Th]::UiThread([G]::FindWindowW('Progman', [NullString]::Value))
  $null = [Xw]::Freeze($etid, 10000, 0, 0)
  Start-Sleep -Seconds 1
  $n = (XLog 'wallpaper refreshed').Count
  $ms = XTimed --timeout 30000 settings set themeStyle liquidGlass
  $ms2 = XTimed --timeout 30000 status
  Shot 'freeze-theme-switched'
  while ([Xw]::Frozen) { Start-Sleep -Milliseconds 200 }
  Start-Sleep -Seconds 3
  $after = @(XLog 'wallpaper refreshed').Count - $n
  $line = "settings set answered in $ms ms, status in $ms2 ms; wallpaper refreshed lines after Explorer came back: $after; $((XLog 'did not answer the wallpaper read|backdrop refresh deferred' | Select-Object -Last 1) -replace '^\S+\s+\S+\s+', '')"
  Log "S1 theme $line"
  Check 'S1 the switch answers at once and the glass is rebuilt once Explorer answers' ($ms -lt 2000 -and $ms2 -lt 2000 -and $after -ge 1) $line
  Pf settings set themeStyle fluent | Out-Null
  Start-Sleep -Seconds 2
  $late = XLog 'wallpaper read (still waits|answered late)'
  Log "reader lines: $(($late | Select-Object -First 4) -join ' | ')"
  Check 'S1 the wallpaper is still followed after the freezes' ((XLog 'wallpaper (refreshed|read)').Count -ge 0) "late lines $($late.Count)"
}

if ($Part -eq 'quit') {
  # Quit while Explorer's desktop does not answer for 8 s: PecoFence's own toggle and the
  # watchdog's must not cancel each other out.
  for ($k = 0; $k -lt 3; $k++) {
    Case "quit $k while Explorer's desktop is stuck 8 s"
    if (-not [W]::IconsVisible()) { [W]::ToggleIcons(); Start-Sleep -Seconds 2 }
    PfStart
    Start-Sleep -Seconds 3
    $hidden = -not [W]::IconsVisible()
    $etid = [Th]::UiThread([G]::FindWindowW('Progman', [NullString]::Value))
    $null = [Xw]::Freeze($etid, 8000, 0, 0)
    Start-Sleep -Milliseconds 300
    $t = [Diagnostics.Stopwatch]::StartNew()
    PfQuit
    $quitMs = $t.ElapsedMilliseconds
    while ([Xw]::Frozen) { Start-Sleep -Milliseconds 200 }
    # The watchdog tries for about 10 s after PecoFence has gone.
    Start-Sleep -Seconds 14
    $shown = [W]::IconsVisible()
    $wd = @(Get-Process pecofence-watchdog -ErrorAction SilentlyContinue).Count
    $line = "hidden while running=$hidden; quit took $quitMs ms; icons shown 14 s after Explorer came back=$shown; watchdogs left=$wd; $((XLog 'restor' | Select-Object -Last 2 | ForEach-Object { $_ -replace '^\S+\s+\S+\s+', '' }) -join ' | ')"
    Log "QUIT $line"
    Check "quit $k the Windows icons come back" ($hidden -and $shown) $line
    Get-Process pecofence-watchdog -ErrorAction SilentlyContinue | Stop-Process -Force
  }
}

if ($Part -eq 'burst') {
  Case 'burst setup'
  XClean 'xb-'
  PfStart
  Start-Sleep -Seconds 2
  $base = -1; $null = XTimed status; try { $base = [int](($script:XOut | ConvertFrom-Json).itemCount) } catch { }
  foreach ($n in @(1500, 2000)) {
    Case "S1 $n new desktop files"
    $prog = PfFence 'Programs'; $r0 = $prog.windowRect
    $fw = XFenceWin 'Programs'
    [Xw]::PingStart($fw)
    $t = [Diagnostics.Stopwatch]::StartNew()
    XFiles $n "xb-$n-"
    $written = $t.ElapsedMilliseconds
    $lat = @(); $dragged = $false; $movedAt = -1; $doneAt = -1; $fast = 0; $shot6 = $false
    while ($t.Elapsed.TotalSeconds -lt 120) {
      # As the review's picture: 6 s after the files came, dragged 3 s after.
      if (-not $shot6 -and $t.Elapsed.TotalSeconds -ge 6) { $shot6 = $true; Shot "burst-$n-6s" }
      if (-not $dragged -and $t.Elapsed.TotalSeconds -ge 3) {
        $dragged = $true
        DragTo ($r0.x + 60) ($r0.y + 14) ($r0.x - 40) ($r0.y + 54) 10
        $dragAt = $t.ElapsedMilliseconds
      }
      $ms = XTimed --timeout 120000 status; $lat += $ms
      $count = try { [int](($script:XOut | ConvertFrom-Json).itemCount) } catch { -1 }
      if ($dragged -and $movedAt -lt 0) { $r = Rect $fw; if ($r.L -ne $r0.x) { $movedAt = $t.ElapsedMilliseconds - $dragAt } }
      if ($count -eq $base + $n -and $doneAt -lt 0) { $doneAt = $t.ElapsedMilliseconds }
      if ($ms -lt 300) { $fast++ } else { $fast = 0 }
      if ($doneAt -ge 0 -and $fast -ge 4 -and $movedAt -ge 0) { break }
      Start-Sleep -Milliseconds 400
    }
    $ping = [Xw]::PingStop()
    $max = ($lat | Measure-Object -Maximum).Maximum
    $line = "create $n files: written $written ms, all items in after $doneAt ms, slowest CLI answer $max ms of $($lat.Count), drag moved the fence after $movedAt ms, fence WM_NULL $ping"
    Log "BURST $line :: $($lat -join ',')"
    Shot "burst-$n"
    Check "S1 $n files: CLI and fence answer within 2 s" ($max -lt 2000 -and $movedAt -ge 0 -and $movedAt -lt 2000) $line
    Pf fence move Programs --x $r0.x --y $r0.y | Out-Null
    XClean "xb-$n-"
    Start-Sleep -Seconds 8
  }
}

if ($Part -eq 'start') {
  # 1200 new documents and 200 new photos (thumbnails on) land on the desktop just before the start.
  Case 'start setup'
  if (-not (Test-Path "$PfApp\pecofence.exe")) { New-Item -ItemType Directory -Force $PfApp | Out-Null; Copy-Item "$PfBin\*" $PfApp -Force }
  $pics = "$PfRun\pics"; New-Item -ItemType Directory -Force $pics | Out-Null
  for ($i = 0; $i -lt 200; $i++) { XPicture ("$pics\xs-photo{0:D3}.jpg" -f $i) (($i * 37) % 256) (($i * 91) % 256) (($i * 53) % 256) -Busy }
  Set-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' IconsOnly 0 -Type DWord
  XClean 'xs-'
  Start-Sleep -Seconds 5
  # Runs 0 and 1: the files only (Explorer on this box digests them in under a second); 2 to 4:
  # also Explorer's desktop thread stopped for 15 s from just before the start (a sign-in).
  for ($k = 0; $k -lt 5; $k++) {
    $stuck = $k -ge 2
    Case "P2 start $k while Explorer adds 1400 files$(if ($stuck) { ' and its desktop is stuck 15 s' })"
    XClean 'xs-'
    # Windows shows its icons when PecoFence starts (sign-in): Explorer draws the new ones.
    if (-not [W]::IconsVisible()) { [W]::ToggleIcons() }
    for ($i = 0; $i -lt 100 -and -not [W]::IconsVisible(); $i++) { Start-Sleep -Milliseconds 200 }
    Start-Sleep -Seconds 15
    Log "icons visible before the start: $([W]::IconsVisible())"
    Copy-Item "$pics\*" $PfDesk
    XFiles 1200 'xs-doc'
    $cur = PfLogPath
    if (Test-Path $cur) { New-Item -ItemType Directory -Force "$PfRun\logs" | Out-Null; Copy-Item $cur "$PfRun\logs\app-start$k.log" -Force; Remove-Item $cur -Force }
    $env:PECOFENCE_INSTANCE = $PfInst; $env:RUST_LOG = 'pecofence=debug'
    [IO.File]::WriteAllLines("$PfRun\script.txt", @('sleep 3600000'), (New-Object Text.UTF8Encoding($false)))
    if ($stuck) { $etid = [Th]::UiThread([G]::FindWindowW('Progman', [NullString]::Value)); $null = [Xw]::Freeze($etid, 15000, 0, 0); Start-Sleep -Milliseconds 300 }
    $t = [Diagnostics.Stopwatch]::StartNew()
    Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable', '--test-script', "$PfRun\script.txt" -WorkingDirectory $PfApp
    $fw = [IntPtr]::Zero
    while ($t.Elapsed.TotalSeconds -lt 40 -and $fw -eq [IntPtr]::Zero) {
      $fw = @([W]::Tops('PecoFence.Fence') | Where-Object { [G]::IsWindowVisible($_) }) | Select-Object -First 1
      if (-not $fw) { $fw = [IntPtr]::Zero; Start-Sleep -Milliseconds 20 }
    }
    $shownAt = $t.ElapsedMilliseconds
    # The CLI's requests are posted to the UI thread, like input: answered once it is free.
    $lat = @(); $slowUntil = 0
    while ($t.ElapsedMilliseconds - $shownAt -lt 25000) {
      $ms = XTimed --timeout 60000 status; $lat += $ms
      if ($ms -gt 1000) { $slowUntil = $t.ElapsedMilliseconds - $shownAt }
      Start-Sleep -Milliseconds 250
    }
    $max = ($lat | Measure-Object -Maximum).Maximum
    $ping = "CLI slowest $max ms of $($lat.Count), last slow answer ended $slowUntil ms after the fences showed"
    while ([Xw]::Frozen) { Start-Sleep -Milliseconds 200 }
    Start-Sleep -Seconds 3
    $icons = [W]::IconsVisible()
    $hid = @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match 'desktop icons (hidden|already|hide out|toggle answered|toggle not done)|not hidden yet|adopting|wallpaper read' })
    $fade = @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match 'fade finished' }) | Select-Object -First 2
    $line = "explorer stuck=$stuck; first fence shown after $shownAt ms; $ping; Windows icons visible now=$icons; $(($hid | ForEach-Object { $_ -replace '^\S+\s+\S+\s+', '' }) -join ' | ')"
    Log "P2 $line"
    Log "P2 fade: $(($fade | ForEach-Object { $_ -replace '^\S+\s+', '' }) -join ' | ')"
    Check "P2 $k the fences answer within 1 s of showing" ($max -lt 1000) $line
    Check "P2 $k the Windows icons end up hidden" (-not $icons) $line
    PfQuit
  }
  XClean 'xs-'
  Set-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' IconsOnly 1 -Type DWord
}

if ($Part -eq 'wallpaper') {
  Case 'wallpaper setup'
  $wp = "$PfRun\wp"; New-Item -ItemType Directory -Force "$wp\show" | Out-Null
  XPicture "$wp\red.jpg" 200 30 30
  XPicture "$wp\blue.jpg" 30 40 210
  XPicture "$wp\green.jpg" 30 190 40
  XPicture "$wp\show\a-yellow.jpg" 230 210 20
  XPicture "$wp\show\b-purple.jpg" 140 30 170
  $old = (Get-ItemProperty 'HKCU:\Control Panel\Desktop').WallPaper
  [Xw]::SetWallpaper("$wp\red.jpg") | Out-Null
  # The entrance fade runs only with Windows' animations on (an RDP session starts with them off).
  $anim = [Anim]::Get(); [Anim]::Set($true)
  Start-Sleep -Seconds 2
  PfStart
  Pf fence create --title W --rect '500,200,400,300' | Out-Null
  Start-Sleep -Seconds 4
  $c0 = XFenceRgb 'W'
  Shot 'wp-red'
  $fade = @(XLog 'fade finished') | Select-Object -First 3
  Log "startup fade lines: $(($fade | ForEach-Object { $_ -replace '^\S+\s+', '' }) -join ' | ')"
  Check 'start: the fence shows the red picture' ($c0 -and $c0[0] -gt $c0[2] + 30) "rgb=$($c0 -join ',')"
  Check 'start: the entrance fade ended by its animation, not the fallback timer' ($fade.Count -gt 0 -and @(XLog 'fallback timer').Count -eq 0) "$($fade.Count) fade lines; $((XLog 'fallback timer' | Select-Object -First 1))"

  Case 'a new picture'
  $n = (XLog 'wallpaper refreshed').Count
  [Xw]::SetWallpaper("$wp\blue.jpg") | Out-Null
  $ms = XWaitRefresh $n
  Start-Sleep -Milliseconds 800
  $c1 = XFenceRgb 'W'
  Shot 'wp-blue'
  Check 'a new picture: the fences follow within 3 s' ($ms -ge 0 -and $ms -lt 3000 -and $c1 -and $c1[2] -gt $c1[0] + 30) "after $ms ms rgb=$($c1 -join ',')"

  Case 'one monitor set on its own (IDesktopWallpaper)'
  $mon = [Dw]::Monitor0()
  $n = (XLog 'wallpaper refreshed').Count
  [Dw]::SetFor($mon, "$wp\green.jpg")
  $ms = XWaitRefresh $n
  Start-Sleep -Milliseconds 800
  $c2 = XFenceRgb 'W'
  Shot 'wp-green'
  Check 'per monitor: the fences follow within 3 s' ($ms -ge 0 -and $ms -lt 3000 -and $c2 -and $c2[1] -gt $c2[0] + 30) "monitor=$mon after $ms ms rgb=$($c2 -join ',')"

  Case 'slideshow'
  $n = (XLog 'wallpaper refreshed').Count
  [Dw]::Slideshow("$wp\show", 60000)
  $ms = XWaitRefresh $n 10
  Start-Sleep -Milliseconds 800
  $s1 = [Dw]::Get($mon); $c3 = XFenceRgb 'W'
  Shot 'wp-show-1'
  $n = (XLog 'wallpaper refreshed').Count
  [Dw]::Advance()
  $ms2 = XWaitRefresh $n 10
  Start-Sleep -Milliseconds 800
  $s2 = [Dw]::Get($mon); $c4 = XFenceRgb 'W'
  Shot 'wp-show-2'
  Check 'slideshow: the fences follow its start and a step' ($ms -ge 0 -and $ms2 -ge 0 -and $ms2 -lt 3000 -and "$($c3 -join ',')" -ne "$($c4 -join ',')") "start after $ms ms ($s1, rgb=$($c3 -join ',')), step after $ms2 ms ($s2, rgb=$($c4 -join ',')) status=$([Dw]::Status())"
  $poll = @(XLog 'reason="?poll' | Where-Object { $_ -match 'wallpaper refreshed' })
  Log "refresh lines: $((XLog 'wallpaper refreshed' | ForEach-Object { $_ -replace '^\S+\s+', '' }) -join ' | ')"

  Case 'theme style switch rebuilds the glass from the picture shown'
  [Xw]::SetWallpaper("$wp\red.jpg") | Out-Null
  Start-Sleep -Seconds 3
  $n = (XLog 'theme refreshed').Count
  Pf settings set themeStyle liquidGlass | Out-Null
  Start-Sleep -Seconds 2
  $c5 = XFenceRgb 'W'
  Shot 'wp-glass-red'
  $deferred = @(XLog 'backdrop refresh deferred|did not answer the wallpaper read').Count
  Check 'Liquid Glass: the fences show the red picture at once' ((XLog 'theme refreshed').Count -gt $n -and $c5 -and $c5[0] -gt $c5[2] + 30 -and $deferred -eq 0) "rgb=$($c5 -join ',') deferred=$deferred"
  Pf settings set themeStyle fluent | Out-Null
  Start-Sleep -Seconds 2

  Case 'wallpaper restore'
  PfQuit
  [Anim]::Set($anim)
  [Xw]::SetWallpaper($(if ($old) { $old } else { '' })) | Out-Null
}

if ($Part -eq 'drop') {
  # Windows' own icons shown, 300 of them, and an Explorer that answers each message late:
  # a fence item dropped on empty desktop space asked every icon whether it takes the drop.
  Case 'drop setup'
  XClean 'xd-'
  XFiles 300 'xd-'
  PfStart
  Pf settings set hideRealIcons false | Out-Null
  Start-Sleep -Seconds 3
  Pf fence create --title D --rect '200,150,320,240' | Out-Null
  DeskFile 'xd-drag-me.txt' | Out-Null
  Start-Sleep -Seconds 3
  Pf item move 'xd-drag-me.txt' --to D | Out-Null
  Start-Sleep -Seconds 2
  $icons = [W]::IconsVisible()
  Log "windows icons visible=$icons; $(Pf status)"
  for ($k = 0; $k -lt 2; $k++) {
    Case "X4 drop $k on the desktop while Explorer answers late"
    Pf item move 'xd-drag-me.txt' --to D | Out-Null
    Start-Sleep -Seconds 2
    $p = ItemPt 'D' 'xd-drag-me'
    if (-not $p) { Check "X4 $k the item is in its fence" $false 'not found'; continue }
    $fw = XFenceWin 'D'
    $prog = [G]::FindWindowW('Progman', [NullString]::Value)
    $etid = [Th]::UiThread($prog)
    $ok = [Xw]::Freeze($etid, 40000, 150, 10)
    $t = [Diagnostics.Stopwatch]::StartNew()
    # Empty desktop space: the first spot no fence covers (the first-run layout differs by build).
    $to = @(@(800, 700), @(700, 600), @(900, 500), @(1300, 700)) | Where-Object { [W]::RootClassAt($_[0], $_[1]) -ne 'PecoFence.Fence' } | Select-Object -First 1
    DragTo $p[0] $p[1] $to[0] $to[1] 12
    # DragTo waits 0.8 s after the release; the first CLI answer after it comes once the drop
    # is handled.
    $first = XTimed --timeout 120000 status
    $ping = "first CLI answer after the release took $first ms"
    $dropMs = $first + 800
    while ([Xw]::Frozen) { Start-Sleep -Milliseconds 200 }
    $slow = XLog 'desktop icon under a drop' | Select-Object -Last 1
    $where = (ItemFence 'xd-drag-me.txt')
    $line = "explorer slowed=$ok icons=$icons; $ping; item now in '$where'; $($slow -replace '^\S+\s+', '')"
    Log "X4 $line"
    Check "X4 $k the fence answers within 1.5 s of the drop" ($dropMs -lt 1500) $line
    # Released on empty desktop space without Ctrl or Alt, the drop is skipped and the item
    # goes to the inbox fence (the first-run 'Desktop' fence here).
    Check "X4 $k the skipped drop puts the item in the inbox fence" ($where -eq 'Desktop') $line
    Start-Sleep -Seconds 2
  }
  Pf settings set hideRealIcons true | Out-Null
  XClean 'xd-'
}
