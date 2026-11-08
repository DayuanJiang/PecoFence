# scripts/wintest/smart.ps1: helpers for the smart fence (智能栅栏) and filter (筛选) tests:
# Everything 1.4 in pf's session, files with a given age, and a smart fence's state as
# `fence get` and `item list` report it. lib.ps1 dot-sources it.
#
# Everything must be installed on the box (once, as Administrator; the installer adds its
# service, so pf's copy indexes NTFS without asking for rights):
#   box.sh ssh <box> 'Invoke-WebRequest -UseBasicParsing https://www.voidtools.com/Everything-1.4.1.1032.x64-Setup.exe -OutFile C:\ev14.exe; Start-Process C:\ev14.exe -ArgumentList /S -Wait'

$PfDl = [Environment]::GetFolderPath('UserProfile') + '\Downloads'
$PfDocs = [Environment]::GetFolderPath('MyDocuments')
$EvExe = 'C:\Program Files\Everything\Everything.exe'
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class EvW { [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindowW(string c, string t); }
"@

function EvInstalled() { return (Test-Path $EvExe) }
function EvRunning() { return [EvW]::FindWindowW('EVERYTHING_TASKBAR_NOTIFICATION', $null) -ne [IntPtr]::Zero }
# Starts Everything in pf's session (true once its IPC window is there).
function EvStart() {
  if (-not (EvInstalled)) { Log 'Everything is not installed on this box (see smart.ps1)'; return $false }
  if (-not (EvRunning)) { Start-Process $EvExe -ArgumentList '-startup' -WorkingDirectory (Split-Path $EvExe) }
  for ($i = 0; $i -lt 40 -and -not (EvRunning); $i++) { Start-Sleep -Milliseconds 500 }
  return (EvRunning)
}
# Ends pf's Everything (the service keeps running for the next start).
function EvStop() {
  if (EvRunning) { Start-Process $EvExe -ArgumentList '-exit' -WorkingDirectory (Split-Path $EvExe) -Wait }
  for ($i = 0; $i -lt 20 -and (EvRunning); $i++) { Start-Sleep -Milliseconds 500 }
  return (-not (EvRunning))
}

# A file in $dir created and written $days days ago.
function AgedFile($dir, $name, $days) {
  New-Item -ItemType Directory -Force $dir | Out-Null
  $p = Join-Path $dir $name
  Set-Content -LiteralPath $p -Value "sample $name" -Encoding UTF8
  $t = (Get-Date).AddDays(-$days)
  $i = Get-Item -LiteralPath $p; $i.CreationTime = $t; $i.LastWriteTime = $t
  return $p
}
# pf's Downloads with six files of the last week and six older ones (and nothing else).
$SmartNew = @('quarterly report.pdf', 'meeting notes.docx', 'product shot.png', '安装程序.msi', '数据表.xlsx', 'photos.zip')
$SmartOld = @('old contract.pdf', 'annual 2025.pptx', 'drivers.zip', 'readme.txt', 'wallpaper.jpg', '简历.docx')
function DownloadsScene() {
  New-Item -ItemType Directory -Force $PfDl | Out-Null
  Get-ChildItem -LiteralPath $PfDl -Force -ErrorAction SilentlyContinue | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
  $new = @(0.05, 0.4, 1.3, 2.5, 3.6, 5.2); $old = @(12, 40, 90, 200, 25, 60)
  for ($i = 0; $i -lt 6; $i++) { AgedFile $PfDl $SmartNew[$i] $new[$i] | Out-Null; AgedFile $PfDl $SmartOld[$i] $old[$i] | Out-Null }
}
function ClearDownloads() { Get-ChildItem -LiteralPath $PfDl -Force -ErrorAction SilentlyContinue | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue }

# A smart fence's `smart` field ({spec, source, total, shown, query}) and the names it shows.
# JSON for `fence set` goes through PowerShell 5.1, which drops bare double quotes on the way
# to a program: write them as \" (see smart-fences.ps1).
function SmartOf($fence) { return (FenceGet $fence).smart }
function SmartNames($fence) { return @(@(PfJson item list --fence $fence) | Where-Object { $_ -and $_.name } | ForEach-Object { $_.name }) }
function SmartLine($fence) { $s = SmartOf $fence; return "source=$($s.source) total=$($s.total) shown=$($s.shown) names=$((SmartNames $fence) -join ', ') query=$($s.query)" }
# Waits until $ok (a scriptblock over $s = the smart field, $n = the names, always an array)
# holds.
function WaitSmart($fence, [scriptblock]$ok, $sec = 20) {
  for ($i = 0; $i -lt $sec * 2; $i++) {
    $s = SmartOf $fence; $n = @(SmartNames $fence)
    if (& $ok) { return $true }
    Start-Sleep -Milliseconds 500
  }
  return $false
}
# A fence's filter ({text, shown}; $null when it has none), and waits on it.
function FilterOf($fence) { return (FenceGet $fence).filter }
function WaitFilterShown($fence, $want, $sec = 5) {
  for ($i = 0; $i -lt $sec * 4; $i++) { $f = FilterOf $fence; if ($f -and $f.shown -eq $want) { return $true }; Start-Sleep -Milliseconds 250 }
  return $false
}
function WaitFilterGone($fence, $sec = 5) {
  for ($i = 0; $i -lt $sec * 4; $i++) { if ($null -eq (FilterOf $fence)) { return $true }; Start-Sleep -Milliseconds 250 }
  return $false
}
function FilterBoxOpen() { $h = [G]::FindWindowW('PecoFence.Filter', [NullString]::Value); return ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) }
# Clicks the first icon of a fence so that its window has the keyboard.
function FocusFence($fence) { $p = CellPt $fence 0; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 400 }
# Right-clicks the middle of a fence's title row; the menu that opens.
function FenceMenuOf($fence) {
  $wr = (FenceGet $fence).windowRect
  [G]::RClick([int]($wr.x + $wr.w / 2), [int]($wr.y + 14)); Start-Sleep -Milliseconds 700
  return (MenuWin 5)
}
# Opens the submenu of row $regex in menu $menu; the submenu whose texts match $inside.
function OpenSubmenu($menu, $regex, $inside) {
  $p = MenuPoint $menu $regex
  if (-not $p) { return [IntPtr]::Zero }
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 800
  foreach ($m in @([W]::Tops('#32768'))) { if ((MenuTexts $m) -match $inside) { return $m } }
  return [IntPtr]::Zero
}
# Picks 新建智能栅栏… in 添加到桌面 of the open fence menu $m; the picked text or $null.
function PickNewSmart($m) {
  $sub = OpenSubmenu $m '^Add to desktop' 'New smart fence'
  if ($sub -eq [IntPtr]::Zero) { Log "no Add to desktop submenu with New smart fence among: $(MenuTexts $m)"; return $null }
  return (MenuPick '^New smart fence' $sub)
}
