# Private feedback #125 point 1: 「发送到 › 桌面快捷方式」 and the sorting rules. Every way to make
# the shortcut (Explorer's Send to menu, PecoFence's own item menu, the SendTo drop target the menu
# uses) for every kind of target, under the first-run rules (part A), under rules that split
# programs from shortcuts (B), with the first rule cut down to programs (C), and with only a .txt
# extension rule and a programs rule (D); plus slow writers (an empty .lnk filled later, a .tmp
# renamed to .lnk). Each check reads where the new shortcut landed (`item list`) and the app log's
# "desktop item routed" line. Outcomes the 0.3.1 build gets wrong: a shortcut to a folder counted
# as a program, and an empty .lnk filed before its target could be read.
#   run.sh --box <box> tests/fb-sendto-rules.ps1        (about 2 minutes; --var Part=B for one part)

# Desktop events with the file size at that moment (what the shell writes when), and the SendTo
# entry's drop target, which Explorer's Send to menu hands the selection to.
Add-Type @'
using System; using System.IO; using System.Text; using System.Diagnostics; using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class Fsw {
  static FileSystemWatcher w; static List<string> ev = new List<string>(); static Stopwatch sw = Stopwatch.StartNew();
  public static void Start(string dir) {
    w = new FileSystemWatcher(dir); w.IncludeSubdirectories = false; w.InternalBufferSize = 65536;
    w.NotifyFilter = NotifyFilters.FileName | NotifyFilters.DirectoryName | NotifyFilters.Size | NotifyFilters.LastWrite | NotifyFilters.Attributes | NotifyFilters.CreationTime;
    w.Created += (s, e) => Add("created", e.Name, e.FullPath);
    w.Changed += (s, e) => Add("changed", e.Name, e.FullPath);
    w.Deleted += (s, e) => Add("deleted", e.Name, null);
    w.Renamed += (s, e) => Add("renamed " + e.OldName + " ->", e.Name, e.FullPath);
    w.EnableRaisingEvents = true;
  }
  static void Add(string what, string name, string path) {
    long len = -1; try { if (path != null && File.Exists(path)) len = new FileInfo(path).Length; } catch {}
    lock (ev) ev.Add(sw.ElapsedMilliseconds + "ms " + what + " " + name + " len=" + len);
  }
  public static string Take() { lock (ev) { var s = string.Join(" | ", ev); ev.Clear(); return s; } }
}
[ComImport, Guid("43826d1e-e718-42ee-bc55-a1e261c37bfe"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface IShellItemP {
  [PreserveSig] int BindToHandler(IntPtr pbc, [In] ref Guid bhid, [In] ref Guid riid, out IntPtr ppv);
}
[StructLayout(LayoutKind.Sequential)] public struct PTL { public int x; public int y; }
[ComImport, Guid("00000122-0000-0000-C000-000000000046"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface IDropTargetP {
  [PreserveSig] int DragEnter(IntPtr pDataObj, uint keys, PTL pt, ref uint effect);
  [PreserveSig] int DragOver(uint keys, PTL pt, ref uint effect);
  [PreserveSig] int DragLeave();
  [PreserveSig] int Drop(IntPtr pDataObj, uint keys, PTL pt, ref uint effect);
}
public static class SendToDrop {
  [DllImport("shell32.dll", CharSet=CharSet.Unicode, PreserveSig=false)]
  static extern void SHCreateItemFromParsingName(string path, IntPtr pbc, [In] ref Guid riid, [MarshalAs(UnmanagedType.Interface)] out IShellItemP item);
  // What Explorer's Send To menu does: the SendTo entry's drop target takes the item's data object.
  public static string Drop(string target, string src) {
    var iid = typeof(IShellItemP).GUID;
    IShellItemP t, s;
    SHCreateItemFromParsingName(target, IntPtr.Zero, ref iid, out t);
    SHCreateItemFromParsingName(src, IntPtr.Zero, ref iid, out s);
    Guid sfui = new Guid("3981e225-f559-11d3-8e3a-00c04f6837d5"), dob = new Guid("b8c0bd9f-ed24-455c-83e6-d5390c4fe8c4");
    Guid iidDt = new Guid("00000122-0000-0000-C000-000000000046"), iidDo = new Guid("0000010e-0000-0000-C000-000000000046");
    IntPtr pdt, pdo;
    int h1 = t.BindToHandler(IntPtr.Zero, ref sfui, ref iidDt, out pdt);
    int h2 = s.BindToHandler(IntPtr.Zero, ref dob, ref iidDo, out pdo);
    if (h1 != 0 || h2 != 0) return "bind failed " + h1.ToString("X") + " " + h2.ToString("X");
    var dt = (IDropTargetP)Marshal.GetObjectForIUnknown(pdt);
    uint eff = 7; var pt = new PTL();
    int h3 = dt.DragEnter(pdo, 1, pt, ref eff);
    uint e1 = eff; eff = 7;
    int h4 = dt.Drop(pdo, 1, pt, ref eff);
    return "enter=" + h3.ToString("X") + " eff=" + e1 + " drop=" + h4.ToString("X") + " eff=" + eff;
  }
}
'@

$Src = "$PfRun\src"
$SendTo = "$env:APPDATA\Microsoft\Windows\SendTo"
$DeskLinkFile = @(Get-ChildItem -LiteralPath $SendTo -Force -ErrorAction SilentlyContinue | Where-Object { $_.Extension -eq '.DeskLink' } | Select-Object -First 1)
function DeskNames() { return @(Get-ChildItem -LiteralPath $PfDesk -Force | ForEach-Object { $_.Name }) }
function CleanLinks() { foreach ($f in @(Get-ChildItem -LiteralPath $PfDesk -Force | Where-Object { $_.Name -like 'st-*.lnk' -or $_.Name -like 'st-*.tmp' })) { Remove-Item -LiteralPath $f.FullName -Force -ErrorAction SilentlyContinue } }
function CleanMade() { foreach ($f in @(Get-ChildItem -LiteralPath $PfDesk -Force | Where-Object { $_.Name -like 'st-*' })) { Remove-Item -LiteralPath $f.FullName -Recurse -Force -ErrorAction SilentlyContinue } }

function ItemOf($file) { return (@(PfJson item list) | Where-Object { $_ -and $_.fileName -eq $file } | Select-Object -First 1) }
function ItemInfo($it) {
  if (-not $it) { return '(no item)' }
  return "fence='$($it.fenceTitle)' kind=$($it.kind) by=$($it.assignedBy) rule='$($it.rule)' target='$($it.shortcutTarget)' size=$($it.size)"
}
# The app log's routing lines, all of them so far (a mark for RouteLines).
function RouteLines() { return @(PfLogAll | Where-Object { $_ -match 'desktop item routed' }) }

function ExplorerAt($dir) {
  foreach ($k in 1..40) {
    foreach ($x in @((New-Object -ComObject Shell.Application).Windows())) { $f = try { $x.Document.Folder.Self.Path } catch { $null }; if ($f -eq $dir) { return $x } }
    if ($k -eq 1) { Start-Process explorer.exe "`"$dir`"" }
    Start-Sleep -Milliseconds 250
  }
  return $null
}

# In an open classic menu: hover Send to, then click Desktop (create shortcut) in its submenu.
function PickSendToDesktop($m) {
  $p = MenuPoint $m '^Send to'
  if (-not $p) { Log "no Send to among: $(MenuTexts $m)"; CloseMenus; return 'no Send to' }
  [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 900
  $sub = [IntPtr]::Zero
  foreach ($k in 1..24) {
    foreach ($t in @([W]::Tops('#32768'))) { if ((MenuTexts $t) -match 'Desktop \(create shortcut\)') { $sub = $t } }
    if ($sub -ne [IntPtr]::Zero) { break }
    if ($k -eq 6) { [G]::Click($p[0], $p[1]) }
    Start-Sleep -Milliseconds 250
  }
  if ($sub -eq [IntPtr]::Zero) { Log 'no Send to submenu'; CloseMenus; return 'no submenu' }
  if ($script:ShotMenu) { Shot $script:ShotMenu; $script:ShotMenu = $null }
  $t = MenuPick '^Desktop \(create shortcut\)' $sub
  return "menu '$t'"
}

# Explorer: select the item, Shift+F10 (Windows 11's menu), Show more options, Send to, Desktop.
function SendToByExplorer($name) {
  trap { PfTrap $_; continue }
  $w = ExplorerAt $Src
  if (-not $w) { return 'no Explorer window' }
  # A window that has just opened may not be in front yet: Shift+F10 then opens the desktop's
  # menu (View, Sort by, ...). Click its title until it is the foreground window.
  $h = [IntPtr]$w.HWND
  for ($k = 0; $k -lt 4 -and [L]::GetForegroundWindow() -ne $h; $k++) { Start-Sleep -Milliseconds 500; TitleClick $h }
  if ([L]::GetForegroundWindow() -ne $h) { Log "Explorer not in front; fg $(Fg)"; return 'Explorer not in front' }
  $w.Document.SelectItem($w.Document.Folder.ParseName($name), 29)
  Start-Sleep -Milliseconds 500
  Chord 0x10 0x79
  Start-Sleep -Milliseconds 700
  $m = MenuWin 1
  if ($m -eq [IntPtr]::Zero) {
    # Windows 11's menu, opened from the keyboard, shows access keys: W is Show more options.
    [G]::Press(0x57)
    $m = MenuWin 3
  }
  if ($m -eq [IntPtr]::Zero) {
    $p = ShowMorePoint
    if (-not $p) { Log "no menu; fg $(Fg)"; [G]::Press(0x1B); return 'no menu' }
    [G]::Click($p[0], $p[1])
    $m = MenuWin 5
  }
  if ($m -eq [IntPtr]::Zero) { [G]::Press(0x1B); return 'no classic menu' }
  return (PickSendToDesktop $m)
}

# Windows 11's menu entry Show more options, in any window of Explorer (File Explorer's menu
# sits in its own window's tree, which the kit's ShellMenuPoint leaves out).
function ShowMorePoint($sec = 4) {
  $any = [Windows.Automation.Condition]::TrueCondition
  $byName = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::NameProperty, 'Show more options')
  $ex = @(Get-Process explorer -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
  for ($i = 0; $i -lt $sec * 2; $i++) {
    foreach ($w in @([Windows.Automation.AutomationElement]::RootElement.FindAll([Windows.Automation.TreeScope]::Children, $any))) {
      if ($ex -notcontains $w.Current.ProcessId -or $w.Current.ClassName -match '^(Progman|WorkerW|Shell_TrayWnd)$') { continue }
      $e = $w.FindFirst([Windows.Automation.TreeScope]::Descendants, $byName)
      if ($e) { $r = $e.Current.BoundingRectangle; Log "Show more options in $($w.Current.ClassName) at $r"; return @([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2)) }
    }
    Start-Sleep -Milliseconds 500
  }
  return $null
}

function SendToByDrop($name) { return [SendToDrop]::Drop($DeskLinkFile[0].FullName, (Join-Path $Src $name)) }

# PecoFence's item menu (Windows' own entries below ours) on an icon of fence Src.
function SendToByFence($name) {
  trap { PfTrap $_; continue }
  $m = RClickItem 'Src' $name
  if ($m -eq [IntPtr]::Zero) { return 'no item menu' }
  return (PickSendToDesktop $m)
}

# Makes the shortcut one way, waits for it, checks the fence it lands in (and the log's routing
# line: its outcome and type). $want = fence title; $log = regex the new routing lines must match.
function Probe($set, $how, $name, $want, $log) {
  trap { PfTrap $_; continue }
  $before = DeskNames
  $mark = (RouteLines).Count
  [Fsw]::Take() | Out-Null
  $r = switch ($how) { 'explorer' { SendToByExplorer $name } 'fence menu' { SendToByFence $name } 'drop target' { SendToByDrop $name } }
  $new = $null
  for ($k = 0; $k -lt 50 -and -not $new; $k++) { $new = @(DeskNames | Where-Object { $before -notcontains $_ -and $_ -like '*.lnk' }) | Select-Object -First 1; if (-not $new) { Start-Sleep -Milliseconds 100 } }
  $it = $null
  for ($k = 0; $k -lt 12 -and $new; $k++) { Start-Sleep -Milliseconds 250; $it = ItemOf $new; if ($it -and $it.fenceTitle -eq $want) { break } }
  $ev = [Fsw]::Take()
  $lines = @(RouteLines | Select-Object -Skip $mark)
  $line = ($lines | Select-Object -Last 1)
  Log "PROBE $set $how $name -> '$new' ($r): $(ItemInfo $it); events: $ev; log: $($lines -join ' || ')"
  Check "$set | $how | $name -> $want" ($new -and $it -and $it.fenceTitle -eq $want) "'$new' $(ItemInfo $it) | $r | events: $ev"
  if ($log) { Check "$set | $how | ${name}: the log says why" ($line -match $log) "$line" }
}

# The facts of a routing line as short text (for the job log and the report).
function RouteFacts($line) { if ($line -match 'desktop item routed (.*)$') { return $Matches[1] }; return $line }

$Names = @('st-tool.exe', 'st-note.txt', 'st-report.docx', 'st-paper.pdf', 'st-folder', 'st-v1.2')
function LinkName($n) { return "$([IO.Path]::GetFileNameWithoutExtension($n)) - Shortcut.lnk" }
if (-not $Part) { $Part = 'ABCD' }

Case 'setup'
CloseWindows
Set-Culture en-US
CleanMade
New-Item -ItemType Directory -Force $Src, "$Src\st-folder", "$Src\st-v1.2" | Out-Null
Copy-Item "$env:WINDIR\System32\notepad.exe" "$Src\st-tool.exe"
Set-Content "$Src\st-note.txt" 'note'
Set-Content "$Src\st-report.docx" 'report'
Set-Content "$Src\st-paper.pdf" 'paper'
$sc = (New-Object -ComObject WScript.Shell).CreateShortcut("$Src\st-ready.lnk"); $sc.TargetPath = "$Src\st-note.txt"; $sc.Save()
Set-Content "$PfDesk\st-desk.txt" 'desk'
Check 'the SendTo folder has Desktop (create shortcut)' ($DeskLinkFile.Count -eq 1) "$((@(Get-ChildItem -LiteralPath $SendTo -Force | ForEach-Object { $_.Name })) -join ', ')"
[Fsw]::Start($PfDesk)
PfStart
Pf fence create --title Src --rect '40,40,330,200' | Out-Null
Pf fence set Src sort name | Out-Null
Start-Sleep -Seconds 1
Pf item move "$PfDesk\st-desk.txt" --to Src | Out-Null
Start-Sleep -Seconds 1
Check 'a fresh instance has the first-run rules, st-desk.txt is in Src' ((Rules).Count -eq 3 -and (ItemFence 'st-desk.txt') -eq 'Src') "rules: $((Rules | ForEach-Object { "$($_.name) -> $($_.targetTitle)" }) -join '; '); $(ItemInfo (ItemOf 'st-desk.txt'))"
$RuleLog = 'outcome="?rule matched'

function PartA {
trap { PfTrap $_; continue }
Case 'A first-run rules: every way, every target'
Log "rules: $((Rules | ForEach-Object { "$($_.name) -> $($_.targetTitle) $($_.allOf | ConvertTo-Json -Compress -Depth 5)" }) -join ' ;; ')"
$script:ShotMenu = 'a-explorer-send-to-menu'
foreach ($n in $Names) { Probe 'A' 'explorer' $n 'Programs' $RuleLog }
CloseWindows
Shot 'a-first-run-rules-programs-fence'
$script:ShotMenu = 'a-fence-send-to-menu'
Probe 'A' 'fence menu' 'st-desk' 'Programs' $RuleLog
CleanLinks; Start-Sleep -Seconds 1
foreach ($n in 'st-tool.exe', 'st-note.txt', 'st-folder') { Probe 'A' 'drop target' $n 'Programs' $RuleLog }
CleanLinks; Start-Sleep -Seconds 1

Case 'A controls: a copied .lnk, a new .txt'
$mark = (RouteLines).Count
Copy-Item "$Src\st-ready.lnk" "$PfDesk\st-ready.lnk"
Set-Content "$PfDesk\st-new.txt" 'new'
Check 'a copied shortcut goes to Programs' ((WaitItemFence 'st-ready.lnk' 'Programs' 5) -eq 'Programs') "$(ItemInfo (ItemOf 'st-ready.lnk'))"
Check 'a new .txt goes to Files and documents' ((WaitItemFence 'st-new.txt' 'Files and documents' 5) -eq 'Files and documents') "$(ItemInfo (ItemOf 'st-new.txt'))"
Check 'two routing lines in the log' (@(RouteLines | Select-Object -Skip $mark).Count -ge 2) "$((@(RouteLines | Select-Object -Skip $mark) | ForEach-Object { RouteFacts $_ }) -join ' || ')"
Remove-Item -LiteralPath "$PfDesk\st-ready.lnk", "$PfDesk\st-new.txt" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1
}

function PartB {
trap { PfTrap $_; continue }
Case 'B rules that split programs and shortcuts'
foreach ($t in 'Docs', 'Progs', 'Links') { if (-not (PfFence $t)) { Pf fence create --title $t --rect "$(40 + 320 * @('Docs', 'Progs', 'Links').IndexOf($t)),560,300,200" | Out-Null } }
Pf rule add --name 'txt to Docs' --ext txt --to Docs --index 0 | Out-Null
Pf rule add --name 'programs to Progs' --type programs --to Progs --index 1 | Out-Null
Pf rule add --name 'shortcuts to Links' --type shortcuts --to Links --index 2 | Out-Null
Log "rules: $((Rules | ForEach-Object { "$($_.name) -> $($_.targetTitle)" }) -join ' ;; ')"
Probe 'B' 'explorer' 'st-tool.exe' 'Progs' 'what="?programs'
foreach ($n in 'st-note.txt', 'st-report.docx', 'st-paper.pdf', 'st-folder', 'st-v1.2') { Probe 'B' 'explorer' $n 'Links' 'what="?shortcuts' }
CloseWindows
Shot 'b-split-rules-links-fence'
Probe 'B' 'fence menu' 'st-desk' 'Links' 'what="?shortcuts'
Probe 'B' 'drop target' 'st-folder' 'Links' 'what="?shortcuts'
CleanLinks; Start-Sleep -Seconds 1

Case 'B slow writers'
$bytes = [IO.File]::ReadAllBytes("$Src\st-ready.lnk")
$mark = (RouteLines).Count
[Fsw]::Take() | Out-Null
[IO.File]::WriteAllBytes("$PfDesk\st-slow.lnk", [byte[]]@())
Start-Sleep -Milliseconds 700
$early = ItemOf 'st-slow.lnk'
[IO.File]::WriteAllBytes("$PfDesk\st-slow.lnk", $bytes)
$late = WaitItemFence 'st-slow.lnk' 'Links' 5
$lines = @(RouteLines | Select-Object -Skip $mark | ForEach-Object { RouteFacts $_ })
Check 'an empty .lnk waits: in no fence before its link is written' (-not $early -or -not $early.fenceTitle) "before the write: $(ItemInfo $early); log: $($lines -join ' || ')"
Check 'then it lands by its target: Links' ($late -eq 'Links') "$(ItemInfo (ItemOf 'st-slow.lnk')); events: $([Fsw]::Take())"
Check 'the log says it waited for the link' (($lines -join ' ') -match 'shortcut not written yet') "$($lines -join ' || ')"
[IO.File]::WriteAllBytes("$PfDesk\st-named.tmp", $bytes)
Start-Sleep -Milliseconds 400
Rename-Item "$PfDesk\st-named.tmp" 'st-renamed.lnk'
Check 'a link written as .tmp and renamed to .lnk lands in Links' ((WaitItemFence 'st-renamed.lnk' 'Links' 5) -eq 'Links') "$(ItemInfo (ItemOf 'st-renamed.lnk'))"
CleanLinks; Start-Sleep -Seconds 1
foreach ($r in 'txt to Docs', 'programs to Progs', 'shortcuts to Links') { Pf rule remove $r | Out-Null }
}

function PartC {
trap { PfTrap $_; continue }
Case 'C first rule edited to programs only'
Pf rule remove 0 | Out-Null
Pf rule add --name 'Programs only' --type programs --to Programs --index 0 | Out-Null
Log "rules: $((Rules | ForEach-Object { "$($_.name) -> $($_.targetTitle)" }) -join ' ;; ')"
Probe 'C' 'explorer' 'st-tool.exe' 'Programs' 'outcome="?rule matched.*what="?programs'
Probe 'C' 'explorer' 'st-note.txt' 'Desktop' 'outcome="?no rule matched.*what="?shortcuts.*inbox=true'
Probe 'C' 'explorer' 'st-folder' 'Desktop' 'outcome="?no rule matched.*what="?shortcuts'
CloseWindows
Shot 'c-no-shortcut-rule-desktop-fence'
CleanLinks; Start-Sleep -Seconds 1
}

function PartD {
trap { PfTrap $_; continue }
Case 'D only a .txt extension rule and a programs type rule'
for ($k = 0; $k -lt 12 -and (Rules).Count -gt 0; $k++) { Pf rule remove 0 | Out-Null }
if (-not (PfFence 'Docs')) { Pf fence create --title Docs --rect '40,560,300,200' | Out-Null }
Pf rule add --name 'txt to Docs' --ext txt --to Docs --index 0 | Out-Null
Pf rule add --name 'Programs' --type programs --to Programs --index 1 | Out-Null
Log "rules: $((Rules | ForEach-Object { "$($_.name) -> $($_.targetTitle)" }) -join ' ;; ')"
Probe 'D' 'explorer' 'st-tool.exe' 'Programs' 'outcome="?rule matched.*what="?programs'
Probe 'D' 'explorer' 'st-note.txt' 'Desktop' 'outcome="?no rule matched.*what="?shortcuts'
Probe 'D' 'explorer' 'st-report.docx' 'Desktop' 'outcome="?no rule matched.*what="?shortcuts'
Probe 'D' 'explorer' 'st-folder' 'Desktop' 'outcome="?no rule matched.*what="?shortcuts'
CloseWindows
Set-Content "$PfDesk\st-d.txt" 'd'
Check 'D a new .txt file goes to Docs' ((WaitItemFence 'st-d.txt' 'Docs' 5) -eq 'Docs') "$(ItemInfo (ItemOf 'st-d.txt'))"
$bytes = [IO.File]::ReadAllBytes("$Src\st-ready.lnk")
[IO.File]::WriteAllBytes("$PfDesk\st-slow.lnk", [byte[]]@())
Start-Sleep -Milliseconds 700
$early = ItemOf 'st-slow.lnk'
[IO.File]::WriteAllBytes("$PfDesk\st-slow.lnk", $bytes)
$late = WaitItemFence 'st-slow.lnk' 'Desktop' 5
Check 'D an empty .lnk to a .txt: in no fence until written' (-not $early -or -not $early.fenceTitle) "before the write: $(ItemInfo $early)"
Check 'D then no rule matches it: Desktop' ($late -eq 'Desktop') "$(ItemInfo (ItemOf 'st-slow.lnk'))"
CleanLinks; Remove-Item -LiteralPath "$PfDesk\st-d.txt" -Force -ErrorAction SilentlyContinue; Start-Sleep -Seconds 1
}

if ($Part -match 'A') { PartA }
if ($Part -match 'B') { PartB }
if ($Part -match 'C') { PartC }
if ($Part -match 'D') { PartD }

Case 'cleanup'
CloseWindows
PfQuit
CleanMade
