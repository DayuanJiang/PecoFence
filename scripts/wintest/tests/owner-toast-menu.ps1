# A right-click on a fence while a Windows toast notification is the foreground window. Windows
# gives a toast the foreground when the window in front closes while the toast shows; from 0.3.2
# on PecoFence then opened no menu at all (it could not take the foreground from the toast, and a
# menu without the foreground used to stay open for good), again and again until the toast went.
# The menus must open, and Esc or a click elsewhere must close them. For a Hyper-V guest with
# Windows 11 or 10 (`where` vm; both show it): on Server 2025 a toast does not get the foreground,
# and the "what the case needs" rows fail there.
trap { PfTrap $_; continue }
$Files = '季度报告.txt', '图.txt', '一个名字比较长的会议记录文件.txt'
function FgInfo() { $f = [G]::GetForegroundWindow(); return "$([G]::Cls($f)) | $([W]::Info($f))" }
function PfMenuWin() { foreach ($m in [W]::Tops('#32768')) { if ([W]::ProcOf($m) -eq 'pecofence') { return $m } }; return [IntPtr]::Zero }
function ToastFront($fg) { return ($fg -match 'New notification|ShellExperienceHost') }
function ShowToast($text) {
  [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
  [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
  $xml = New-Object Windows.Data.Xml.Dom.XmlDocument
  $xml.LoadXml("<toast><visual><binding template='ToastGeneric'><text>owner-toast-menu</text><text>$text</text></binding></visual></toast>")
  $toast = New-Object Windows.UI.Notifications.ToastNotification $xml
  [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\WindowsPowerShell\v1.0\powershell.exe').Show($toast)
}
# Notepad in front, a toast of another program shows, Notepad closes: the toast has the foreground.
function ToastInFront($text) {
  Start-Process notepad.exe; Start-Sleep -Seconds 2
  ShowToast $text; Start-Sleep -Milliseconds 1800
  Get-Process notepad -ErrorAction SilentlyContinue | ForEach-Object { $_.CloseMainWindow() | Out-Null }
  Start-Sleep -Milliseconds 900
  return (FgInfo)
}
function TitleRClick($t) {
  $wr = (FenceGet $t).windowRect
  [G]::RClick([int]($wr.x + $wr.w / 2), [int]($wr.y + 14)); Start-Sleep -Milliseconds 800
  return (PfMenuWin)
}
# As a user who wants the menu gone: Esc, or a click on the bare desktop. True when it closed.
function CloseBy($how) {
  if ($how -eq 'Esc') { [G]::Press(0x1B) } else { $p = FreeSpot 1000 500; [G]::Click($p[0], $p[1]) }
  Start-Sleep -Milliseconds 700
  return ((PfMenuWin) -eq [IntPtr]::Zero)
}
function CliMs() {
  $sw = [Diagnostics.Stopwatch]::StartNew()
  $null = & "$PfApp\pecofence-cli.exe" --timeout 3000 status 2>&1 | Out-String
  return "$([int]$sw.ElapsedMilliseconds) ms exit $LASTEXITCODE"
}
# A menu that nothing closed would hold every later step: an entry click ends it.
function Unstick() {
  $m = PfMenuWin
  if ($m -eq [IntPtr]::Zero) { return }
  $r = Rect $m; [G]::Click($r.L + 40, $r.T + 12); Start-Sleep -Milliseconds 800
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  Log 'a stuck menu was ended by a click on its first row'
}
function ToastGone() { DeskClick | Out-Null; Start-Sleep -Seconds 7 }

Case 'setup'
CloseWindows
foreach ($f in $Files) { Remove-Item -LiteralPath (Join-Path $PfDesk $f) -Force -ErrorAction SilentlyContinue }
foreach ($f in $Files) { DeskFile $f | Out-Null }
PfStart -Script @('sleep 3600000')
Start-Sleep -Seconds 5
ClearFences | Out-Null
Pf fence create --title '文件' --rect '40,60,420,260' | Out-Null
Start-Sleep -Seconds 1
Pf item move $Files[0] $Files[1] $Files[2] --to '文件' | Out-Null
Start-Sleep -Seconds 1
$names = @(@(PfJson item list --fence '文件') | Where-Object { $_ -and $_.name } | ForEach-Object { $_.name } | Sort-Object)
Check 'three files in the fence' ($names.Count -eq 3) ($names -join ', ')
ParkMouse

Case 'control: no toast'
$m = TitleRClick '文件'
Check 'control: a right-click on the fence title opens the fence menu' ($m -ne [IntPtr]::Zero) "fg=$(FgInfo)"
Check 'control: Esc closes it' (CloseBy 'Esc') ''
DeskClick | Out-Null

foreach ($round in 1, 2, 3) {
  Case "fence menu, toast in front, round $round"
  $fg0 = ToastInFront "round $round"
  Shot "toast-$round-before-rclick"
  $m1 = TitleRClick '文件'
  $fg1 = FgInfo
  Shot "toast-$round-first-rclick"
  $esc = CloseBy 'Esc'
  Unstick
  # The first menu may have taken the foreground from the toast: a second toast in front.
  ToastGone
  $fg2 = ToastInFront "round $round, second"
  $m2 = TitleRClick '文件'
  $fg3 = FgInfo
  $click = CloseBy 'click'
  Unstick
  $cli = CliMs
  Log "TOAST round ${round}: fg before=$fg0; first right-click menu=$m1 fg=$fg1 closed by Esc=$esc; fg before the second=$fg2; second menu=$m2 fg=$fg3 closed by a desktop click=$click; CLI status $cli"
  Check "round ${round}: the toast had the foreground after the window in front closed (what the case needs)" ((ToastFront $fg0) -and (ToastFront $fg2)) "foreground: $fg0 // $fg2"
  Check "round ${round}: the right-click on the fence title opens the fence menu" ($m1 -ne [IntPtr]::Zero) "foreground before: $fg0; after: $fg1"
  Check "round ${round}: Esc closes it" (($m1 -ne [IntPtr]::Zero) -and $esc) ''
  Check "round ${round}: the next right-click opens it again" ($m2 -ne [IntPtr]::Zero) "foreground before: $fg2; after: $fg3"
  Check "round ${round}: a click on the desktop closes it" (($m2 -ne [IntPtr]::Zero) -and $click) ''
  Check "round ${round}: PecoFence answers afterwards" ($cli -match ' exit 0$') $cli
  ToastGone
}

Case 'item menu, toast in front'
$fg0 = ToastInFront 'item menu'
$pt = ItemPt '文件' $names[0]
[G]::RClick($pt[0], $pt[1]); Start-Sleep -Milliseconds 1500
$m = PfMenuWin
$fg1 = FgInfo
Shot 'toast-item-menu'
$esc = CloseBy 'Esc'
Unstick
Check 'item menu: the toast had the foreground (what the case needs)' (ToastFront $fg0) $fg0
Check 'item menu: a right-click on a file in the fence opens its menu' ($m -ne [IntPtr]::Zero) "file '$($names[0])'; foreground after: $fg1"
Check 'item menu: Esc closes it' (($m -ne [IntPtr]::Zero) -and $esc) ''
ToastGone

Case 'fence menu, toast in front, an entry clicked'
$fg0 = ToastInFront 'entry'
$m = TitleRClick '文件'
$picked = MenuPick '^Roll up' $m
Start-Sleep -Milliseconds 900
$rolled = (FenceGet '文件').rolledUp
$closed = (PfMenuWin) -eq [IntPtr]::Zero
Unstick
Check 'entry: the toast had the foreground (what the case needs)' (ToastFront $fg0) $fg0
Check 'entry: a click on 「Roll up」 in the menu rolls the fence up and closes the menu' (($picked -eq 'Roll up') -and $rolled -and $closed) "picked '$picked' rolledUp=$rolled menu closed=$closed"
Pf fence unroll '文件' | Out-Null
ToastGone

# The tray icon's own message, as menus-foreground.ps1 sends it, so that no click moves the
# foreground first: in the guests the icon sits in the taskbar's overflow flyout, and the click
# that opens the flyout takes the foreground from the toast (the tray menu then opened before the
# fix as well).
foreach ($how in 'Esc', 'click') {
  Case "tray menu, toast in front, closed by $how"
  $fg0 = ToastInFront "tray menu, $how"
  $ctl = [G]::FindWindowW('PecoFence.Control', [NullString]::Value)
  [W]::PostMessageW($ctl, 0x8003, [IntPtr](1500 -bor (830 -shl 16)), [IntPtr](0x7B -bor (1 -shl 16))) | Out-Null
  Start-Sleep -Milliseconds 1000
  $m = PfMenuWin
  $fg1 = FgInfo
  Shot "toast-tray-menu-$how"
  $closed = CloseBy $how
  Unstick
  Check "tray menu ($how): the toast had the foreground (what the case needs)" (ToastFront $fg0) $fg0
  Check "tray menu ($how): the tray icon's message opens the tray menu" ($m -ne [IntPtr]::Zero) "foreground after: $fg1"
  Check "tray menu ($how): $how closes it" (($m -ne [IntPtr]::Zero) -and $closed) ''
  ToastGone
}

foreach ($how in 'Esc', 'click') {
  Case "a toast arrives while the fence menu is open, closed by $how"
  $m = TitleRClick '文件'
  $fg0 = FgInfo
  ShowToast "menu open, $how"; Start-Sleep -Milliseconds 2500
  $fg1 = FgInfo
  $still = (PfMenuWin) -ne [IntPtr]::Zero
  Shot "toast-while-menu-$how"
  $closed = CloseBy $how
  Unstick
  $cli = CliMs
  Check "toast while the menu is open ($how): the menu opened" ($m -ne [IntPtr]::Zero) $fg0
  Check "toast while the menu is open ($how): it is still open 2.5 s after the toast came" $still "foreground: $fg1"
  Check "toast while the menu is open ($how): $how closes it" ($still -and $closed) ''
  Check "toast while the menu is open ($how): PecoFence answers afterwards" ($cli -match ' exit 0$') $cli
  Start-Sleep -Seconds 7
}

Case 'cleanup'
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
$left = (PfMenuWin) -ne [IntPtr]::Zero
Check 'no menu is left open at the end' (-not $left) ''
Unstick
if (PfAlive) { ClearFences | Out-Null }
foreach ($f in $Files) { Remove-Item -LiteralPath (Join-Path $PfDesk $f) -Force -ErrorAction SilentlyContinue }
