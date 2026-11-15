# Feedback #153: Windows' 「发送到」 (Send to) in the item menu of a fence and of a folder fence,
# against Explorer's own classic menu ("Show more options") for the same files. The report came
# from Windows 11 26300 (Japanese, 125 %): the entry was missing for a file in a fence.
#   run-hyperv.sh --vm PF-Win11-3 tests/issues-vm-sendto.ps1        (about 2 minutes)

$Docs = "$PfRun\docs153"
$DeskName = 'st153 桌面 note.txt'
$DocName = 'st153 文档.txt'

function ExplorerAt($dir) {
  foreach ($k in 1..40) {
    foreach ($x in @((New-Object -ComObject Shell.Application).Windows())) { $f = try { $x.Document.Folder.Self.Path } catch { $null }; if ($f -eq $dir) { return $x } }
    if ($k -eq 1) { Start-Process explorer.exe "`"$dir`"" }
    Start-Sleep -Milliseconds 250
  }
  return $null
}
# Windows 11's menu entry Show more options, in any window of Explorer.
function ShowMorePoint($sec = 4) {
  $any = [Windows.Automation.Condition]::TrueCondition
  $byName = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::NameProperty, 'Show more options')
  $ex = @(Get-Process explorer -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
  for ($i = 0; $i -lt $sec * 2; $i++) {
    foreach ($w in @([Windows.Automation.AutomationElement]::RootElement.FindAll([Windows.Automation.TreeScope]::Children, $any))) {
      if ($ex -notcontains $w.Current.ProcessId -or $w.Current.ClassName -match '^(Progman|WorkerW|Shell_TrayWnd)$') { continue }
      $e = $w.FindFirst([Windows.Automation.TreeScope]::Descendants, $byName)
      if ($e) { $r = $e.Current.BoundingRectangle; return @([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2)) }
    }
    Start-Sleep -Milliseconds 500
  }
  return $null
}
function NameOfFile($fence, $file) { return (@(PfJson item list --fence $fence) | Where-Object { $_ -and $_.fileName -eq $file } | Select-Object -First 1).name }

# Opens the item menu of $file in $fence (Shift held with -Shift; with -Also a second file is
# Ctrl+clicked first, so two are selected), lists it, checks Send to.
function FenceMenu($label, $fence, $file, [switch]$Shift, [string]$Also = '') {
  trap { PfTrap $_; continue }
  $n = NameOfFile $fence $file
  if (-not $n) { Check "${label}: the file is in the fence" $false "items: $((@(PfJson item list --fence $fence) | ForEach-Object { $_.fileName }) -join ', ')"; return }
  $p = ItemPt $fence $n
  if ($Also) {
    $q = ItemPt $fence (NameOfFile $fence $Also)
    [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 400
    [G]::KeyDown(0x11); Start-Sleep -Milliseconds 60; [G]::Click($q[0], $q[1]); Start-Sleep -Milliseconds 60; [G]::KeyUp(0x11)
    Start-Sleep -Milliseconds 400
  }
  if ($Shift) { [G]::KeyDown(0x10); Start-Sleep -Milliseconds 80 }
  [G]::RClick($p[0], $p[1])
  $m = MenuWin 8
  if ($Shift) { [G]::KeyUp(0x10) }
  if ($m -eq [IntPtr]::Zero) { Check "${label}: a menu opened" $false "name='$n' fg=$(Fg)"; return }
  Start-Sleep -Milliseconds 800
  $t = MenuTexts $m
  $r = Rect $m
  Log "MENU ${label}: rows=$(@(MenuRows $m).Count) rect=$($r.L),$($r.T),$($r.R),$($r.B): $t"
  Shot "menu-$label"
  Check "${label}: Send to in the menu" ($t -match '(^| \| )Send to( |$)') "$t"
  CloseMenus
  Start-Sleep -Milliseconds 400
}

function ExplorerMenu($label, $dir, $file) {
  trap { PfTrap $_; continue }
  $w = ExplorerAt $dir
  if (-not $w) { Check "${label}: Explorer opened" $false ''; return }
  $h = [IntPtr]$w.HWND
  for ($k = 0; $k -lt 4 -and [L]::GetForegroundWindow() -ne $h; $k++) { Start-Sleep -Milliseconds 500; TitleClick $h }
  $w.Document.SelectItem($w.Document.Folder.ParseName($file), 29)
  Start-Sleep -Milliseconds 500
  Chord 0x10 0x79
  Start-Sleep -Milliseconds 700
  $m = MenuWin 1
  if ($m -eq [IntPtr]::Zero) { [G]::Press(0x57); $m = MenuWin 3 }
  if ($m -eq [IntPtr]::Zero) { $p = ShowMorePoint; if ($p) { [G]::Click($p[0], $p[1]); $m = MenuWin 5 } }
  if ($m -eq [IntPtr]::Zero) { Check "${label}: Explorer's classic menu opened" $false "fg=$(Fg)"; [G]::Press(0x1B); return }
  Start-Sleep -Milliseconds 800
  $t = MenuTexts $m
  Log "MENU ${label}: rows=$(@(MenuRows $m).Count): $t"
  Shot "menu-$label"
  Check "${label}: Send to in Explorer's classic menu" ($t -match '(^| \| )Send to( |$)') "$t"
  CloseMenus
  CloseWindows
}

Case 'setup'
CloseWindows
Set-Culture en-US
New-Item -ItemType Directory -Force $Docs | Out-Null
Set-Content -LiteralPath "$Docs\$DocName" 'doc'
Set-Content -LiteralPath "$Docs\st153-b.docx" 'docx'
New-Item -ItemType Directory -Force "$Docs\st153 子文件夹" | Out-Null
$sc = (New-Object -ComObject WScript.Shell).CreateShortcut("$Docs\st153-link.lnk"); $sc.TargetPath = "$env:WINDIR\notepad.exe"; $sc.Save()
Set-Content -LiteralPath "$PfDesk\$DeskName" 'desk'
$build = [Environment]::OSVersion.Version.Build
$ubr = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion').UBR
Log "windows build $build.$ubr"
$sendTo = "$env:APPDATA\Microsoft\Windows\SendTo"
Log "SendTo folder: $((@(Get-ChildItem -LiteralPath $sendTo -Force -ErrorAction SilentlyContinue | ForEach-Object { $_.Name })) -join ', ')"
$h = Get-ItemProperty 'Registry::HKEY_CLASSES_ROOT\AllFilesystemObjects\shellex\ContextMenuHandlers\SendTo' -ErrorAction SilentlyContinue
Log "SendTo handler: $($h.'(default)')"
$blocked = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Shell Extensions\Blocked' -ErrorAction SilentlyContinue
Log "blocked extensions (HKLM): $(($blocked.PSObject.Properties | Where-Object { $_.Name -notmatch '^PS' } | ForEach-Object { $_.Name }) -join ', ')"
PfStart
Pf fence create --title Plain --rect '40,40,360,220' | Out-Null
Pf fence set Plain sort name | Out-Null
Start-Sleep -Seconds 1
Pf item move "$PfDesk\$DeskName" --to Plain | Out-Null
Pf fence create --portal $Docs --title 'Docs153' --rect '460,40,360,220' | Out-Null
Pf fence set Docs153 sort name | Out-Null
Start-Sleep -Seconds 3
ParkMouse

Case 'Explorer: the classic menu of the same files'
ExplorerMenu 'explorer-docs' $Docs $DocName

Case 'PecoFence: a fence of the desktop'
FenceMenu 'plain' 'Plain' $DeskName
FenceMenu 'plain-shift' 'Plain' $DeskName -Shift

Case 'PecoFence: a folder fence'
FenceMenu 'folder' 'Docs153' $DocName
FenceMenu 'folder-shift' 'Docs153' $DocName -Shift
FenceMenu 'folder-docx' 'Docs153' 'st153-b.docx'
FenceMenu 'folder-lnk' 'Docs153' 'st153-link.lnk'
FenceMenu 'folder-subfolder' 'Docs153' 'st153 子文件夹'
FenceMenu 'folder-two-files' 'Docs153' $DocName -Also 'st153-b.docx'

Case 'PecoFence in Japanese, a folder fence'
Pf settings set language ja | Out-Null
Start-Sleep -Seconds 2
FenceMenu 'ja-folder' 'Docs153' $DocName
FenceMenu 'ja-plain' 'Plain' $DeskName

Case 'the log'
foreach ($l in @(PfLogAll | Where-Object { $_ -match 'item menu|shell verbs|shell menu|shell extension' })) { Log "APP $l" }

Case 'cleanup'
CloseWindows
PfQuit
Remove-Item -LiteralPath "$PfDesk\$DeskName" -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $Docs -Recurse -Force -ErrorAction SilentlyContinue
