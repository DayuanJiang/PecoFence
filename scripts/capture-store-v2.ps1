param([string[]]$Scenes = @("overview","dark","tabs","peek","auto","portal"))
$ErrorActionPreference = "Stop"
$workspace = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$fixtureRoot = Join-Path $workspace "extras\pecofence-promo\.capture\reviewed\store-v2"
$assetRoot = Join-Path $workspace ".cache\store-v2"
$outputRoot = Join-Path $assetRoot "native"
New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class StoreCapture {
 public delegate bool EnumProc(IntPtr h,IntPtr l);
 public struct Rect { public int Left,Top,Right,Bottom; }
 [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr p);
 [DllImport("user32.dll")] public static extern uint GetDpiForSystem();
 [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc p,IntPtr l);
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h,out uint p);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h,StringBuilder s,int n);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr h,StringBuilder s,int n);
 [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h,IntPtr a,int x,int y,int w,int t,uint f);
 [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h,int n);
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h,out Rect r);
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
 [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 public static string Title(IntPtr h) { var s=new StringBuilder(300);GetWindowText(h,s,300);return s.ToString(); }
 public static string Class(IntPtr h) { var s=new StringBuilder(150);GetClassName(h,s,150);return s.ToString(); }
 public static List<IntPtr> Windows(uint pid) { var r=new List<IntPtr>();EnumWindows((h,l)=>{uint p;GetWindowThreadProcessId(h,out p);if(p==pid)r.Add(h);return true;},IntPtr.Zero);return r; }
}
"@
[StoreCapture]::SetProcessDpiAwarenessContext([IntPtr](-4)) | Out-Null
$screen = [Windows.Forms.Screen]::PrimaryScreen.Bounds
$work = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea
$scale = [StoreCapture]::GetDpiForSystem() / 96.0
$plateW = 2560; $plateH = 1440
if ($screen.Width -lt $plateW -or $screen.Height -lt $plateH) { throw "Native capture requires a 2560x1440 display; found $screen" }
$plateX = [int](($screen.Width-$plateW)/2); $plateY = [int](($screen.Height-$plateH)/2)
$binary = Join-Path $workspace "extras\pecofence-promo\.capture\reviewed-build\target-auto\debug\pecofence.exe"
$hash = (Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash.ToLowerInvariant()
if ($hash -ne "5db5d8aba634a52736dff716f1be86bd3522120dabe42e16468ef5588c0850af") { throw "Review the fixture build before capturing an unknown executable." }
$previous = [StoreCapture]::GetForegroundWindow()
$saved = @{instance=$env:PECOFENCE_INSTANCE; test=$env:PECOFENCE_UI_TEST_WINDOWS; fixture=$env:PECOFENCE_DEMO_DESKTOP; local=$env:LOCALAPPDATA}
$manifest = [Collections.Generic.List[object]]::new()

function Save-Frame([string]$Name) {
    $bitmap = [Drawing.Bitmap]::new($plateW,$plateH)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen($plateX,$plateY,0,0,$bitmap.Size)
        $bitmap.Save((Join-Path $outputRoot "$Name.png"),[Drawing.Imaging.ImageFormat]::Png)
    } finally { $graphics.Dispose(); $bitmap.Dispose() }
}

try {
 foreach ($scene in $Scenes) {
    if ($scene -notin @("overview","dark","tabs","peek","auto","portal")) { throw "Unknown scene" }
    $caseDir = Join-Path $fixtureRoot $scene
    $appDir = Join-Path $caseDir "app"
    $config = Get-Content -LiteralPath (Join-Path $caseDir "template.json") -Raw | ConvertFrom-Json -AsHashtable
    $fingerprint = @(@{devicePath=[Windows.Forms.Screen]::PrimaryScreen.DeviceName; workDip=@(($work.Width/$scale),($work.Height/$scale)); dpi=[int]($scale*96)})
    $config.layouts[0].fingerprint = $fingerprint
    foreach ($f in $config.layouts[0].fences) {
        $g=$f.geometry
        $g.monitor=$fingerprint[0].devicePath
        $g.x=($plateX+$g.x)/$scale; $g.y=($plateY+$g.y)/$scale
        $g.w=$g.w/$scale; $g.h=$g.h/$scale
        $g.workW=$work.Width/$scale; $g.workH=$work.Height/$scale
        $f.expandedH=$g.h
    }
    $configPath = Join-Path $appDir "config\config.json"
    $config | ConvertTo-Json -Depth 35 | Set-Content -LiteralPath $configPath -Encoding utf8
    Copy-Item -LiteralPath $binary -Destination (Join-Path $appDir "pecofence.exe")
    Copy-Item -LiteralPath (Join-Path $workspace "target\release\WebView2Loader.dll") -Destination (Join-Path $appDir "WebView2Loader.dll")
    $wallpaper = Join-Path $assetRoot $(if($scene -in @("dark","peek")){"paper-dark.png"}else{"paper-light.png"})
    $back = [Windows.Forms.Form]::new()
    $back.FormBorderStyle="None"; $back.StartPosition="Manual"; $back.ShowInTaskbar=$false
    $back.Bounds=$screen; $back.Text="PecoFence Store v2 isolated backdrop"
    $back.BackgroundImage=[Drawing.Image]::FromFile($wallpaper); $back.BackgroundImageLayout="Stretch"
    $app=$null; $board=$null; $done=@{}; $shown=@{}
    try {
        $back.Show()
        [StoreCapture]::SetWindowPos($back.Handle,[IntPtr](-1),0,0,0,0,0x13) | Out-Null
        [Windows.Forms.Application]::DoEvents()
        $env:PECOFENCE_INSTANCE="store-v2-$scene"
        $env:PECOFENCE_UI_TEST_WINDOWS="1"
        $env:PECOFENCE_DEMO_DESKTOP=Join-Path $caseDir "desktop-fixture"
        $env:LOCALAPPDATA=Join-Path $caseDir "appdata"
        New-Item -ItemType Directory -Force -Path $env:LOCALAPPDATA | Out-Null
        $args=@("--portable","--no-hide-icons","--wallpaper",('"' + $wallpaper + '"'),"--test-script",('"' + (Join-Path $caseDir "commands.txt") + '"'),"--exit-after","20000")
        $app=Start-Process -FilePath (Join-Path $appDir "pecofence.exe") -ArgumentList $args -WindowStyle Hidden -PassThru
        $timer=[Diagnostics.Stopwatch]::StartNew()
        while (-not $app.HasExited -and $timer.Elapsed.TotalSeconds -lt 18) {
            [Windows.Forms.Application]::DoEvents()
            foreach ($window in [StoreCapture]::Windows([uint32]$app.Id)) {
                $title=[StoreCapture]::Title($window)
                if ($title -eq "Capture inbox") { [StoreCapture]::ShowWindow($window,0)|Out-Null;continue }
                if ([StoreCapture]::Class($window) -eq "PecoFence.Fence") {
                    if (-not $shown[$window.ToInt64()] -and $timer.Elapsed.TotalSeconds -gt 1.8) {
                        [StoreCapture]::ShowWindow($window,4)|Out-Null
                        $shown[$window.ToInt64()]=$true
                    }
                    if ($scene -ne "peek" -or $timer.Elapsed.TotalSeconds -gt 7) { [StoreCapture]::SetWindowPos($window,[IntPtr](-1),0,0,0,0,0x13)|Out-Null }
                    if ($timer.Elapsed.TotalSeconds -gt 9.2) { [StoreCapture]::PostMessage($window,0x2A3,[IntPtr]::Zero,[IntPtr]::Zero)|Out-Null }
                }
            }
            $t=$timer.Elapsed.TotalSeconds
            if ($scene -eq "peek" -and $t -gt 2.5 -and -not $done.board) {
                $done.board=$true
                $board=[Windows.Forms.Form]::new()
                $board.FormBorderStyle="None";$board.StartPosition="Manual";$board.ShowInTaskbar=$false
                $board.Bounds=[Drawing.Rectangle]::new(($plateX+100),($plateY+200),2360,1120)
                $board.Text="Terra — Creative brief"
                $board.BackgroundImage=[Drawing.Image]::FromFile((Join-Path $assetRoot "brief-window.png"));$board.BackgroundImageLayout="Stretch"
                $board.Show()
                [StoreCapture]::SetWindowPos($back.Handle,[IntPtr](-2),0,0,0,0,0x13)|Out-Null
                [StoreCapture]::SetWindowPos($board.Handle,[IntPtr]::Zero,0,0,0,0,0x13)|Out-Null
                [StoreCapture]::SetForegroundWindow($board.Handle)|Out-Null
            }
            if ($scene -eq "auto" -and $t -gt 6 -and -not $done.newFiles) {
                Save-Frame "auto-before"
                $done.newFiles=$true
                foreach ($name in @("Form study.png","Launch checklist.md")) {
                    Copy-Item -LiteralPath (Join-Path $assetRoot "studies\$name") -Destination (Join-Path $env:PECOFENCE_DEMO_DESKTOP $name)
                }
            }
            if ($scene -eq "tabs" -and $t -gt 7 -and -not $done.work) { Save-Frame "tabs-projects";$done.work=$true }
            if ($scene -eq "tabs" -and $t -gt 11 -and -not $done.art) { Save-Frame "tabs-inspiration";$done.art=$true }
            if ($scene -ne "tabs" -and $t -gt 10 -and -not $done.frame) { Save-Frame $scene;$done.frame=$true }
            Start-Sleep -Milliseconds 60
        }
        if (-not $app.HasExited) { $app.WaitForExit(4000)|Out-Null }
        $state=Get-Content -LiteralPath $configPath -Raw|ConvertFrom-Json -AsHashtable
        foreach ($item in $state.items.Values) {
            if (-not $item.key.path.StartsWith($fixtureRoot.ToLowerInvariant())) { throw "Unexpected non-demo file appeared in the capture catalog." }
        }
        Copy-Item -LiteralPath $configPath -Destination (Join-Path $outputRoot "$scene-state.json")
        $manifest.Add(@{scene=$scene;capturedAt=[DateTime]::UtcNow.ToString("o");sourceBinarySha256=$hash;themeStyle=$config.settings.themeStyle;theme=$config.settings.theme;dpi=[int]($scale*96);plate=@($plateX,$plateY,$plateW,$plateH);catalogItems=$state.items.Count;exitCode=$app.ExitCode;fixtureOnly=$true})
        Write-Output "Captured $scene; isolated catalog=$($state.items.Count), exit=$($app.ExitCode)"
    } finally {
        if ($app -and -not $app.HasExited) { $app.WaitForExit(5000)|Out-Null }
        if ($board) { $board.Close();$board.BackgroundImage.Dispose();$board.Dispose() }
        $back.Close();$back.BackgroundImage.Dispose();$back.Dispose()
    }
 }
} finally {
    $env:PECOFENCE_INSTANCE=$saved.instance;$env:PECOFENCE_UI_TEST_WINDOWS=$saved.test
    $env:PECOFENCE_DEMO_DESKTOP=$saved.fixture;$env:LOCALAPPDATA=$saved.local
    [StoreCapture]::SetForegroundWindow($previous)|Out-Null
    $manifest|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $outputRoot "capture-manifest.json") -Encoding utf8
}
