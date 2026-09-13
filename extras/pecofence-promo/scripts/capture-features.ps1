param(
    [string[]]$Scenes = @("groups","rules","portal","tabs","peek","hide"),
    [switch]$Launch
)
$ErrorActionPreference = "Stop"
$projectDir = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$workspaceDir = (Resolve-Path (Join-Path $projectDir "..\..")).Path
$stageDir = Join-Path $projectDir $(if($Launch){".capture\launch"}else{".capture\features"})
$assetDir = Join-Path $projectDir $(if($Launch){"public\launch\raw"}else{"public\features"})
New-Item -ItemType Directory -Force -Path $stageDir,$assetDir | Out-Null
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;using System.Text;using System.Collections.Generic;using System.Runtime.InteropServices;
public static class FeatureNative {
 public delegate bool EnumProc(IntPtr h,IntPtr p);
 public struct Rect {public int Left,Top,Right,Bottom;}
 [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr c);
 [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc p,IntPtr l);
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h,out uint p);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h,StringBuilder s,int n);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr h,StringBuilder s,int n);
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h,out Rect r);
 [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h,IntPtr a,int x,int y,int w,int z,uint f);
 [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h,int n);
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
 [DllImport("user32.dll")] public static extern bool GetCursorPos(out Point p);
 public struct Point {public int X,Y;}
 [DllImport("user32.dll")] public static extern void mouse_event(uint f,uint x,uint y,uint d,UIntPtr e);
 [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("user32.dll",CharSet=CharSet.Unicode,EntryPoint="SendMessageW")] public static extern IntPtr SendText(IntPtr h,uint m,IntPtr w,string l);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern IntPtr FindWindowEx(IntPtr p,IntPtr a,string c,string t);
 [DllImport("user32.dll")] public static extern int GetMenuItemCount(IntPtr m);
 [DllImport("user32.dll")] public static extern uint GetMenuItemID(IntPtr m,int i);
 [DllImport("user32.dll")] public static extern bool GetMenuItemRect(IntPtr h,IntPtr m,uint i,out Rect r);
 public static string Title(IntPtr h){var s=new StringBuilder(512);GetWindowText(h,s,512);return s.ToString();}
 public static string Class(IntPtr h){var s=new StringBuilder(128);GetClassName(h,s,128);return s.ToString();}
 public static IntPtr EditOf(IntPtr parent){return FindWindowEx(parent,IntPtr.Zero,"EDIT",null);}
 public static List<IntPtr> Windows(uint pid){var a=new List<IntPtr>();EnumWindows((h,l)=>{uint p;GetWindowThreadProcessId(h,out p);if(p==pid)a.Add(h);return true;},IntPtr.Zero);return a;}
}
"@
[FeatureNative]::SetProcessDpiAwarenessContext([IntPtr](-4)) | Out-Null
$screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$workArea = [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea
$plateW = 2560; $plateH = 1440
$plateX = [int](($screen.Width-$plateW)/2); $plateY = [int](($screen.Height-$plateH)/2)
if ($screen.Width -lt $plateW -or $screen.Height -lt $plateH) {throw "Display is too small for this capture."}
$reference = Get-Content -LiteralPath (Join-Path $env:APPDATA "PecoFence\config.json") -Raw | ConvertFrom-Json -AsHashtable
$fingerprint = $reference.layouts[0].fingerprint
$scale = $fingerprint[0].dpi/96.0
$desktopDir = [Environment]::GetFolderPath("DesktopDirectory")
$createdDesktopFiles = [System.Collections.Generic.List[string]]::new()
$originalCursor = [FeatureNative+Point]::new()
[FeatureNative]::GetCursorPos([ref]$originalCursor) | Out-Null
$originalForeground = [FeatureNative]::GetForegroundWindow()

function Make-Sample([string]$File,[string]$Kind="text") {
    if (Test-Path -LiteralPath $File) {throw "Refusing to overwrite existing sample path: $File"}
    if ($Kind -eq "image") {
        $bmp=[System.Drawing.Bitmap]::new(640,400)
        $g=[System.Drawing.Graphics]::FromImage($bmp)
        try {
            $g.Clear([System.Drawing.ColorTranslator]::FromHtml("#e8f1fb"))
            $b=[System.Drawing.SolidBrush]::new([System.Drawing.ColorTranslator]::FromHtml("#1579ed"))
            $g.FillRectangle($b,30,30,580,70); $b.Dispose()
            $b=[System.Drawing.SolidBrush]::new([System.Drawing.Color]::White)
            $g.FillRectangle($b,30,130,270,230);$g.FillRectangle($b,330,130,280,90)
            $g.FillRectangle($b,330,240,280,120);$b.Dispose()
            $bmp.Save($File,[System.Drawing.Imaging.ImageFormat]::Png)
        } finally {$g.Dispose();$bmp.Dispose()}
    } else {
        $stream=[System.IO.File]::Open($File,[System.IO.FileMode]::CreateNew)
        try {$bytes=[Text.Encoding]::UTF8.GetBytes("Website launch demo`r`nA sample file prepared for the PecoFence feature video.");$stream.Write($bytes,0,$bytes.Length)}
        finally {$stream.Dispose()}
    }
}
function Desktop-Sample([string]$Name,[string]$Kind="text") {
    $file=Join-Path $desktopDir $Name
    Make-Sample $file $Kind
    $createdDesktopFiles.Add($file)
    return $file
}
function Fence([string]$Title,[int]$X,[int]$Y,[int]$W,[int]$H,[string]$Folder="") {
    return @{
        id=[guid]::NewGuid().ToString(); title=$Title
        kind=$(if($Folder){"folderPortal"}else{"virtual"})
        source=$(if($Folder){@{kind="folder";path=$Folder;recursive=$false;filter=$null}}else{@{kind="desktop"}})
        geometry=@{monitor=$fingerprint[0].devicePath;x=($plateX+$X)/$scale;y=($plateY+$Y)/$scale;w=$W/$scale;h=$H/$scale;workW=$workArea.Width/$scale;workH=$workArea.Height/$scale;anchor="leftTop"}
        rolledUp=$false;expandedH=$H/$scale
        view=@{iconSize=48;sort="name";labelLines=2;autoHeight=$false;reverse=$false;layout="icons";spacing="loose"}
        appearance=@{tintRgb=$null;opacity=0.50;backdrop="acrylic";titleRgb=$null;titleSize="large"}
        excludeFromQuickHide=$false;locked=$false;tabHost=$null;activeTab=$null;tabOrder=@()
        portalNavigate=$true;hideTitleIcon=$false;items=@()
    }
}
function Rule([string]$Name,[string]$Target,[array]$Conditions) {
    return @{id=[guid]::NewGuid().ToString();name=$Name;enabled=$true;target=@{fence=$Target};allOf=$Conditions;priorityClass="type"}
}
function Rect-Of([IntPtr]$Window) {
    $r=[FeatureNative+Rect]::new()
    [FeatureNative]::GetWindowRect($Window,[ref]$r) | Out-Null
    return $r
}
function Find-Fence([string]$Title) {
    foreach($window in [FeatureNative]::Windows([uint32]$appProcess.Id)){
        if([FeatureNative]::Title($window) -eq $Title){return $window}
        if($scene -eq "groups" -and $Title -eq "Work" -and [FeatureNative]::Title($window) -eq "新栅栏"){return $window}
    }
    throw "Demo window not found: $Title"
}
function Mouse-Point([int]$X,[int]$Y) {
    [FeatureNative]::SetCursorPos($X,$Y)|Out-Null
    Start-Sleep -Milliseconds 60
}
function Mouse-Click([int]$X,[int]$Y,[switch]$Right,[switch]$Double) {
    Mouse-Point $X $Y
    $down=if($Right){8}else{2};$up=if($Right){16}else{4}
    [FeatureNative]::mouse_event($down,0,0,0,[UIntPtr]::Zero)
    Start-Sleep -Milliseconds 45
    [FeatureNative]::mouse_event($up,0,0,0,[UIntPtr]::Zero)
    if($Double){Start-Sleep -Milliseconds 80;[FeatureNative]::mouse_event(2,0,0,0,[UIntPtr]::Zero);Start-Sleep -Milliseconds 40;[FeatureNative]::mouse_event(4,0,0,0,[UIntPtr]::Zero)}
}
function Window-Click([IntPtr]$Window,[int]$X,[int]$Y) {
    $r=Rect-Of $Window
    Mouse-Point ($r.Left+$X) ($r.Top+$Y)
    $point=[IntPtr](($Y -shl 16) -bor ($X -band 65535))
    [FeatureNative]::PostMessage($Window,0x200,[IntPtr]::Zero,$point)|Out-Null
    [FeatureNative]::PostMessage($Window,0x201,[IntPtr]1,$point)|Out-Null
    Start-Sleep -Milliseconds 60
    [FeatureNative]::PostMessage($Window,0x202,[IntPtr]::Zero,$point)|Out-Null
}
function Drag([int]$X,[int]$Y,[int]$ToX,[int]$ToY) {
    Mouse-Point $X $Y
    [FeatureNative]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
    Start-Sleep -Milliseconds 130
    for($j=1;$j -le 32;$j++){
        [FeatureNative]::SetCursorPos([int]($X+($ToX-$X)*$j/32),[int]($Y+($ToY-$Y)*$j/32))|Out-Null
        Start-Sleep -Milliseconds 19
    }
    Start-Sleep -Milliseconds 180
    [FeatureNative]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
}
function Select-Menu([uint32]$Id) {
  for($attempt=0;$attempt -lt 20;$attempt++){
    foreach($window in [FeatureNative]::Windows([uint32]$appProcess.Id)){
        if([FeatureNative]::Class($window) -eq "#32768"){
            $menu=[FeatureNative]::SendMessage($window,0x1e1,[IntPtr]::Zero,[IntPtr]::Zero)
            for($i=0;$i -lt [FeatureNative]::GetMenuItemCount($menu);$i++){
                if([FeatureNative]::GetMenuItemID($menu,$i) -eq $Id){
                    $r=[FeatureNative+Rect]::new()
                    [FeatureNative]::GetMenuItemRect([IntPtr]::Zero,$menu,[uint32]$i,[ref]$r)|Out-Null
                    Mouse-Click ([int](($r.Left+$r.Right)/2)) ([int](($r.Top+$r.Bottom)/2))
                    return
                }
            }
        }
    }
    Start-Sleep -Milliseconds 100
  }
    throw "Expected demo menu item $Id was not visible"
}
function Save-Plate([string]$File) {
    $bmp=[System.Drawing.Bitmap]::new($plateW,$plateH)
    $g=[System.Drawing.Graphics]::FromImage($bmp)
    try {$g.CopyFromScreen($plateX,$plateY,0,0,$bmp.Size);$bmp.Save($File,[System.Drawing.Imaging.ImageFormat]::Png)}
    finally {$g.Dispose();$bmp.Dispose()}
}
function At([string]$Name,[double]$Time,[scriptblock]$Action) {
    if($timer.Elapsed.TotalSeconds -ge $Time -and -not $done[$Name]){
        $done[$Name]=$true
        $eventTime=[math]::Round($timer.Elapsed.TotalSeconds-$recordStartedAt,3)
        try {& $Action; $events.Add(@{event=$Name;time=$eventTime;ok=$true})}
        catch {$events.Add(@{event=$Name;time=$eventTime;ok=$false;error=$_.Exception.Message});Write-Warning "$scene / $Name : $_"}
    }
}

try {
 foreach($scene in $Scenes){
    Write-Output "Capturing feature: $scene"
    $caseDir=Join-Path $stageDir $scene
    $demoDir=Join-Path $caseDir "app"
    New-Item -ItemType Directory -Force -Path $caseDir,$demoDir,(Join-Path $demoDir "config")|Out-Null
    Copy-Item -LiteralPath (Join-Path $workspaceDir "target\debug\pecofence.exe") -Destination (Join-Path $demoDir "pecofence.exe")
    Copy-Item -LiteralPath (Join-Path $workspaceDir "target\release\WebView2Loader.dll") -Destination (Join-Path $demoDir "WebView2Loader.dll")
    $config=$reference | ConvertTo-Json -Depth 30 | ConvertFrom-Json -AsHashtable
    $config.items=@{};$config.snapshots=@();$config.undoLog=@()
    $config.settings.hideRealIcons=$false;$config.settings.theme="dark";$config.settings.themeStyle="fluent"
    if($Launch){$config.settings.themeStyle="liquidGlass";$config.settings.language="en"}
    $config.settings.peek.enabled=$false;$config.settings.peek.dim=$true
    $config.settings.rollUp.hoverPeek=$true;$config.settings.rollUp.clickToExpand=$false
    $config.settings.snapping.sizeToCells=$false
    $reg=Get-ItemProperty -LiteralPath "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name PecoFence -ErrorAction SilentlyContinue
    $config.settings.autostart=($null -ne $reg)
    $fences=@();$rules=@()
    $folders=@{}
    foreach($title in @("Work","Media","Project assets")){
        $folder=Join-Path $caseDir $title
        New-Item -ItemType Directory -Force -Path $folder|Out-Null
        $folders[$title]=$folder
    }
    foreach($name in @("Brand","Screenshots")){New-Item -ItemType Directory -Force -Path (Join-Path $folders["Project assets"] $name)|Out-Null}
    foreach($pair in @(@("Work","Launch plan"),@("Work","Meeting notes"),@("Work","Research"),@("Media","Photography"),@("Media","Video edits"),@("Media","Brand kit"))){
        New-Item -ItemType Directory -Force -Path (Join-Path $folders[$pair[0]] $pair[1])|Out-Null
    }
    foreach($sample in @((Join-Path $folders["Project assets"] "brief.txt"),(Join-Path $folders["Project assets"] "Brand\colors.txt"))){
        if(-not(Test-Path -LiteralPath $sample)){Make-Sample $sample}
    }
    if($Launch){
        foreach($name in @("Ideas.txt","Roadmap.txt","Launch brief.txt")){
            $sample=Join-Path $folders["Work"] $name
            if(-not(Test-Path -LiteralPath $sample)){Make-Sample $sample}
        }
        foreach($name in @("Coast","Dunes","Horizon")){
            Copy-Item -LiteralPath (Join-Path $projectDir "public\launch\demo-images\$name.png") -Destination (Join-Path $folders["Media"] "$name.png")
        }
        $oldSample=Join-Path $folders["Project assets"] "Moodboard.png"
        if(Test-Path -LiteralPath $oldSample){
            Move-Item -LiteralPath $oldSample -Destination (Join-Path $folders["Project assets"] "Study.png") -Force
        }
        foreach($name in @("Cover","Palette","Study")){
            Copy-Item -LiteralPath (Join-Path $projectDir "public\launch\demo-images\Horizon.png") -Destination (Join-Path $folders["Project assets"] "$name.png")
        }
    }
    $appCommands=@("sleep 1000","pin-test-windows")
    $caseLength=19.0
    switch($scene){
      "hero" {
        $fences=@(
            (Fence "Work" 160 470 690 580 $folders["Work"]),
            (Fence "Art" 935 470 690 580 $folders["Media"]),
            (Fence "Project assets" 1710 470 690 580 $folders["Project assets"])
        )
        $caseLength=11
      }
      "groups" {
        $source=Fence "Desktop files" 150 420 770 550
        $fences=@($source)
        foreach($name in @("brief-demo.txt","meeting-demo.txt")){Desktop-Sample $name|Out-Null}
        $rules=@((Rule "Demo source files" $source.id @(@{cond="exactName";value=@("brief-demo.txt","meeting-demo.txt")})))
        $appCommands+=@("sleep 5700","new-fence $($plateX+1250) $($plateY+420) 770 550")
        $caseLength=23.0
      }
      "rules" {
        $images=Fence "Images" 230 400 850 620
        $documents=Fence "Documents" 1410 400 850 620
        $fences=@($images,$documents)
        $rules=@(
          (Rule "PNG demo files to Images" $images.id @(@{cond="ext";value=@("png")},@{cond="name";value=@{op="endsWith";value="-demo.png"}})),
          (Rule "Text demo files to Documents" $documents.id @(@{cond="ext";value=@("txt")},@{cond="name";value=@{op="endsWith";value="-demo.txt"}}))
        )
        Desktop-Sample "wireframe-demo.png" "image"|Out-Null
      }
      "portal" {
        $priorNote=Join-Path $folders["Project assets"] "review-notes.txt"
        if(Test-Path -LiteralPath $priorNote){
            $resolvedNote=(Resolve-Path -LiteralPath $priorNote).Path
            if(-not $resolvedNote.StartsWith($caseDir+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)){throw "Unexpected staged sample path"}
            $savedNote=Join-Path $caseDir ("prior-review-notes-"+[guid]::NewGuid().ToString()+".txt")
            Move-Item -LiteralPath $resolvedNote -Destination $savedNote
        }
        $fences=@((Fence "Project assets" 670 350 1180 750 $folders["Project assets"]))
      }
      "tabs" {
        $fences=@((Fence "Work" 200 410 900 610 $folders["Work"]),(Fence "Art" 1390 410 900 610 $folders["Media"]))
        $fences[1].tabHost=$fences[0].id
        $fences[0].activeTab=$fences[0].id
        $fences[0].tabOrder=@($fences[0].id,$fences[1].id)
        $appCommands+=@("sleep 12500","detach Art")
        $caseLength=22
      }
      "peek" {
        $fences=@((Fence "Work" 200 410 900 610 $folders["Work"]),(Fence "Art" 1390 410 900 610 $folders["Media"]))
        $appCommands=@("sleep 8000","peek","sleep 5000","end-peek")
        $caseLength=18
      }
      "hide" {
        $fences=@((Fence "Work" 200 410 900 610 $folders["Work"]),(Fence "Media" 1390 410 900 610 $folders["Media"]))
        $appCommands+=@("sleep 5300","quick-hide","sleep 1900","quick-show","sleep 2000","roll Work")
        $caseLength=18.5
      }
      default {throw "Unknown scene: $scene"}
    }
    $inbox=Fence "Private inbox excluded from film" (-$plateX) (-$plateY) 250 100
    $inbox.kind="inbox";$inbox.rolledUp=$true
    $fences+=@($inbox)
    $config.layouts=@(@{fingerprint=$fingerprint;fences=$fences})
    $config.rules=@{defaultTarget="inbox";keepUpdated=$true;list=$rules}
    $configPath=Join-Path $demoDir "config\config.json"
    $config|ConvertTo-Json -Depth 35|Set-Content -LiteralPath $configPath -Encoding utf8
    $appCommands+=@("sleep 22000","exit")
    $testPath=Join-Path $caseDir "events.txt"
    $appCommands|Set-Content -LiteralPath $testPath -Encoding utf8
    $background=[System.Windows.Forms.Form]::new()
    $background.FormBorderStyle="None";$background.StartPosition="Manual";$background.ShowInTaskbar=$false
    $background.Bounds=[System.Drawing.Rectangle]::new($plateX,$plateY,$plateW,$plateH)
    if($Launch){$background.Bounds=$screen}
    $background.BackgroundImage=[System.Drawing.Image]::FromFile((Join-Path $projectDir "public\wallpaper.png"))
    $background.BackgroundImageLayout="Stretch";$background.Text="PecoFence feature capture"
    $document=$null;$recorder=$null;$appProcess=$null
    $savedInstance=$env:PECOFENCE_INSTANCE;$savedLocal=$env:LOCALAPPDATA;$savedTest=$env:PECOFENCE_UI_TEST_WINDOWS
    $events=[System.Collections.Generic.List[object]]::new()
    $done=@{};$shown=@{};$recordStartedAt=0.0
    try {
      $background.Show()
      [FeatureNative]::SetWindowPos($background.Handle,[IntPtr](-1),0,0,0,0,0x13)|Out-Null
      [System.Windows.Forms.Application]::DoEvents()
      $env:PECOFENCE_INSTANCE=$(if($Launch){"launch-$scene"}else{"feature-$scene"})
      $env:PECOFENCE_UI_TEST_WINDOWS="1"
      $env:LOCALAPPDATA=Join-Path $caseDir "appdata"
      New-Item -ItemType Directory -Force -Path $env:LOCALAPPDATA|Out-Null
      $args=@("--portable","--no-hide-icons","--dark","--wallpaper",('"' +(Join-Path $projectDir "public\wallpaper.png")+'"'),"--test-script",('"'+$testPath+'"'),"--exit-after","35000")
      $appProcess=Start-Process -FilePath (Join-Path $demoDir "pecofence.exe") -ArgumentList $args -WindowStyle Hidden -PassThru
      $timer=[Diagnostics.Stopwatch]::StartNew()
      while($timer.Elapsed.TotalSeconds -lt $caseLength -and -not $appProcess.HasExited){
        [System.Windows.Forms.Application]::DoEvents()
        foreach($window in [FeatureNative]::Windows([uint32]$appProcess.Id)){
            $title=[FeatureNative]::Title($window)
            if($title -eq "Private inbox excluded from film"){[FeatureNative]::ShowWindow($window,0)|Out-Null;continue}
            if([FeatureNative]::Class($window) -eq "PecoFence.Fence"){
                if(-not $shown[$window.ToInt64()] -and $timer.Elapsed.TotalSeconds -gt 1.7){
                    [FeatureNative]::ShowWindow($window,4)|Out-Null;$shown[$window.ToInt64()]=$true
                }
                if($scene -ne "peek"){
                    $flags=if($scene -eq "groups"){0x53}else{0x13}
                    [FeatureNative]::SetWindowPos($window,[IntPtr](-1),0,0,0,0,$flags)|Out-Null
                }
            }
        }
        At "record" 4 {
            $script:recordStartedAt=$timer.Elapsed.TotalSeconds
            Mouse-Point ($plateX+2400) ($plateY+1270)
            $recordArgs=@("-hide_banner","-loglevel","warning","-y","-f","gdigrab","-framerate","30","-draw_mouse","1","-offset_x",$plateX,"-offset_y",$plateY,"-video_size","2560x1440","-i","desktop","-t",($caseLength-4.4),"-c:v","libx264","-preset","ultrafast","-crf","17","-pix_fmt","yuv420p",('"'+(Join-Path $assetDir "$scene-raw.mp4")+'"'))
            $script:recorder=Start-Process -FilePath (Get-Command ffmpeg.exe).Source -ArgumentList $recordArgs -WindowStyle Hidden -PassThru -RedirectStandardError (Join-Path $caseDir "record.log")
        }
        switch($scene){
          "groups" {
            At "draw-new-area" 5.0 {Drag ($plateX+1250) ($plateY+420) ($plateX+2000) ($plateY+970)}
            At "create-menu" 8.0 {Select-Menu 250}
            At "rename-menu" 9.4 {$w=Find-Fence "新栅栏";$r=Rect-Of $w;Mouse-Click ($r.Left+110) ($r.Top+30) -Right}
            At "choose-rename" 10.0 {Select-Menu 330}
            At "type-name" 10.6 {
                $rename=Find-Fence "rename"
                $edit=[FeatureNative]::EditOf($rename)
                if($edit -eq [IntPtr]::Zero){throw "The rename edit control was not found"}
                [FeatureNative]::SetForegroundWindow($rename)|Out-Null
                [FeatureNative]::SetWindowPos($rename,[IntPtr](-1),0,0,0,0,0x53)|Out-Null
                Start-Sleep -Milliseconds 100
                [System.Windows.Forms.SendKeys]::SendWait("^a")
                [System.Windows.Forms.SendKeys]::SendWait("Work{ENTER}")
            }
            At "drag-file-1" 11.5 {$a=Rect-Of (Find-Fence "Desktop files");$b=Rect-Of (Find-Fence "Work");Drag ($a.Left+96) ($a.Top+145) ($b.Left+185) ($b.Top+160)}
            At "drag-file-2" 13.0 {$a=Rect-Of (Find-Fence "Desktop files");$b=Rect-Of (Find-Fence "Work");Drag ($a.Left+96) ($a.Top+145) ($b.Left+370) ($b.Top+160)}
            At "position-group" 14.7 {$b=Rect-Of (Find-Fence "Work");Drag ($b.Left+360) ($b.Top+28) ($b.Left+460) ($b.Top+155)}
            At "resize-group" 16.5 {$b=Rect-Of (Find-Fence "Work");Drag ($b.Right-3) ($b.Bottom-3) ($b.Right+165) ($b.Bottom+130)}
          }
          "rules" {
            At "new-image" 7.3 {Desktop-Sample "homepage-demo.png" "image"|Out-Null}
            At "new-document" 11.2 {Desktop-Sample "brief-demo.txt"|Out-Null}
            At "rule-proof" 15 {
                $state=Get-Content -LiteralPath $configPath -Raw|ConvertFrom-Json -AsHashtable
                $proof=@()
                foreach($f in $state.layouts[0].fences){
                    if($f.title -in @("Images","Documents")){
                        $proof+=@{group=$f.title;items=@($f.items|ForEach-Object{$state.items[$_.itemId].displayName})}
                    }
                }
                $proof|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $assetDir "rules-proof.json") -Encoding utf8
            }
          }
          "portal" {
            At "enter-brand" 6.2 {$r=Rect-Of (Find-Fence "Project assets");Mouse-Click ($r.Left+96) ($r.Top+145) -Double}
            At "return-parent" 8.8 {$w=Find-Fence "Project assets";[FeatureNative]::PostMessage($w,0x100,[IntPtr]8,[IntPtr]::Zero)|Out-Null}
            At "live-folder-update" 11.2 {$file=Join-Path $folders["Project assets"] "review-notes.txt";if(-not(Test-Path -LiteralPath $file)){Make-Sample $file}}
            At "point-new-file" 14 {$r=Rect-Of (Find-Fence "Project assets");Mouse-Point ($r.Left+710) ($r.Top+140)}
          }
          "tabs" {
            At "tabbed-groups" 6.3 {$r=Rect-Of (Find-Fence "Work");Mouse-Point ($r.Left+70) ($r.Top+30)}
            At "merge-state" 8.0 {
                $windows=@([FeatureNative]::Windows([uint32]$appProcess.Id)|ForEach-Object{@{title=[FeatureNative]::Title($_);class=[FeatureNative]::Class($_);rect=(Rect-Of $_)}})
                $windows|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $caseDir "windows-merged.json")
                Copy-Item -LiteralPath $configPath -Destination (Join-Path $caseDir "config-merged.json")
            }
            At "switch-media" 9.2 {
                Window-Click (Find-Fence "Work") 170 30
            }
            At "switch-work" 11.2 {
                Window-Click (Find-Fence "Work") 70 30
            }
            At "separate-tab" 13.7 {
                $r=Rect-Of (Find-Fence "Work")
                Mouse-Point ($r.Left+170) ($r.Top+30)
            }
            At "align-detached-tab" 17.3 {
                if(-not $Launch){
                    $w=Find-Fence "Art"
                    [FeatureNative]::SetWindowPos($w,[IntPtr](-1),($plateX+1390),($plateY+410),960,538,0x50)|Out-Null
                }
            }
            At "park-pointer" 17 {Mouse-Point ($plateX+2400) ($plateY+1270)}
          }
          "peek" {
            At "working-application" 4.5 {
                $script:document=[System.Windows.Forms.Form]::new()
                $document.FormBorderStyle="None";$document.StartPosition="Manual";$document.ShowInTaskbar=$false
                $document.Bounds=[System.Drawing.Rectangle]::new($plateX+100,$plateY+175,2360,1120)
                $document.BackColor=[Drawing.ColorTranslator]::FromHtml("#152337")
                $document.Text="Website launch — Demo workspace"
                $label=[Windows.Forms.Label]::new()
                $label.Font=[Drawing.Font]::new("Segoe UI",28)
                $label.ForeColor=[Drawing.ColorTranslator]::FromHtml("#cedaea")
                $label.Text="WEBSITE LAUNCH  /  PROJECT BRIEF"
                $label.Location=[Drawing.Point]::new(135,80);$label.Size=[Drawing.Size]::new(2100,100)
                $footer=[Windows.Forms.Label]::new()
                $footer.Font=[Drawing.Font]::new("Segoe UI",21)
                $footer.ForeColor=[Drawing.ColorTranslator]::FromHtml("#adc5db")
                $footer.Text="NEXT UP`r`nReview the visual direction. Collect feedback. Prepare the launch."
                $footer.Location=[Drawing.Point]::new(135,880);$footer.Size=[Drawing.Size]::new(2100,210)
                $document.Controls.Add($label);$document.Controls.Add($footer);$document.Show()
                if($Launch){
                    $document.Controls.Clear()
                    $label.Dispose();$footer.Dispose()
                    $document.BackgroundImage=[Drawing.Image]::FromFile((Join-Path $projectDir "public\launch\workspace-board.png"))
                    $document.BackgroundImageLayout="Stretch"
                }
                [FeatureNative]::SetWindowPos($document.Handle,[IntPtr](-1),0,0,0,0,0x13)|Out-Null
                [FeatureNative]::SetForegroundWindow($document.Handle)|Out-Null
            }
          }
          "hide" {
            At "point-desktop" 5.8 {Mouse-Point ($plateX+1250) ($plateY+1150)}
            At "point-title" 10.2 {$r=Rect-Of (Find-Fence "Work");Mouse-Point ($r.Left+280) ($r.Top+30)}
            At "leave-title" 10.6 {Mouse-Point ($plateX+1250) ($plateY+1150)}
            At "hover-expand" 13.6 {$r=Rect-Of (Find-Fence "Work");Mouse-Point ($r.Left+280) ($r.Top+30)}
          }
        }
        At "poster" ($caseLength-2.0) {Save-Plate (Join-Path $assetDir "$scene-result.png")}
        Start-Sleep -Milliseconds 45
      }
      if($recorder){$recorder.WaitForExit(5000)|Out-Null}
    } finally {
      if($recorder -and -not $recorder.HasExited){$recorder.WaitForExit(8000)|Out-Null}
      if($appProcess -and -not $appProcess.HasExited){$appProcess.WaitForExit(18000)|Out-Null}
      if($document){$document.Close();if($document.BackgroundImage){$document.BackgroundImage.Dispose()};$document.Dispose()}
      $background.Close();$background.BackgroundImage.Dispose();$background.Dispose()
      $env:PECOFENCE_INSTANCE=$savedInstance;$env:LOCALAPPDATA=$savedLocal;$env:PECOFENCE_UI_TEST_WINDOWS=$savedTest
      foreach($file in @($createdDesktopFiles)){
          $absolute=[IO.Path]::GetFullPath($file)
          if([IO.Path]::GetDirectoryName($absolute) -ne [IO.Path]::GetFullPath($desktopDir)){throw "Unexpected sample cleanup path"}
          if(Test-Path -LiteralPath $absolute){Remove-Item -LiteralPath $absolute -Force}
      }
      $createdDesktopFiles.Clear()
      $events|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $assetDir "$scene-events.json") -Encoding utf8
    }
    Write-Output "Completed $scene. Recorded action events: $($events.Count)"
 }
} finally {
    foreach($file in @($createdDesktopFiles)){if(Test-Path -LiteralPath $file){Remove-Item -LiteralPath $file -Force}}
    [FeatureNative]::SetCursorPos($originalCursor.X,$originalCursor.Y)|Out-Null
    [FeatureNative]::SetForegroundWindow($originalForeground)|Out-Null
}
