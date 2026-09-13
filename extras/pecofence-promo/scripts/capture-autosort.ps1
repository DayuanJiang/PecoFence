param([ValidatePattern('^auto-v[0-9]+$')][string]$Take="auto-v1")
$ErrorActionPreference="Stop"
$projectDir=(Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$workspaceDir=(Resolve-Path (Join-Path $projectDir "..\..")).Path
$caseDir=Join-Path $projectDir ".capture\reviewed\$Take"
$assetDir=Join-Path $projectDir "public\autosort\$Take"
if((Test-Path -LiteralPath $caseDir) -or (Test-Path -LiteralPath $assetDir)){throw "Choose a new immutable take name."}
$appDir=Join-Path $caseDir "app"
$filesDir=Join-Path $caseDir "desktop-fixture"
$payloadDir=Join-Path $caseDir "payload"
New-Item -ItemType Directory -Path $caseDir,$assetDir,$appDir,$filesDir,$payloadDir,(Join-Path $appDir "config") -Force|Out-Null
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;using System.Text;using System.Collections.Generic;using System.Runtime.InteropServices;
public static class AutoNative {
 public delegate bool EnumProc(IntPtr h,IntPtr l);
 [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr c);
 [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc p,IntPtr l);
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h,out uint p);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h,StringBuilder s,int n);
 [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h,IntPtr a,int x,int y,int w,int z,uint f);
 [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h,int n);
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
 public static string Title(IntPtr h){var s=new StringBuilder(512);GetWindowText(h,s,512);return s.ToString();}
 public static List<IntPtr> Windows(uint pid){var a=new List<IntPtr>();EnumWindows((h,l)=>{uint p;GetWindowThreadProcessId(h,out p);if(p==pid)a.Add(h);return true;},IntPtr.Zero);return a;}
}
"@
[AutoNative]::SetProcessDpiAwarenessContext([IntPtr](-4))|Out-Null
$priorForeground=[AutoNative]::GetForegroundWindow()
[uint32]$foregroundOwner=0
[AutoNative]::GetWindowThreadProcessId($priorForeground,[ref]$foregroundOwner)|Out-Null
if($foregroundOwner -and ([Diagnostics.Process]::GetProcessById($foregroundOwner).ProcessName -eq "LockApp")){throw "The desktop is locked."}
$screen=[Windows.Forms.Screen]::PrimaryScreen.Bounds
$workArea=[Windows.Forms.Screen]::PrimaryScreen.WorkingArea
if($screen.Width -lt 2560 -or $screen.Height -lt 1440){throw "Capture requires a 2560x1440 or larger display."}
$plateX=[int](($screen.Width-2560)/2);$plateY=[int](($screen.Height-1440)/2)
$reference=Get-Content -LiteralPath (Join-Path $projectDir ".capture\launch\hero\app\config\config.json") -Raw|ConvertFrom-Json -AsHashtable
$fingerprint=$reference.layouts[0].fingerprint
$dpiScale=$fingerprint[0].dpi/96.0
$binary=Join-Path $projectDir ".capture\reviewed-build\target-auto\debug\pecofence.exe"
Copy-Item -LiteralPath $binary -Destination (Join-Path $appDir "pecofence.exe")
$loader=Join-Path $workspaceDir "third_party\webview2\WebView2Loader.x64.dll"
Copy-Item -LiteralPath $loader -Destination (Join-Path $appDir "WebView2Loader.dll")

[IO.File]::WriteAllText((Join-Path $payloadDir "Notes.txt"),"Website launch notes.")
Copy-Item -LiteralPath (Join-Path $projectDir "public\launch\demo-images\Dunes.png") -Destination (Join-Path $filesDir "Dunes.png")
$briefText=Join-Path $payloadDir "Brief.txt"
[IO.File]::WriteAllText($briefText,"PecoFence demonstration brief.")
$wsh=New-Object -ComObject WScript.Shell
$shortcut=$wsh.CreateShortcut((Join-Path $payloadDir "Brief.lnk"))
$shortcut.TargetPath=$briefText
$shortcut.IconLocation="C:\Windows\System32\shell32.dll,1"
$shortcut.Description="Project brief"
$shortcut.Save()
[Runtime.InteropServices.Marshal]::FinalReleaseComObject($shortcut)|Out-Null
[Runtime.InteropServices.Marshal]::FinalReleaseComObject($wsh)|Out-Null
Copy-Item -LiteralPath (Join-Path $payloadDir "Brief.lnk") -Destination (Join-Path $filesDir "Brief.lnk")

function New-Fence([string]$Title,[int]$X,[int]$Y,[int]$W,[int]$H){
 return @{
  id=[guid]::NewGuid().ToString();title=$Title;kind="virtual";source=@{kind="desktop"}
  geometry=@{monitor=$fingerprint[0].devicePath;x=($plateX+$X)/$dpiScale;y=($plateY+$Y)/$dpiScale;w=$W/$dpiScale;h=$H/$dpiScale;workW=$workArea.Width/$dpiScale;workH=$workArea.Height/$dpiScale;anchor="leftTop"}
  rolledUp=$false;expandedH=$H/$dpiScale
  view=@{iconSize=96;sort="name";labelLines=2;autoHeight=$false;reverse=$false;layout="icons";spacing="normal"}
  appearance=@{tintRgb=$null;opacity=0.5;backdrop="acrylic";titleRgb=$null;titleSize="large"}
  excludeFromQuickHide=$false;locked=$false;tabHost=$null;activeTab=$null;tabOrder=@()
  portalNavigate=$true;hideTitleIcon=$false;items=@()
 }
}
$work=New-Fence "Work" 960 560 650 420
$art=New-Fence "Art" 1720 560 650 420
$inbox=New-Fence "Demo inbox" (-$plateX) (-$plateY) 250 100
$inbox.kind="inbox"
$workRule=[guid]::NewGuid().ToString();$artRule=[guid]::NewGuid().ToString()
$rules=@(
 @{id=$workRule;name="Text and shortcuts to Work";enabled=$true;target=@{fence=$work.id};allOf=@(@{cond="ext";value=@("txt","lnk")});priorityClass="type"},
 @{id=$artRule;name="PNG files to Art";enabled=$true;target=@{fence=$art.id};allOf=@(@{cond="ext";value=@("png")});priorityClass="type"}
)
$settings=$reference.settings|ConvertTo-Json -Depth 20|ConvertFrom-Json -AsHashtable
$settings.language="en";$settings.theme="dark";$settings.themeStyle="liquidGlass"
$settings.desktopPath=$filesDir;$settings.hideRealIcons=$false;$settings.showRealIconsWhenFencesHidden=$false
$settings.peek.enabled=$false;$settings.snapping.sizeToCells=$false;$settings.autostart=$false
$config=@{schemaVersion=1;settings=$settings;items=@{};layouts=@(@{fingerprint=$fingerprint;fences=@($work,$art,$inbox)});rules=@{defaultTarget="inbox";keepUpdated=$true;list=$rules};undoLog=@();snapshots=@()}
$configPath=Join-Path $appDir "config\config.json"
$config|ConvertTo-Json -Depth 35|Set-Content -LiteralPath $configPath -Encoding utf8
$scriptFile=Join-Path $caseDir "commands.txt"
@("sleep 900","pin-test-windows","sleep 14000","dump before-merge","merge Art Work","sleep 1700","activate Work","sleep 11000","exit")|Set-Content -LiteralPath $scriptFile -Encoding utf8

$expected=@{}
function Remember-File([string]$Name,[string]$Target,[bool]$New){
 $file=Join-Path $filesDir $Name
 $meta=Get-Item -LiteralPath $file
 $expected[$Name]=@{name=$Name;path=$file;target=$Target;new=$New;sha256=(Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant();mtime=$meta.LastWriteTimeUtc.ToString("o")}
}
Remember-File "Brief.lnk" "Work" $false
Remember-File "Dunes.png" "Art" $false
function Read-State{
 for($attempt=0;$attempt -lt 6;$attempt++){
  try{return (Get-Content -LiteralPath $configPath -Raw|ConvertFrom-Json -AsHashtable)}
  catch{if($attempt -eq 5){throw};Start-Sleep -Milliseconds 40}
 }
}
function Assert-Routing([hashtable]$State,[int]$Count){
 if($State.items.Count -ne $Count -or -not $State.rules.keepUpdated){throw "Unexpected catalog size or automatic rules disabled."}
 $proof=@()
 foreach($entry in $expected.Values){
  $matches=@($State.items.Values|Where-Object{$_.key.path -eq $entry.path.ToLowerInvariant()})
  if($matches.Count -ne 1){throw "Missing unique native item: $($entry.name)"}
  $item=$matches[0]
  $groups=@($State.layouts[0].fences|Where-Object{$_.items.itemId -contains $item.id})
  if($groups.Count -ne 1 -or $groups[0].title -ne $entry.target){throw "Wrong automatic destination: $($entry.name)"}
  $member=@($groups[0].items|Where-Object{$_.itemId -eq $item.id})[0]
  $wantedRule=if($entry.target -eq "Work"){$workRule}else{$artRule}
  if($member.assignedBy.rule -ne $wantedRule){throw "Native membership is not assigned by the expected rule."}
  if((Get-FileHash -LiteralPath $entry.path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.sha256){throw "File contents changed."}
  if((Get-Item -LiteralPath $entry.path).LastWriteTimeUtc.ToString("o") -ne $entry.mtime){throw "File modification time changed."}
  $proof+=@{name=$entry.name;id=$item.id;group=$entry.target;newlyCreated=$entry.new;ruleId=$wantedRule;assignedBy=$member.assignedBy;pathUnchanged=$true;contentUnchanged=$true;mtimeUnchanged=$true;sha256=$entry.sha256}
 }
 return $proof
}
function Save-Plate([string]$Name){
 $bmp=[Drawing.Bitmap]::new(2560,1440);$g=[Drawing.Graphics]::FromImage($bmp)
 try{$g.CopyFromScreen($plateX,$plateY,0,0,$bmp.Size);$bmp.Save((Join-Path $assetDir $Name),[Drawing.Imaging.ImageFormat]::Png)}
 finally{$g.Dispose();$bmp.Dispose()}
}
$background=[Windows.Forms.Form]::new()
$background.FormBorderStyle="None";$background.StartPosition="Manual";$background.ShowInTaskbar=$false
$background.Bounds=$screen
$background.Text="PecoFence isolated automatic sorting capture"
$background.BackgroundImage=[Drawing.Image]::FromFile((Join-Path $projectDir "public\wallpaper.png"))
$background.BackgroundImageLayout="Stretch"
$savedEnv=@{instance=$env:PECOFENCE_INSTANCE;test=$env:PECOFENCE_UI_TEST_WINDOWS;fixture=$env:PECOFENCE_DEMO_DESKTOP;local=$env:LOCALAPPDATA;log=$env:RUST_LOG}
$savedPointer=[Windows.Forms.Cursor]::Position
$events=[Collections.Generic.List[object]]::new()
$done=@{};$shown=@{};$app=$null;$recorder=$null;$recordAt=0.0
try {
 $env:PECOFENCE_INSTANCE="video-$Take"
 $env:PECOFENCE_UI_TEST_WINDOWS="1"
 $env:PECOFENCE_DEMO_DESKTOP=$filesDir
 $env:LOCALAPPDATA=Join-Path $caseDir "appdata"
 $env:RUST_LOG="pecofence=debug"
 New-Item -ItemType Directory -Path $env:LOCALAPPDATA -Force|Out-Null
 $background.Show()
 [AutoNative]::SetWindowPos($background.Handle,[IntPtr](-1),0,0,0,0,0x13)|Out-Null
 [Windows.Forms.Cursor]::Position=[Drawing.Point]::new($screen.Left+20,$screen.Top+20)
 [Windows.Forms.Application]::DoEvents()
 $arguments=@("--portable","--no-hide-icons","--dark","--wallpaper",('"'+(Join-Path $projectDir "public\wallpaper.png")+'"'),"--test-script",('"'+$scriptFile+'"'),"--exit-after","33000")
 $app=Start-Process -FilePath (Join-Path $appDir "pecofence.exe") -ArgumentList $arguments -WindowStyle Hidden -PassThru
 $timer=[Diagnostics.Stopwatch]::StartNew()
 Write-Output "Recording isolated automatic file routing."
 while($timer.Elapsed.TotalSeconds -lt 29 -and -not $app.HasExited){
  [Windows.Forms.Application]::DoEvents()
  foreach($window in [AutoNative]::Windows([uint32]$app.Id)){
   $title=[AutoNative]::Title($window)
   if($title -eq "Demo inbox"){[AutoNative]::ShowWindow($window,0)|Out-Null;continue}
   if($title -in @("Work","Art")){
    if(-not $shown[$window.ToInt64()] -and $timer.Elapsed.TotalSeconds -gt 1.7){
     [AutoNative]::ShowWindow($window,4)|Out-Null;$shown[$window.ToInt64()]=$true
    }
    [AutoNative]::SetWindowPos($window,[IntPtr](-1),0,0,0,0,0x13)|Out-Null
   }
  }
  if(-not $done.record -and $timer.Elapsed.TotalSeconds -ge 4){
   if($shown.Count -lt 2){throw "Native Work/Art windows did not appear."}
   $done.record=$true;$recordAt=$timer.Elapsed.TotalSeconds
   $recordArgs=@("-hide_banner","-loglevel","warning","-n","-f","gdigrab","-framerate","30","-draw_mouse","0","-offset_x",$plateX,"-offset_y",$plateY,"-video_size","2560x1440","-i","desktop","-t","22","-an","-c:v","libx264","-preset","ultrafast","-crf","17","-pix_fmt","yuv420p",('"'+(Join-Path $assetDir "raw.mp4")+'"'))
   $recorder=Start-Process -FilePath (Get-Command ffmpeg.exe).Source -ArgumentList $recordArgs -WindowStyle Hidden -PassThru -RedirectStandardError (Join-Path $caseDir "encoder.log")
   $events.Add(@{event="record";time=$recordAt})
  }
  if(-not $done.before -and $timer.Elapsed.TotalSeconds -ge 5.7){
   $done.before=$true;$state=Read-State
   $baseline=Assert-Routing $state 2
   foreach($name in @("Coast.png","Notes.txt")){
    $newPath=Join-Path $filesDir $name
    $known=@($state.items.Values|Where-Object{$_.key.path -eq $newPath.ToLowerInvariant()})
    if((Test-Path -LiteralPath $newPath) -or $known.Count){throw "New file was already present on disk or in the catalog."}
   }
   @{keepUpdated=$state.rules.keepUpdated;catalogCount=2;newFilesAbsentFromDisk=$true;newFilesAbsentFromCatalog=$true;existingItems=$baseline}|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $assetDir "before-proof.json") -Encoding utf8
   Save-Plate "before.png"
  }
  if(-not $done.image -and $timer.Elapsed.TotalSeconds -ge 7.2){
   if(-not $done.before){throw "Missing before-creation evidence."}
   $done.image=$true
   Copy-Item -LiteralPath (Join-Path $projectDir "public\launch\demo-images\Coast.png") -Destination (Join-Path $filesDir "Coast.png")
   Remember-File "Coast.png" "Art" $true
   $events.Add(@{event="created-image";name="Coast.png";elapsed=$timer.Elapsed.TotalSeconds;recordRelative=$timer.Elapsed.TotalSeconds-$recordAt;createdUtc=[DateTime]::UtcNow.ToString("o")})
  }
  if(-not $done.imageProof -and $timer.Elapsed.TotalSeconds -ge 8.8){
   $done.imageProof=$true
   Assert-Routing (Read-State) 3|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $assetDir "image-routing-proof.json") -Encoding utf8
   Save-Plate "image-routed.png"
  }
  if(-not $done.document -and $timer.Elapsed.TotalSeconds -ge 9.5){
   $done.document=$true
   Copy-Item -LiteralPath (Join-Path $payloadDir "Notes.txt") -Destination (Join-Path $filesDir "Notes.txt")
   Remember-File "Notes.txt" "Work" $true
   $events.Add(@{event="created-document";name="Notes.txt";elapsed=$timer.Elapsed.TotalSeconds;recordRelative=$timer.Elapsed.TotalSeconds-$recordAt;createdUtc=[DateTime]::UtcNow.ToString("o")})
  }
  if(-not $done.automatic -and $timer.Elapsed.TotalSeconds -ge 12.2){
   $done.automatic=$true
   Assert-Routing (Read-State) 4|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $assetDir "automatic-proof.json") -Encoding utf8
   Save-Plate "automatically-grouped.png"
  }
  if(-not $done.tabs -and $timer.Elapsed.TotalSeconds -ge 23){
   $done.tabs=$true;Save-Plate "tabs-result.png"
  }
  Start-Sleep -Milliseconds 30
 }
 if(-not $done.automatic -or -not $done.tabs){throw "The routing or tab demonstration did not complete."}
 $recorder.WaitForExit(6000)|Out-Null
 if(-not $recorder.HasExited -or $recorder.ExitCode -ne 0){throw "The recording did not finish successfully."}
 if(-not $app.HasExited){$app.WaitForExit(5000)|Out-Null}
 if(-not $app.HasExited -or $app.ExitCode -ne 0){throw "The isolated app did not exit successfully."}
 $finalState=Read-State
 $finalProof=Assert-Routing $finalState 4
 $hostFence=@($finalState.layouts[0].fences|Where-Object{$_.title -eq "Work"})[0]
 $childFence=@($finalState.layouts[0].fences|Where-Object{$_.title -eq "Art"})[0]
 if($childFence.tabHost -ne $hostFence.id -or ($hostFence.activeTab -and $hostFence.activeTab -ne $hostFence.id)){throw "Final tab identity or selection is incorrect."}
 $logFiles=@(Get-ChildItem -LiteralPath (Join-Path $caseDir "appdata") -Recurse -File -Filter "*.log")
 $logs=($logFiles|ForEach-Object{Get-Content -LiteralPath $_.FullName -Raw}) -join "`n"
 $resyncs=[regex]::Matches($logs,"desktop resynced").Count
 if($resyncs -lt 2 -or $logs -notmatch "fs change"){throw "Native filesystem-watcher evidence is missing."}
 $controlLines=Get-Content -LiteralPath $scriptFile
 if(@($controlLines|Where-Object{$_ -notmatch '^(sleep [0-9]+|pin-test-windows|dump before-merge|merge Art Work|activate Work|exit)$'}).Count){throw "Unexpected control command in recording."}
 if($logs -match '>> (move |message |drop-desktop |transfer |new-text |new-folder )'){throw "Manual routing or creation command found in native control log."}
 $sourceManifest=Get-Content -LiteralPath (Join-Path $projectDir ".capture\reviewed-build\source-manifest-v3.json") -Raw|ConvertFrom-Json -AsHashtable
 $events|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $assetDir "events.json") -Encoding utf8
 $finalProof|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $assetDir "final-proof.json") -Encoding utf8
 @{
  take=$Take;product="PecoFence";sourceVersion="v3";sourceFingerprint=$sourceManifest.originalSourceFingerprint
  executableSha256=(Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash.ToLowerInvariant()
  rulesKeepUpdated=$true;newFiles=@("Coast.png","Notes.txt");sampleCount=4;nativeFsResyncs=$resyncs
  noManualRoutingCommands=$true;controlCommands=$controlLines
  language="en";themeStyle="liquidGlass";groupKind="virtual";uiTestWindowStyles=$true
  fixtureOverride="Only debug known-Desktop resolution points to the isolated workspace fixture."
  encoderExit=$recorder.ExitCode;applicationExit=$app.ExitCode
  buildCommand="cargo build --locked --offline --manifest-path extras/pecofence-promo/.capture/reviewed-build/source-v3/Cargo.toml --target-dir extras/pecofence-promo/.capture/reviewed-build/target-auto -p pecofence"
  recordedAt=[DateTime]::UtcNow.ToString("o")
 }|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $assetDir "capture-manifest.json") -Encoding utf8
 Write-Output "Verified new-file watcher routing, exact rule IDs, unchanged files, and native tab continuity."
}finally{
 if($recorder -and -not $recorder.HasExited){$recorder.Kill();$recorder.WaitForExit(3000)|Out-Null}
 if($app -and -not $app.HasExited){$app.Kill();$app.WaitForExit(3000)|Out-Null}
 $background.Close();$background.BackgroundImage.Dispose();$background.Dispose()
 $env:PECOFENCE_INSTANCE=$savedEnv.instance;$env:PECOFENCE_UI_TEST_WINDOWS=$savedEnv.test
 $env:PECOFENCE_DEMO_DESKTOP=$savedEnv.fixture;$env:LOCALAPPDATA=$savedEnv.local;$env:RUST_LOG=$savedEnv.log
 [Windows.Forms.Cursor]::Position=$savedPointer
 [AutoNative]::SetForegroundWindow($priorForeground)|Out-Null
}
