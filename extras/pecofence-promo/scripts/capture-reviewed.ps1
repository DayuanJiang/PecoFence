param([ValidatePattern('^[a-z0-9-]+$')][string]$Take = "v1")
$ErrorActionPreference = "Stop"
$projectDir=(Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$workspaceDir=(Resolve-Path (Join-Path $projectDir "..\..")).Path
$caseDir=Join-Path $projectDir ".capture\reviewed\$Take"
$assetDir=Join-Path $projectDir "public\reviewed\$Take"
if(Test-Path -LiteralPath $caseDir){throw "Use a new take name; existing captures are preserved."}
$appDir=Join-Path $caseDir "app"
$filesDir=Join-Path $caseDir "desktop-fixture"
$payloadDir=Join-Path $caseDir "documents"
New-Item -ItemType Directory -Path $caseDir,$assetDir,$appDir,$filesDir,$payloadDir,(Join-Path $appDir "config") -Force|Out-Null
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;using System.Text;using System.Collections.Generic;using System.Runtime.InteropServices;
public static class ReviewNative {
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
 [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 public static string Title(IntPtr h){var s=new StringBuilder(512);GetWindowText(h,s,512);return s.ToString();}
 public static string Class(IntPtr h){var s=new StringBuilder(128);GetClassName(h,s,128);return s.ToString();}
 public static List<IntPtr> Windows(uint pid){var a=new List<IntPtr>();EnumWindows((h,l)=>{uint p;GetWindowThreadProcessId(h,out p);if(p==pid)a.Add(h);return true;},IntPtr.Zero);return a;}
}
"@
[ReviewNative]::SetProcessDpiAwarenessContext([IntPtr](-4))|Out-Null
$screen=[Windows.Forms.Screen]::PrimaryScreen.Bounds
$workArea=[Windows.Forms.Screen]::PrimaryScreen.WorkingArea
$plateW=2560;$plateH=1440
if($screen.Width -lt $plateW -or $screen.Height -lt $plateH){throw "Capture requires a 2560x1440 or larger display."}
$plateX=[int](($screen.Width-$plateW)/2);$plateY=[int](($screen.Height-$plateH)/2)
# Reuse only device geometry and preference defaults from an earlier isolated demo.
$reference=Get-Content -LiteralPath (Join-Path $projectDir ".capture\launch\hero\app\config\config.json") -Raw|ConvertFrom-Json -AsHashtable
$fingerprint=$reference.layouts[0].fingerprint
$dpiScale=$fingerprint[0].dpi/96.0
$binary=Join-Path $projectDir ".capture\reviewed-build\target\debug\pecofence.exe"
$viewer=Join-Path $projectDir ".capture\reviewed-build\DemoDocumentViewer.exe"
Copy-Item -LiteralPath $binary -Destination (Join-Path $appDir "pecofence.exe")
Copy-Item -LiteralPath (Join-Path $workspaceDir "target\release\WebView2Loader.dll") -Destination (Join-Path $appDir "WebView2Loader.dll")
$proofPath=Join-Path $caseDir "viewer-opened.json"
$brief=Join-Path $payloadDir "Brief.txt"
[IO.File]::WriteAllText($brief,"The launch starts here.`r`n`r`nBuild a clear homepage.`r`nChoose the final images.`r`nShare the first draft.")
[IO.File]::WriteAllText((Join-Path $filesDir "Notes.txt"),"Website launch notes. Keep the work in one place.")
foreach($name in @("Coast","Dunes")){
    Copy-Item -LiteralPath (Join-Path $projectDir "public\launch\demo-images\$name.png") -Destination (Join-Path $filesDir "$name.png")
}
$wsh=New-Object -ComObject WScript.Shell
$shortcut=$wsh.CreateShortcut((Join-Path $filesDir "Brief.lnk"))
$shortcut.TargetPath=$viewer
$shortcut.Arguments=('"'+$brief+'" "'+$proofPath+'" '+($plateX+740)+' '+($plateY+430)+' 1100 700')
$shortcut.IconLocation="C:\Windows\System32\shell32.dll,1"
$shortcut.Description="Open the staged website launch brief"
$shortcut.Save()
[Runtime.InteropServices.Marshal]::FinalReleaseComObject($shortcut)|Out-Null
[Runtime.InteropServices.Marshal]::FinalReleaseComObject($wsh)|Out-Null

function New-Fence([string]$Title,[int]$X,[int]$Y,[int]$W,[int]$H){
    return @{
        id=[guid]::NewGuid().ToString();title=$Title;kind="virtual";source=@{kind="desktop"}
        geometry=@{monitor=$fingerprint[0].devicePath;x=($plateX+$X)/$dpiScale;y=($plateY+$Y)/$dpiScale;w=$W/$dpiScale;h=$H/$dpiScale;workW=$workArea.Width/$dpiScale;workH=$workArea.Height/$dpiScale;anchor="leftTop"}
        rolledUp=$false;expandedH=$H/$dpiScale
        view=@{iconSize=96;sort="manual";labelLines=2;autoHeight=$false;reverse=$false;layout="icons";spacing="normal"}
        appearance=@{tintRgb=$null;opacity=0.5;backdrop="acrylic";titleRgb=$null;titleSize="large"}
        excludeFromQuickHide=$false;locked=$false;tabHost=$null;activeTab=$null;tabOrder=@()
        portalNavigate=$true;hideTitleIcon=$false;items=@()
    }
}
$unsorted=New-Fence "Unsorted" 200 560 650 700
$work=New-Fence "Work" 960 560 650 420
$art=New-Fence "Art" 1720 560 650 420
$inbox=New-Fence "Demo inbox" (-$plateX) (-$plateY) 250 100
$inbox.kind="inbox";$inbox.rolledUp=$true
$items=@{};$identities=@()
$names=@("Brief.lnk","Notes.txt","Coast.png","Dunes.png")
for($i=0;$i -lt $names.Count;$i++){
    $file=Join-Path $filesDir $names[$i]
    $id=[guid]::NewGuid().ToString()
    $meta=Get-Item -LiteralPath $file
    $item=@{
        id=$id;key=@{path=$file.ToLowerInvariant()};origin="userDesktop"
        displayName=$(if($i -eq 0){"Brief"}else{$names[$i]})
        fileId=$null;mtime=0;isFolder=$false;attrs=32
        iconKey=@{byContent=@{path=$file;mtime=0}}
        orphanedSince=$null;size=$meta.Length;openCount=0
    }
    $items[$id]=$item
    $unsorted.items+=@{itemId=$id;manualIndex=$i;assignedBy="user"}
    $identities+=@{id=$id;name=$names[$i];path=$file;sha256=(Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant();lastWriteUtc=$meta.LastWriteTimeUtc.ToString("o");target=$(if($i -lt 2){"Work"}else{"Art"})}
}
$settings=$reference.settings|ConvertTo-Json -Depth 20|ConvertFrom-Json -AsHashtable
$settings.language="en";$settings.theme="dark";$settings.themeStyle="liquidGlass"
$settings.desktopPath=$filesDir;$settings.hideRealIcons=$false;$settings.showRealIconsWhenFencesHidden=$false
$settings.peek.enabled=$false;$settings.peek.dim=$true;$settings.snapping.sizeToCells=$false
$reg=Get-ItemProperty -LiteralPath "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name PecoFence -ErrorAction SilentlyContinue
$settings.autostart=($null -ne $reg)
$config=@{schemaVersion=1;settings=$settings;items=$items;layouts=@(@{fingerprint=$fingerprint;fences=@($unsorted,$work,$art,$inbox)});rules=@{defaultTarget="inbox";keepUpdated=$false;list=@()};undoLog=@();snapshots=@()}
$configPath=Join-Path $appDir "config\config.json"
$config|ConvertTo-Json -Depth 35|Set-Content -LiteralPath $configPath -Encoding utf8
$identities|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $caseDir "fixture-before.json") -Encoding utf8

function Save-Plate([string]$Path){
    $bmp=[Drawing.Bitmap]::new($plateW,$plateH);$g=[Drawing.Graphics]::FromImage($bmp)
    try{$g.CopyFromScreen($plateX,$plateY,0,0,$bmp.Size);$bmp.Save($Path,[Drawing.Imaging.ImageFormat]::Png)}
    finally{$g.Dispose();$bmp.Dispose()}
}
function Read-DemoState{
    return Get-Content -LiteralPath $configPath -Raw|ConvertFrom-Json -AsHashtable
}
function Verify-Membership([hashtable]$State){
    $result=@()
    if($State.items.Count -ne 4){throw "Fixture isolation failed: expected exactly four enumerated items."}
    foreach($expected in $identities){
        $entry=$State.items[$expected.id]
        if(-not $entry){throw "A staged identity disappeared."}
        if($entry.key.path -ne $expected.path.ToLowerInvariant()){throw "A canonical file path changed."}
        if((Get-FileHash -LiteralPath $expected.path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expected.sha256){throw "Grouping changed file contents."}
        if((Get-Item -LiteralPath $expected.path).LastWriteTimeUtc.ToString("o") -ne $expected.lastWriteUtc){throw "Grouping changed a file timestamp."}
        $group=@($State.layouts[0].fences|Where-Object{$_.items.itemId -contains $expected.id})
        if($group.Count -ne 1 -or $group[0].title -ne $expected.target){throw "Wrong destination for $($expected.name)"}
        $result+=@{id=$expected.id;name=$expected.name;group=$group[0].title;sha256=$expected.sha256;pathUnchanged=$true;mtimeUnchanged=$true}
    }
    return $result
}
$priorForeground=[ReviewNative]::GetForegroundWindow()
$saved=@{instance=$env:PECOFENCE_INSTANCE;test=$env:PECOFENCE_UI_TEST_WINDOWS;local=$env:LOCALAPPDATA;fixture=$env:PECOFENCE_DEMO_DESKTOP}
$sceneReports=@()
try {
 foreach($scene in @("flow","peek")){
    $sceneDir=Join-Path $caseDir $scene
    New-Item -ItemType Directory -Path $sceneDir -Force|Out-Null
    $commands=if($scene -eq "flow"){
        @("sleep 1000","pin-test-windows","sleep 5500",
          "move Unsorted Work","sleep 800","move Unsorted Work","sleep 800",
          "move Unsorted Art","sleep 800","move Unsorted Art","sleep 600",
          "delete Unsorted","sleep 2800","merge Art Work","sleep 1600",
          "activate Work","sleep 3500","exit")
    }else{@("sleep 6500","peek","sleep 14500","exit")}
    $scriptFile=Join-Path $sceneDir "commands.txt"
    $commands|Set-Content -LiteralPath $scriptFile -Encoding utf8
    $background=[Windows.Forms.Form]::new()
    $background.FormBorderStyle="None";$background.StartPosition="Manual";$background.ShowInTaskbar=$false
    $background.Bounds=$screen
    $background.BackgroundImage=[Drawing.Image]::FromFile((Join-Path $projectDir "public\wallpaper.png"))
    $background.BackgroundImageLayout="Stretch"
    $background.Text="PecoFence isolated capture"
    $board=$null;$app=$null;$recorder=$null
    $done=@{};$shown=@{};$events=[Collections.Generic.List[object]]::new()
    $caseLength=if($scene -eq "flow"){20.0}else{17.0}
    try {
        $background.Show()
        [ReviewNative]::SetWindowPos($background.Handle,[IntPtr](-1),0,0,0,0,0x13)|Out-Null
        [Windows.Forms.Application]::DoEvents()
        $env:PECOFENCE_INSTANCE="review-$Take-$scene"
        $env:PECOFENCE_UI_TEST_WINDOWS="1"
        $env:PECOFENCE_DEMO_DESKTOP=$filesDir
        $env:LOCALAPPDATA=Join-Path $sceneDir "appdata"
        New-Item -ItemType Directory -Path $env:LOCALAPPDATA -Force|Out-Null
        $arguments=@("--portable","--no-hide-icons","--dark","--wallpaper",('"'+(Join-Path $projectDir "public\wallpaper.png")+'"'),"--test-script",('"'+$scriptFile+'"'),"--exit-after","30000")
        $app=Start-Process -FilePath (Join-Path $appDir "pecofence.exe") -ArgumentList $arguments -WindowStyle Hidden -PassThru
        $timer=[Diagnostics.Stopwatch]::StartNew()
        Write-Output "Capturing isolated $scene / $Take"
        while($timer.Elapsed.TotalSeconds -lt $caseLength -and -not $app.HasExited){
            [Windows.Forms.Application]::DoEvents()
            foreach($window in [ReviewNative]::Windows([uint32]$app.Id)){
                $title=[ReviewNative]::Title($window)
                if($title -eq "Demo inbox"){[ReviewNative]::ShowWindow($window,0)|Out-Null;continue}
                if([ReviewNative]::Class($window) -eq "PecoFence.Fence"){
                    if(-not $shown[$window.ToInt64()] -and $timer.Elapsed.TotalSeconds -gt 1.7){
                        [ReviewNative]::ShowWindow($window,4)|Out-Null;$shown[$window.ToInt64()]=$true
                    }
                    if($scene -eq "flow"){[ReviewNative]::SetWindowPos($window,[IntPtr](-1),0,0,0,0,0x13)|Out-Null}
                }
            }
            if($scene -eq "peek" -and -not $done.board -and $timer.Elapsed.TotalSeconds -ge 2){
                $done.board=$true
                $board=[Windows.Forms.Form]::new()
                $board.FormBorderStyle="None";$board.StartPosition="Manual";$board.ShowInTaskbar=$false
                $board.Bounds=[Drawing.Rectangle]::new($plateX+100,$plateY+175,2360,1120)
                $board.BackgroundImage=[Drawing.Image]::FromFile((Join-Path $projectDir "public\launch\workspace-board.png"))
                $board.BackgroundImageLayout="Stretch";$board.Text="Website launch — Demo workspace"
                $board.Show()
                [ReviewNative]::SetWindowPos($background.Handle,[IntPtr](-2),0,0,0,0,0x13)|Out-Null
                [ReviewNative]::SetWindowPos($board.Handle,[IntPtr]::Zero,0,0,0,0,0x13)|Out-Null
                [ReviewNative]::SetForegroundWindow($board.Handle)|Out-Null
            }
            if(-not $done.record -and $timer.Elapsed.TotalSeconds -ge 4){
                $done.record=$true;$recordAt=$timer.Elapsed.TotalSeconds
                $recordArgs=@("-hide_banner","-loglevel","warning","-n","-f","gdigrab","-framerate","30","-draw_mouse","0","-offset_x",$plateX,"-offset_y",$plateY,"-video_size","2560x1440","-i","desktop","-t",($caseLength-4.5),"-an","-c:v","libx264","-preset","ultrafast","-crf","17","-pix_fmt","yuv420p",('"'+(Join-Path $assetDir "$scene-raw.mp4")+'"'))
                $recorder=Start-Process -FilePath (Get-Command ffmpeg.exe).Source -ArgumentList $recordArgs -WindowStyle Hidden -PassThru -RedirectStandardError (Join-Path $sceneDir "encoder.log")
                $events.Add(@{name="record";elapsed=$recordAt})
            }
            if($scene -eq "flow" -and -not $done.before -and $timer.Elapsed.TotalSeconds -ge 5.6){
                $done.before=$true
                $state=Read-DemoState
                if($state.items.Count -ne 4){throw "Non-fixture items entered the capture."}
                foreach($expected in $identities){
                    $observed=$state.items[$expected.id]
                    if(-not $observed -or $null -ne $observed.orphanedSince -or $observed.key.path -ne $expected.path.ToLowerInvariant()){
                        throw "A staged item identity, normalized path, or live status changed during reconciliation."
                    }
                }
                $source=@($state.layouts[0].fences|Where-Object{$_.title -eq "Unsorted"})[0]
                if(($source.items.itemId -join ",") -ne ($identities.id -join ",")){throw "Initial membership order differs from the staged transfer order."}
                Save-Plate (Join-Path $assetDir "before.png")
            }
            if($scene -eq "flow" -and -not $done.grouped -and $timer.Elapsed.TotalSeconds -ge 11.4){
                $done.grouped=$true;$state=Read-DemoState
                $membership=Verify-Membership $state
                $membership|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $assetDir "grouping-proof.json") -Encoding utf8
                Save-Plate (Join-Path $assetDir "grouped.png")
            }
            if($scene -eq "peek" -and -not $done.open -and $timer.Elapsed.TotalSeconds -ge 9){
                $done.open=$true
                $window=@([ReviewNative]::Windows([uint32]$app.Id)|Where-Object{[ReviewNative]::Title($_) -eq "Work"})[0]
                if(-not $window){throw "Combined Work window is missing."}
                [ReviewNative]::SetForegroundWindow($window)|Out-Null
                $point=[IntPtr]((180 -shl 16) -bor 140)
                [ReviewNative]::PostMessage($window,0x200,[IntPtr]::Zero,$point)|Out-Null
                [ReviewNative]::PostMessage($window,0x201,[IntPtr]1,$point)|Out-Null
                [ReviewNative]::PostMessage($window,0x202,[IntPtr]::Zero,$point)|Out-Null
                [ReviewNative]::PostMessage($window,0x203,[IntPtr]1,$point)|Out-Null
                [ReviewNative]::PostMessage($window,0x202,[IntPtr]::Zero,$point)|Out-Null
                $events.Add(@{name="native-double-click-Brief";elapsed=$timer.Elapsed.TotalSeconds;recordRelative=$timer.Elapsed.TotalSeconds-$recordAt})
            }
            if(-not $done.poster -and $timer.Elapsed.TotalSeconds -ge ($caseLength-2)){
                $done.poster=$true;Save-Plate (Join-Path $assetDir "$scene-result.png")
            }
            Start-Sleep -Milliseconds 35
        }
        if(-not $done.record){throw "Recording never started."}
        if($recorder){$recorder.WaitForExit(6000)|Out-Null;if(-not $recorder.HasExited -or $recorder.ExitCode -ne 0){throw "Encoder failed or did not finish."}}
        if($app -and -not $app.HasExited){$app.WaitForExit(15000)|Out-Null}
        if(-not $app.HasExited -or $app.ExitCode -ne 0){throw "Demo process did not exit successfully."}
        $finalState=Read-DemoState
        Verify-Membership $finalState|Out-Null
        if($scene -eq "flow"){
            $groupHost=@($finalState.layouts[0].fences|Where-Object{$_.title -eq "Work"})[0]
            $child=@($finalState.layouts[0].fences|Where-Object{$_.title -eq "Art"})[0]
            $tabMembers=@($finalState.layouts[0].fences|Where-Object{$_.id -eq $groupHost.id -or $_.tabHost -eq $groupHost.id})
            # Native persistence omits redundant default tab order and active-self fields.
            if($child.tabHost -ne $groupHost.id -or $tabMembers.Count -ne 2 -or ($groupHost.activeTab -and $groupHost.activeTab -ne $groupHost.id)){
                throw "The same Work/Art groups were not merged with Work active."
            }
            Copy-Item -LiteralPath $configPath -Destination (Join-Path $caseDir "after-flow.json")
        }else{
            if(-not(Test-Path -LiteralPath $proofPath)){throw "The native open action did not show the document viewer."}
            $opened=Get-Content -LiteralPath $proofPath -Raw|ConvertFrom-Json -AsHashtable
            $briefHash=(Get-FileHash -LiteralPath $brief -Algorithm SHA256).Hash.ToLowerInvariant()
            if($opened.documentSha256 -ne $briefHash -or $finalState.items[$identities[0].id].openCount -lt 1){throw "Missing file-backed native open evidence."}
            @{title=$opened.title;shownAt=$opened.shownAt;documentSha256=$briefHash;nativeOpenCount=$finalState.items[$identities[0].id].openCount}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $assetDir "open-proof.json") -Encoding utf8
        }
        $events|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $assetDir "$scene-events.json") -Encoding utf8
        $sceneReports+=@{scene=$scene;recordedAt=(Get-Item -LiteralPath (Join-Path $assetDir "$scene-raw.mp4")).LastWriteTimeUtc.ToString("o");encoderExit=$recorder.ExitCode;applicationExit=$app.ExitCode}
        Write-Output "Verified $scene capture and native postconditions."
    }finally{
        if($app -and -not $app.HasExited){$app.WaitForExit(15000)|Out-Null}
        if($recorder -and -not $recorder.HasExited){$recorder.WaitForExit(6000)|Out-Null}
        if($board){$board.Close();$board.BackgroundImage.Dispose();$board.Dispose()}
        $background.Close();$background.BackgroundImage.Dispose();$background.Dispose()
    }
 }
 $sourceManifest=Get-Content -LiteralPath (Join-Path $projectDir ".capture\reviewed-build\source-manifest.json") -Raw|ConvertFrom-Json -AsHashtable
 @{
    take=$Take;executableSha256=(Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash.ToLowerInvariant()
    sourceFingerprint=$sourceManifest.originalSourceFingerprint
    fixtureOverride="Debug-only known desktop folder resolver override; native product handlers and rendering unchanged."
    language="en";theme="liquidGlass";sceneReports=$sceneReports
    groupKind="virtual";sampleCount=4;uiTestWindowStyles=$true;physicalInputVerified=$false;globalHotkeyVerified=$false
    stagedApplications=@("Launchpad project-board window","File-backed demo document viewer")
 }|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $assetDir "capture-manifest.json") -Encoding utf8
}finally{
    $env:PECOFENCE_INSTANCE=$saved.instance;$env:PECOFENCE_UI_TEST_WINDOWS=$saved.test
    $env:LOCALAPPDATA=$saved.local;$env:PECOFENCE_DEMO_DESKTOP=$saved.fixture
    [ReviewNative]::SetForegroundWindow($priorForeground)|Out-Null
}
