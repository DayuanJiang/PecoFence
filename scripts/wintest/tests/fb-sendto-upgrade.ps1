# Feedback #125: shortcuts the 0.3.1 build filed under the wrong rule, after the update. The 0.3.1
# build (--extra, from origin/main f6ff439) files a folder shortcut and an empty .lnk under rules
# that split programs from shortcuts; then the build under test starts on the same config, and
# 「立即应用整理规则」 from the tray menu files them again. A shortcut placed by hand stays put.
#   run.sh --box <box> --extra <0.3.1 build> tests/fb-sendto-upgrade.ps1     (about 1 minute)
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
$DeskLink = (Get-ChildItem -LiteralPath "$env:APPDATA\Microsoft\Windows\SendTo" -Force | Where-Object { $_.Extension -eq '.DeskLink' } | Select-Object -First 1).FullName
function CleanMade() { foreach ($f in @(Get-ChildItem -LiteralPath $PfDesk -Force | Where-Object { $_.Name -like 'st-*' })) { Remove-Item -LiteralPath $f.FullName -Recurse -Force -ErrorAction SilentlyContinue } }
function ItemOf($file) { return (@(PfJson item list) | Where-Object { $_ -and $_.fileName -eq $file } | Select-Object -First 1) }
function Where1($file) { $it = ItemOf $file; if (-not $it) { return '(no item)' }; return "$($it.fenceTitle) by $($it.assignedBy) '$($it.rule)'" }
function SendTo($name) { $r = [SendToDrop]::Drop($DeskLink, (Join-Path $Src $name)); Start-Sleep -Seconds 2; return $r }
$Files = @('st-folder - Shortcut.lnk', 'st-note - Shortcut.lnk', 'st-tool - Shortcut.lnk', 'st-slow.lnk')
function State() { return (($Files | ForEach-Object { "$_ = $(Where1 $_)" }) -join '; ') }

Case 'setup on the 0.3.1 build'
CloseWindows
Set-Culture en-US
CleanMade
New-Item -ItemType Directory -Force $Src, "$Src\st-folder", "$Src\st-other" | Out-Null
Copy-Item "$env:WINDIR\System32\notepad.exe" "$Src\st-tool.exe"
Set-Content "$Src\st-note.txt" 'note'
$sc = (New-Object -ComObject WScript.Shell).CreateShortcut("$Src\st-ready.lnk"); $sc.TargetPath = "$Src\st-note.txt"; $sc.Save()
$NewBin = $PfBin
$PfBin = $PfExtra
PfStart
Pf fence create --title Progs --rect '40,560,300,200' | Out-Null
Pf fence create --title Links --rect '360,560,300,200' | Out-Null
Pf rule add --name 'programs to Progs' --type programs --to Progs --index 0 | Out-Null
Pf rule add --name 'shortcuts to Links' --type shortcuts --to Links --index 1 | Out-Null
foreach ($n in 'st-folder', 'st-note.txt', 'st-tool.exe') { Log "send to ${n}: $(SendTo $n)" }
[IO.File]::WriteAllBytes("$PfDesk\st-slow.lnk", [byte[]]@())
Start-Sleep -Milliseconds 700
[IO.File]::WriteAllBytes("$PfDesk\st-slow.lnk", [IO.File]::ReadAllBytes("$Src\st-ready.lnk"))
Start-Sleep -Seconds 2
$s0 = State
Check 'on 0.3.1 the folder shortcut and the empty .lnk land in Progs' ((ItemFence 'st-folder - Shortcut.lnk') -eq 'Progs' -and (ItemFence 'st-slow.lnk') -eq 'Progs') $s0

Case 'the build under test on the same config'
PfQuit
Copy-Item "$NewBin\*" $PfApp -Force
$PfBin = $NewBin
PfStart
Start-Sleep -Seconds 2
$s1 = State
Check 'after the update nothing moves by itself' ($s1 -eq $s0) "before: $s0 | now: $s1"
Log "send to st-other: $([SendToDrop]::Drop($DeskLink, (Join-Path $Src 'st-other')))"
Start-Sleep -Seconds 2
Pf item move "$PfDesk\st-other - Shortcut.lnk" --to Progs | Out-Null
Start-Sleep -Seconds 1
Check 'a folder shortcut put in Progs by hand' ((ItemFence 'st-other - Shortcut.lnk') -eq 'Progs') "$(Where1 'st-other - Shortcut.lnk')"
$dry = PfJson rule apply --dry-run
Log "dry run: $((@($dry.moves) | Where-Object { $_ } | ForEach-Object { "$($_.name): $($_.fromTitle) -> $($_.toTitle) ($($_.ruleName))" }) -join '; ')"

Case 'Apply sorting rules now from the tray menu'
$t = TrayPick 'Apply sorting rules now'
if (-not $t) { Log "tray menu: $script:LastTrayMenu; CLI instead"; Pf rule apply | Out-Null }
Start-Sleep -Seconds 2
$s2 = State
Shot 'after-apply'
Check 'the tray menu entry was used' ([bool]$t) "picked '$t'; menu: $script:LastTrayMenu"
Check 'the folder shortcut moves to Links' ((ItemFence 'st-folder - Shortcut.lnk') -eq 'Links') $s2
Check 'the once-empty .lnk to a .txt moves to Links' ((ItemFence 'st-slow.lnk') -eq 'Links') $s2
Check 'the program shortcut stays in Progs, the .txt shortcut in Links' ((ItemFence 'st-tool - Shortcut.lnk') -eq 'Progs' -and (ItemFence 'st-note - Shortcut.lnk') -eq 'Links') $s2
Check 'the folder shortcut placed by hand stays in Progs' ((ItemFence 'st-other - Shortcut.lnk') -eq 'Progs') "$(Where1 'st-other - Shortcut.lnk')"

Case 'cleanup'
CloseWindows
PfQuit
CleanMade
