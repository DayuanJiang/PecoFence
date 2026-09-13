param([switch]$PrepareOnly)
$ErrorActionPreference = "Stop"
$projectDir = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$workspaceDir = (Resolve-Path (Join-Path $projectDir "..\\..")).Path
$captureDir = Join-Path $projectDir ".capture"
$publicDir = Join-Path $projectDir "public"
$demoDir = Join-Path $captureDir "demo"
New-Item -ItemType Directory -Force -Path $demoDir,(Join-Path $demoDir "config") | Out-Null
Copy-Item -LiteralPath (Join-Path $workspaceDir "target\debug\pecofence.exe") -Destination (Join-Path $demoDir "pecofence.exe")
Copy-Item -LiteralPath (Join-Path $workspaceDir "target\release\WebView2Loader.dll") -Destination (Join-Path $demoDir "WebView2Loader.dll")

Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms
Add-Type @"
using System;
using System.Text;
using System.Runtime.InteropServices;
using System.Collections.Generic;
public class PromoNative {
 [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr c);
 [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr a,int x,int y,int w,int z,uint f);
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h,out Rect r);
 [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc p,IntPtr l);
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h,out uint p);
 [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h,int n);
 [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h,StringBuilder s,int n);
 [DllImport("shell32.dll",CharSet=CharSet.Unicode)] public static extern IntPtr SHGetFileInfo(string path,uint attr,ref SHINFO info,uint cb,uint flags);
 [DllImport("user32.dll")] public static extern bool DestroyIcon(IntPtr h);
 public delegate bool EnumProc(IntPtr h,IntPtr l);
 public struct Rect { public int Left,Top,Right,Bottom; }
 [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)]
 public struct SHINFO { public IntPtr icon; public int index; public uint attrs; [MarshalAs(UnmanagedType.ByValTStr, SizeConst=260)] public string display; [MarshalAs(UnmanagedType.ByValTStr, SizeConst=80)] public string type; }
 public static List<IntPtr> Windows(uint pid) {
  var a=new List<IntPtr>();
  EnumWindows((h,l)=> {uint p;GetWindowThreadProcessId(h,out p);if(p==pid)a.Add(h);return true;},IntPtr.Zero);
  return a;
 }
 public static string Title(IntPtr h) {var s=new StringBuilder(300);GetWindowText(h,s,300);return s.ToString();}
}
"@
[PromoNative]::SetProcessDpiAwarenessContext([IntPtr](-4)) | Out-Null

$groups = @{
    "Projects" = @("Brand assets","Launch plan","Design notes","Inspiration","Meeting notes","Archive")
    "Creative" = @("Photography","UI design","Brand kit","Video edits","Color studies","Icon library")
    "Documents" = @("Weekly plan.txt","Project notes.txt","Reading list.txt","Meeting notes.txt","Ideas.txt","Travel plans.txt")
}
foreach ($group in $groups.Keys) {
    $folder = Join-Path $captureDir "samples\$group"
    New-Item -ItemType Directory -Force -Path $folder | Out-Null
    foreach ($name in $groups[$group]) {
        $item = Join-Path $folder $name
        if ($name.EndsWith(".txt")) { "PecoFence promotional demo sample." | Set-Content -LiteralPath $item -Encoding utf8 }
        else { New-Item -ItemType Directory -Force -Path $item | Out-Null }
    }
}
function Export-ShellIcon([string]$ItemPath,[string]$OutputPath) {
    $info = [PromoNative+SHINFO]::new()
    [PromoNative]::SHGetFileInfo($ItemPath,0,[ref]$info,[uint32][System.Runtime.InteropServices.Marshal]::SizeOf($info),0x100) | Out-Null
    if ($info.icon -ne [IntPtr]::Zero) {
        $icon = [System.Drawing.Icon]::FromHandle($info.icon)
        $bitmap = $icon.ToBitmap()
        try { $bitmap.Save($OutputPath,[System.Drawing.Imaging.ImageFormat]::Png) }
        finally { $bitmap.Dispose(); $icon.Dispose(); [PromoNative]::DestroyIcon($info.icon) | Out-Null }
    }
}
Export-ShellIcon (Join-Path $captureDir "samples\Projects\Brand assets") (Join-Path $publicDir "folder-icon.png")
Export-ShellIcon (Join-Path $captureDir "samples\Documents\Weekly plan.txt") (Join-Path $publicDir "document-icon.png")

$referencePath = Join-Path $env:APPDATA "PecoFence\config.json"
$reference = Get-Content -LiteralPath $referencePath -Raw | ConvertFrom-Json -AsHashtable
$reference["items"] = @{}
$reference["snapshots"] = @()
$reference["undoLog"] = @()
$reference["rules"] = @{defaultTarget="inbox";keepUpdated=$false;list=@()}
$reference["settings"]["hideRealIcons"] = $false
$reference["settings"]["theme"] = "dark"
$reference["settings"]["peek"]["enabled"] = $false
$reference["settings"]["peek"]["dim"] = $false
$registered = Get-ItemProperty -LiteralPath 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name PecoFence -ErrorAction SilentlyContinue
$reference["settings"]["autostart"] = ($null -ne $registered)
$fingerprint = $reference["layouts"][0]["fingerprint"]
$screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$work = [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea
$scale = $fingerprint[0]["dpi"] / 96.0
$plateW = 2560; $plateH = 1440
$plateX = [int](($screen.Width - $plateW) / 2)
$plateY = [int](($screen.Height - $plateH) / 2)
if ($screen.Width -lt $plateW -or $screen.Height -lt $plateH) { throw "Capture needs a 2560x1440 or larger display. Detected $screen" }

$fences = @()
$titles = @("Projects","Creative","Documents")
for ($n = 0; $n -lt 3; $n++) {
    $fences += @{
        id=[guid]::NewGuid().ToString();title=$titles[$n];kind="folderPortal"
        source=@{kind="folder";path=(Join-Path $captureDir "samples\$($titles[$n])");recursive=$false;filter=$null}
        geometry=@{monitor=$fingerprint[0]["devicePath"];x=($plateX+150+$n*770)/$scale;y=($plateY+440)/$scale;w=720/$scale;h=530/$scale;workW=$work.Width/$scale;workH=$work.Height/$scale;anchor="leftTop"}
        rolledUp=$false;expandedH=530/$scale
        view=@{iconSize=48;sort="name";labelLines=2;autoHeight=$false;reverse=$false;layout="icons";spacing="loose";columnWidths=$null;columnsVisible=$null}
        appearance=@{tintRgb=$null;opacity=0.46;backdrop="acrylic";titleRgb=$null;titleSize="normal"}
        excludeFromQuickHide=$false;locked=$false;tabHost=$null;activeTab=$null;tabOrder=@();portalNavigate=$true;hideTitleIcon=$true;items=@()
    }
}
$fences += @{
 id=[guid]::NewGuid().ToString();title="Demo inbox (excluded from capture)";kind="inbox";source=@{kind="desktop"}
 geometry=@{monitor=$fingerprint[0]["devicePath"];x=0;y=0;w=180;h=80;workW=$work.Width/$scale;workH=$work.Height/$scale;anchor="leftTop"}
 rolledUp=$true;expandedH=80;view=@{iconSize=32;sort="name";labelLines=2;autoHeight=$false};items=@()
}
$reference["layouts"] = @(@{fingerprint=$fingerprint;fences=$fences})
$configPath = Join-Path $demoDir "config\config.json"
$reference | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $configPath -Encoding utf8
$testPath = Join-Path $captureDir "capture.txt"
@"
sleep 1000
pin-test-windows
sleep 6000
merge Creative Projects
sleep 2000
roll Projects
sleep 1300
unroll Projects
sleep 1700
quick-hide
sleep 1200
quick-show
sleep 3500
exit
"@ | Set-Content -LiteralPath $testPath -Encoding utf8
if ($PrepareOnly) { Write-Output "Prepared $configPath"; exit 0 }

function Save-Capture([string]$Name,[int]$X,[int]$Y,[int]$W,[int]$H) {
    $bitmap = [System.Drawing.Bitmap]::new($W,$H)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try { $graphics.CopyFromScreen($X,$Y,0,0,$bitmap.Size); $bitmap.Save((Join-Path $publicDir $Name),[System.Drawing.Imaging.ImageFormat]::Png) }
    finally { $graphics.Dispose(); $bitmap.Dispose() }
}
$foreground = [PromoNative]::GetForegroundWindow()
$backdrop = [System.Windows.Forms.Form]::new()
$backdrop.FormBorderStyle = [System.Windows.Forms.FormBorderStyle]::None
$backdrop.StartPosition = [System.Windows.Forms.FormStartPosition]::Manual
$backdrop.Bounds = [System.Drawing.Rectangle]::new($plateX,$plateY,$plateW,$plateH)
$backdrop.BackgroundImage = [System.Drawing.Image]::FromFile((Join-Path $publicDir "wallpaper.png"))
$backdrop.BackgroundImageLayout = [System.Windows.Forms.ImageLayout]::Stretch
$backdrop.ShowInTaskbar = $false
$backdrop.Text = "PecoFence promotional capture"
$oldInstance = $env:PECOFENCE_INSTANCE
$oldLocalAppData = $env:LOCALAPPDATA
$oldTestWindows = $env:PECOFENCE_UI_TEST_WINDOWS
$appProcess = $null
$recordingProcess = $null
try {
    $backdrop.Show()
    [PromoNative]::SetWindowPos($backdrop.Handle,[IntPtr](-1),0,0,0,0,0x13) | Out-Null
    [System.Windows.Forms.Application]::DoEvents()
    $env:PECOFENCE_INSTANCE = "promo-capture"
    $env:PECOFENCE_UI_TEST_WINDOWS = "1"
    $env:LOCALAPPDATA = Join-Path $captureDir "appdata"
    New-Item -ItemType Directory -Force -Path $env:LOCALAPPDATA | Out-Null
    $arguments = @("--portable","--no-hide-icons","--dark","--wallpaper",('"' + (Join-Path $publicDir "wallpaper.png") + '"'),"--test-script",('"' + $testPath + '"'),"--exit-after","20000")
    $appProcess = Start-Process -FilePath (Join-Path $demoDir "pecofence.exe") -ArgumentList $arguments -WindowStyle Hidden -PassThru
    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    $saved = @{}
    $shownWindows = @{}
    while ($timer.Elapsed.TotalSeconds -lt 18 -and -not $appProcess.HasExited) {
        [System.Windows.Forms.Application]::DoEvents()
        $windows = [PromoNative]::Windows([uint32]$appProcess.Id)
        foreach ($window in $windows) {
            $title = [PromoNative]::Title($window)
            if ($title -like "*Demo inbox*") { [PromoNative]::ShowWindow($window,0) | Out-Null; continue }
            if ($title -in $titles) {
                if (-not $shownWindows[$window.ToInt64()] -and $timer.Elapsed.TotalSeconds -gt 1.5) {
                    [PromoNative]::ShowWindow($window,4) | Out-Null
                    $shownWindows[$window.ToInt64()] = $true
                }
                [PromoNative]::SetWindowPos($window,[IntPtr](-1),0,0,0,0,0x13) | Out-Null
            }
        }
        $t = $timer.Elapsed.TotalSeconds
        if ($t -gt 4 -and -not $saved["all"]) {
            Save-Capture "product-desktop.png" $plateX $plateY $plateW $plateH
            foreach ($window in $windows) {
                $title = [PromoNative]::Title($window)
                if ($title -in $titles) {
                    $rect = [PromoNative+Rect]::new()
                    [PromoNative]::GetWindowRect($window,[ref]$rect) | Out-Null
                    $index = [array]::IndexOf($titles,$title)
                    Save-Capture "panel-$index.png" $rect.Left $rect.Top ($rect.Right-$rect.Left) ($rect.Bottom-$rect.Top)
                }
            }
            $saved["all"]=$true
            $recordArgs = @("-hide_banner","-loglevel","warning","-y","-f","gdigrab","-framerate","30","-draw_mouse","0","-offset_x",$plateX,"-offset_y",$plateY,"-video_size","2560x1440","-i","desktop","-t","12","-c:v","libx264","-preset","ultrafast","-crf","18","-pix_fmt","yuv420p",('"' + (Join-Path $publicDir "product-demo.mp4") + '"'))
            $recordingProcess = Start-Process -FilePath (Get-Command ffmpeg.exe).Source -ArgumentList $recordArgs -WindowStyle Hidden -PassThru -RedirectStandardError (Join-Path $captureDir "recording.log")
        }
        if ($t -gt 8 -and -not $saved["tabs"]) { Save-Capture "product-tabs.png" $plateX $plateY $plateW $plateH; $saved["tabs"]=$true }
        if ($t -gt 10 -and -not $saved["rolled"]) { Save-Capture "product-rolled.png" $plateX $plateY $plateW $plateH; $saved["rolled"]=$true }
        Start-Sleep -Milliseconds 70
    }
    if ($null -ne $recordingProcess) { $recordingProcess.WaitForExit(5000) | Out-Null }
}
finally {
    if ($null -ne $appProcess -and -not $appProcess.HasExited) {
        foreach ($window in [PromoNative]::Windows([uint32]$appProcess.Id)) {
            if ([PromoNative]::Title($window) -eq "PecoFence") { [PromoNative]::PostMessage($window,0x0010,[IntPtr]::Zero,[IntPtr]::Zero) | Out-Null }
        }
        $appProcess.WaitForExit(3000) | Out-Null
    }
    $backdrop.Close()
    $backdrop.BackgroundImage.Dispose()
    $backdrop.Dispose()
    $env:PECOFENCE_INSTANCE = $oldInstance
    $env:LOCALAPPDATA = $oldLocalAppData
    $env:PECOFENCE_UI_TEST_WINDOWS = $oldTestWindows
    [PromoNative]::SetForegroundWindow($foreground) | Out-Null
}
Get-ChildItem -LiteralPath $publicDir | Select-Object Name,Length
