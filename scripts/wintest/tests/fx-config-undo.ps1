# Fence fix round 2026-11-07, area config-undo: checks of S16, S13, S11, S12, U2 and C5 (the
# watchdog checks of C14 are in fx-config-undo-watchdog.ps1). One job, about 5 minutes.
# The C5 part moves the Desktop folder to another C: folder and back at its end.
$PfAllowLogErrors = $true

# ---- S16 config files held open at a start
Add-Type @'
using System.IO;
using System.Collections.Generic;
public static class Hold {
  static List<FileStream> held = new List<FileStream>();
  public static void Lock(string path) { held.Add(new FileStream(path, FileMode.Open, FileAccess.ReadWrite, FileShare.None)); }
  public static void UnlockAll() { foreach (var f in held) f.Dispose(); held.Clear(); }
}
'@
$cfgDir = "$PfApp\config"
function CfgFiles() { return ((Get-ChildItem $cfgDir -File -Recurse | ForEach-Object { "$($_.Name)=$($_.Length)" }) -join ' ') }
function HasP($path) { return ((Get-Content $path -Raw -Encoding UTF8) -match '"title": "P"') }
function LoadLines() { return ((Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match 'config loaded|config file|saves held|config read|locked' } | ForEach-Object { $_.Substring([math]::Min(28, $_.Length)) }) -join ' / ') }
function LockAll() {
  $files = @("$cfgDir\config.json", "$cfgDir\config.bak") + @(Get-ChildItem "$cfgDir\backups" -File | ForEach-Object FullName)
  foreach ($f in $files) { [Hold]::Lock($f) }
  return $files.Count
}
function StartRaw() {
  $env:PECOFENCE_INSTANCE = $PfInst
  $env:RUST_LOG = 'pecofence=debug'
  [IO.File]::WriteAllLines("$PfRun\script.txt", @('sleep 3600000'), (New-Object Text.UTF8Encoding($false)))
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable', '--test-script', "$PfRun\script.txt" -WorkingDirectory $PfApp
}

Case 'S16 setup'
PfStart
Pf fence create --title P --rect '300,150,320,260' | Out-Null
Start-Sleep -Seconds 2
Pf fence move P --x 320 --y 150 | Out-Null
Start-Sleep -Seconds 2
PfQuit
Log "files: $(CfgFiles)"
Check 'setup: config.json and config.bak have fence P' ((HasP "$cfgDir\config.json") -and (HasP "$cfgDir\config.bak")) (CfgFiles)

Case 'S16 A: locked 3 s across a start'
$n = LockAll
StartRaw
Start-Sleep -Seconds 3
[Hold]::UnlockAll()
$null = WaitWin 'PecoFence.Control' 20 -Hidden
Start-Sleep -Seconds 5
$p = PfFence 'P'
Shot 'A-after-3s-lock'
Check 'A a start while the files are locked for 3 s shows the user''s fence P' ($null -ne $p) "locked $n files; P there: $([bool]$p); fences: $((@(PfJson fence list) | Where-Object { $_.title } | ForEach-Object title) -join ', '); $(LoadLines)"
Pf fence create --title Q --rect '700,150,300,200' | Out-Null
Start-Sleep -Seconds 3
Check 'A afterwards config.json and config.bak still have P' ((HasP "$cfgDir\config.json") -and (HasP "$cfgDir\config.bak")) (CfgFiles)
PfQuit

Case 'S16 B: locked 15 s across a start'
$before = (Get-FileHash "$cfgDir\config.json").Hash
$n = LockAll
StartRaw
$null = WaitWin 'PecoFence.Control' 25 -Hidden
Start-Sleep -Seconds 12
$during = @(PfJson fence list) | Where-Object { $_.title } | ForEach-Object title
$toast = ToastOcr 'PecoFence' 3
Shot 'B-during-lock'
Log "B during the lock: fences $($during -join ', '); toast: $toast"
Pf fence move Q --x 760 --y 160 | Out-Null
Start-Sleep -Seconds 2
[Hold]::UnlockAll()
$same = (Get-FileHash "$cfgDir\config.json").Hash -eq $before
Start-Sleep -Seconds 6
$p = PfFence 'P'
$toast2 = ToastOcr 'PecoFence' 3
Shot 'B-after-unlock'
Check 'B while locked: config.json unchanged' $same "files: $(CfgFiles)"
Check 'B after the unlock the user''s fence P shows' ($null -ne $p) "P there: $([bool]$p); during: $($during -join ', '); now: $((@(PfJson fence list) | Where-Object { $_.title } | ForEach-Object title) -join ', '); toast during: $toast | after: $toast2; $(LoadLines)"
Pf fence move P --x 340 --y 150 | Out-Null
Start-Sleep -Seconds 3
PfQuit
Check 'B after the next save config.json and config.bak have P' ((HasP "$cfgDir\config.json") -and (HasP "$cfgDir\config.bak")) (CfgFiles)
PfStart
$p = PfFence 'P'
Check 'B the next start loads P' ($null -ne $p) (LoadLines)
PfQuit

# ---- S13 records of deleted desktop files
$cfg = "$PfApp\config\config.json"
function Count() { try { return [int]((Pf status | ConvertFrom-Json).itemCount) } catch { return -1 } }
function WaitCount($want, $sec = 30) {
  for ($i = 0; $i -lt $sec * 4; $i++) { $c = Count; if ($c -eq $want) { return $c }; Start-Sleep -Milliseconds 250 }
  return (Count)
}
function Orphans() { $j = Get-Content $cfg -Raw -Encoding UTF8 | ConvertFrom-Json; return @($j.items.PSObject.Properties | Where-Object { $_.Value.orphanedSince }).Count }
function Records() { $j = Get-Content $cfg -Raw -Encoding UTF8 | ConvertFrom-Json; return @($j.items.PSObject.Properties).Count }

Case 'S13 1500 files made and deleted'
Get-ChildItem $PfDesk -Filter 's13-*' -Force -ErrorAction SilentlyContinue | Remove-Item -Force
PfStart
Start-Sleep -Seconds 2
$base = Count
$size0 = (Get-Item $cfg).Length
$rec0 = Records
for ($r = 0; $r -lt 5; $r++) {
  for ($i = 0; $i -lt 300; $i++) { [IO.File]::WriteAllText("$PfDesk\s13-r$r-$i.txt", "x") }
  $c1 = WaitCount ($base + 300)
  Get-ChildItem $PfDesk -Filter "s13-r$r-*" | Remove-Item -Force
  $c2 = WaitCount $base
  Log "round ${r}: items $c1 then $c2"
}
Start-Sleep -Seconds 3
$size1 = (Get-Item $cfg).Length
$orph = Orphans
$rec1 = Records
Check 'S13 config.json stays about the same size after 1500 deleted files' ($size1 -lt ($size0 + 50KB)) "size $size0 -> $size1 bytes; records $rec0 -> $rec1, orphaned $orph; items now $(Count) (base $base)"

Case 'S13 a file moved by hand comes back into its fence'
for ($i = 0; $i -lt 5; $i++) { [IO.File]::WriteAllText("$PfDesk\s13-keep-$i.txt", "k") }
$null = WaitCount ($base + 5)
for ($i = 0; $i -lt 5; $i++) { Pf item move "$PfDesk\s13-keep-$i.txt" --to Programs | Out-Null }
Start-Sleep -Seconds 2
$moved = @(0..4 | ForEach-Object { ItemFence "s13-keep-$_.txt" }) -join ','
Get-ChildItem $PfDesk -Filter 's13-keep-*' | Remove-Item -Force
$null = WaitCount $base
Start-Sleep -Seconds 2
$orphKeep = Orphans
for ($i = 0; $i -lt 5; $i++) { [IO.File]::WriteAllText("$PfDesk\s13-keep-$i.txt", "k") }
$null = WaitCount ($base + 5)
Start-Sleep -Seconds 1
$back = @(0..4 | ForEach-Object { ItemFence "s13-keep-$_.txt" }) -join ','
Check 'S13 hand-placed files keep their fence when they come back' (($back -split ',' | Where-Object { $_ -eq 'Programs' }).Count -eq 5) "moved to: $moved; orphaned records while gone: $orphKeep; back in: $back"
Get-ChildItem $PfDesk -Filter 's13-*' -Force -ErrorAction SilentlyContinue | Remove-Item -Force
$null = WaitCount $base
PfQuit
$lines = @(PfLogAll | Where-Object { $_ -match 'records of deleted desktop items dropped' })
Log "drop log lines: $($lines.Count); last: $($lines | Select-Object -Last 1)"

# ---- S11 names ending in dots or spaces
$RenameClass = 'PecoFence.Rename'
$lp = '\\?\' + $PfDesk + '\'
function OnDisk($name) { return [IO.File]::Exists($lp + $name) }
function IdOf($file) { foreach ($it in @(PfJson item list)) { if ($it -and $it.fileName -eq $file) { return $it.id } }; return $null }
function WaitId($file, $sec = 15) { for ($i = 0; $i -lt $sec * 4; $i++) { $id = IdOf $file; if ($id) { return $id }; Start-Sleep -Milliseconds 250 }; return $null }
function Cleanup() { foreach ($e in [IO.Directory]::GetFileSystemEntries($lp, 's11-*')) { try { [IO.File]::Delete($e) } catch { Log "cleanup failed $e" } } }

Case 'S11 setup'
PfStart
Cleanup
Pf fence create --title S11 --rect '300,200,320,220' | Out-Null
Start-Sleep -Seconds 1

Case 'S11 CLI rename'
foreach ($n in 's11-dots...', 's11-space ', 's11-normal.txt') {
  [IO.File]::WriteAllText($lp + $n, 'odd')
  $id = WaitId $n
  $to = 's11-cli-' + ($n -replace '[^a-z]', '')
  $r = Pf item rename ($PfDesk + '\' + $n) $to
  Start-Sleep -Milliseconds 800
  $want = if ($n -like '*.txt') { "$to.txt" } else { $to }
  Check "S11 CLI rename of '$n'" ((OnDisk $want) -and -not (OnDisk $n)) "id=$id; reply: $($r.Trim()); on disk now: $((@([IO.Directory]::GetFileSystemEntries($lp, 's11-*') | ForEach-Object { [IO.Path]::GetFileName($_) }) | ForEach-Object { "'$_'" }) -join ' ')"
}
Cleanup
Start-Sleep -Seconds 1

Case 'S11 F2 rename in a fence'
foreach ($n in 's11-f2dots...', 's11-f2space ') {
  [IO.File]::WriteAllText($lp + $n, 'odd')
  $id = WaitId $n
  Pf item move $id --to S11 | Out-Null
  Start-Sleep -Seconds 1
  $f = PfFence 'S11'
  $wr = $f.windowRect
  # The only icon in the fence: first cell.
  $x = [int]($wr.x + 50); $y = [int]($wr.y + 36 + 8 + 32)
  [G]::Click($x, $y); Start-Sleep -Milliseconds 600
  [G]::Press(0x71); Start-Sleep -Milliseconds 700
  $box = [G]::FindWindowW($RenameClass, [NullString]::Value)
  if ($box -eq [IntPtr]::Zero -or -not [G]::IsWindowVisible($box)) {
    Log "F2: no rename box yet (foreground $(Fg)); click and F2 again"
    [G]::Click($x, $y); Start-Sleep -Milliseconds 600
    [G]::Press(0x71); Start-Sleep -Milliseconds 700
  }
  Chord 0x11 0x41; Start-Sleep -Milliseconds 150
  $to = 's11-ui-' + ($n -replace '[^a-z0-9]', '')
  [G]::Type($to); Start-Sleep -Milliseconds 200
  Shot "F2-$($n.Trim('. '))-typed"
  [G]::Press(0x0D); Start-Sleep -Seconds 1
  $dlg = @([W]::Tops('#32770') | ForEach-Object { [W]::Info($_) + ' :: ' + [W]::Statics($_) }) -join ' | '
  $toast = (@(PfLogAll | Where-Object { $_ -match 'rename' }) | Select-Object -Last 2 | ForEach-Object { $_.Substring([math]::Min(28, $_.Length)) }) -join ' / '
  Shot "F2-$($n.Trim('. '))-after"
  Check "S11 F2 rename of '$n'" ((OnDisk $to) -and -not (OnDisk $n)) "id=$id at $x,$y; dialogs: $dlg; log: $toast; on disk now: $((@([IO.Directory]::GetFileSystemEntries($lp, 's11-*') | ForEach-Object { [IO.Path]::GetFileName($_) }) | ForEach-Object { "'$_'" }) -join ' ')"
  foreach ($h in @([W]::Tops('#32770'))) { [L]::PostMessageW($h, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null }
  [G]::Press(0x1B)
  Cleanup
  Start-Sleep -Seconds 1
}
PfQuit

# ---- S12 a newer version's configuration
$cfgDir = "$PfApp\config"
$cfg = "$cfgDir\config.json"
function Files() { return ((Get-ChildItem $cfgDir -File -Recurse | ForEach-Object { "$($_.Name)=$($_.Length)" }) -join ' ') }

Case 'S12 setup'
PfStart
Pf fence create --title S12 --rect '300,200,300,200' | Out-Null
Start-Sleep -Seconds 2
PfQuit
$j = Get-Content $cfg -Raw -Encoding UTF8
$j = [regex]::Replace($j, '"savedBy": "[^"]*",\s*', '')
$j = $j.Replace('"schemaVersion": 1,', '"schemaVersion": 1, "savedBy": "9.9.9", "futureFeature": {"keep": "me"},')
[IO.File]::WriteAllText($cfg, $j, (New-Object Text.UTF8Encoding($false)))
Check 'setup: config.json says a newer version saved it' ((Get-Content $cfg -Raw) -match '"savedBy": "9.9.9"' -and (Get-Content $cfg -Raw) -match 'futureFeature') (Files)

Case 'S12 this version opens and saves a newer version''s file'
PfStart
Pf fence create --title S12b --rect '700,200,300,200' | Out-Null
Start-Sleep -Seconds 3
PfQuit
$copies = @(Get-ChildItem "$cfgDir\backups" -Filter '*-9.9.9.json' -ErrorAction SilentlyContinue)
$keeps = $copies.Count -eq 1 -and ((Get-Content $copies[0].FullName -Raw) -match 'futureFeature')
$now = Get-Content $cfg -Raw -Encoding UTF8
$by = [regex]::Match($now, '"savedBy": "([^"]*)"').Groups[1].Value
Log "config.json savedBy now '$by'; futureFeature still there: $($now -match 'futureFeature'); files: $(Files)"
Check 'S12 the newer version''s file is kept in the backups before it is replaced' $keeps "copies: $(($copies | ForEach-Object { $_.Name }) -join ', '); files: $(Files)"
Check 'S12 the saved file says which version saved it' ($by -ne '' -and $by -ne '9.9.9') "savedBy='$by'"
$line = @(PfLogAll | Where-Object { $_ -match 'newer version' } | ForEach-Object { $_.Substring([math]::Min(28, $_.Length)) }) -join ' / '
Log "log: $line"

# ---- U2 undo
# the last fence move, resize, icon move between fences and fence delete.
function FRect($t) { $f = PfFence $t; if ($f) { $w = $f.windowRect; return "$($w.x),$($w.y),$($w.w),$($w.h)" }; return 'gone' }
function In($t) { return (@(PfJson item list) | Where-Object { $_ -and $_.fenceTitle -eq $t } | ForEach-Object { $_.fileName } | Sort-Object) -join ',' }
function RuleTo() { $r = @((PfJson rule list).list) | Where-Object { $_.name -eq 'U2r' }; if ($r) { return "$($r.targetTitle)" }; return 'none' }
function Undo() { Chord 0x11 0x5A; Start-Sleep -Milliseconds 1200 }
function FocusTitle($t) { $w = (PfFence $t).windowRect; [G]::Click($w.x + 40, $w.y + 14); Start-Sleep -Milliseconds 500 }
function MenuDelete($t) {
  $w = (PfFence $t).windowRect
  [G]::RClick($w.x + 60, $w.y + 14)
  $m = MenuWin 6
  MenuPick 'Delete fence' $m | Out-Null
  Start-Sleep -Milliseconds 800
  $box = @([W]::Tops('#32770'))
  if ($box.Count -gt 0) { Log "confirm: $([W]::Statics($box[0]))"; ClickButton $box[0] '^&?Yes' | Out-Null }
  Start-Sleep -Milliseconds 1500
}
function Setup() {
  Pf fence create --title U2 --rect '300,200,330,230' | Out-Null
  foreach ($n in 'u2-a', 'u2-b', 'u2-c') { DeskFile "$n.txt" | Out-Null }
  Start-Sleep -Seconds 2
  Pf item move "$PfDesk\u2-a.txt" "$PfDesk\u2-b.txt" "$PfDesk\u2-c.txt" --to U2 | Out-Null
  Pf rule add --name U2r --ext u2x --to U2 | Out-Null
  Start-Sleep -Seconds 1
}

Case 'U2 setup'
Get-ChildItem $PfDesk -Filter 'u2-*' -ErrorAction SilentlyContinue | Remove-Item -Force
PfStart
Setup
Pf fence set --all sort name | Out-Null
Check 'setup: fence U2 with three icons and a rule' ((In 'U2') -eq 'u2-a.txt,u2-b.txt,u2-c.txt' -and (RuleTo) -eq 'U2') "U2 has: $(In 'U2'); rule to: $(RuleTo)"

Case 'U2 Delete fence from its menu: a snapshot, a notice with Undo'
$s0 = @(Snaps).Count
MenuDelete 'U2'
$snaps = @(Snaps)
$notice = [G]::FindWindowW('PecoFence.Notice', [NullString]::Value)
$shown = $notice -ne [IntPtr]::Zero -and [G]::IsWindowVisible($notice)
Shot 'U2-after-menu-delete'
Check 'U2 the menu delete removed the fence' ((FRect 'U2') -eq 'gone') "U2: $(FRect 'U2'); Desktop has: $(In 'Desktop')"
Check 'U2 the menu delete took a snapshot' ($snaps.Count -eq $s0 + 1) "snapshots $s0 -> $($snaps.Count): $(($snaps | ForEach-Object { $_.name }) -join ' | ')"
Check 'U2 a notice offers Undo' $shown "notice=$notice $(if ($shown) { [W]::Info($notice) })"
if ($shown) { $nr = Rect $notice; [G]::Click($nr.R - 50, $nr.B - 30); Start-Sleep -Milliseconds 1500 }
Shot 'U2-after-notice-undo'
Check 'U2 the notice''s Undo brings the fence back with its icons and rule' ((In 'U2') -eq 'u2-a.txt,u2-b.txt,u2-c.txt' -and (RuleTo) -eq 'U2') "U2: $(FRect 'U2'); has: $(In 'U2'); rule to: $(RuleTo)"

Case 'U2 Ctrl+Z after a title drag'
# Without a way back (before the fix) the fence is made again for the next cases.
if ((FRect 'U2') -eq 'gone') { Log 'U2 gone: made again'; Setup; Pf fence set U2 sort name | Out-Null }
$before = FRect 'U2'
$w = (PfFence 'U2').windowRect
DragTo ($w.x + 150) ($w.y + 14) ($w.x + 450) ($w.y + 180) 20
Start-Sleep -Milliseconds 800
$moved = FRect 'U2'
FocusTitle 'U2'
Undo
Check 'U2 Ctrl+Z puts a dragged fence back' ($moved -ne $before -and (FRect 'U2') -eq $before) "before $before, dragged $moved, after Ctrl+Z $(FRect 'U2')"

Case 'U2 Ctrl+Z after a resize'
$before = FRect 'U2'
$w = (PfFence 'U2').windowRect
DragTo ($w.x + $w.w - 2) ($w.y + 120) ($w.x + $w.w + 90) ($w.y + 120) 15
Start-Sleep -Milliseconds 800
$sized = FRect 'U2'
FocusTitle 'U2'
Undo
Check 'U2 Ctrl+Z undoes a resize' ($sized -ne $before -and (FRect 'U2') -eq $before) "before $before, resized $sized, after Ctrl+Z $(FRect 'U2')"

Case 'U2 Ctrl+Z after an icon dragged into another fence'
$p = ItemPt 'U2' 'u2-a'
$t = (PfFence 'Folders').windowRect
if ($p) { DragTo $p[0] $p[1] ($t.x + 100) ($t.y + 90) 20 }
Start-Sleep -Seconds 1
$after = In 'Folders'
FocusTitle 'U2'
Undo
Check 'U2 Ctrl+Z puts an icon dragged into another fence back' ($after -match 'u2-a' -and (In 'U2') -eq 'u2-a.txt,u2-b.txt,u2-c.txt') "Folders after the drag: $after; U2 after Ctrl+Z: $(In 'U2')"

Case 'U2 Ctrl+Z after a menu delete'
CloseNotice | Out-Null
MenuDelete 'U2'
$gone = FRect 'U2'
CloseNotice | Out-Null
FocusTitle 'Programs'
Undo
Shot 'U2-after-ctrlz-delete'
Check 'U2 Ctrl+Z on another fence brings a deleted fence back' ($gone -eq 'gone' -and (In 'U2') -eq 'u2-a.txt,u2-b.txt,u2-c.txt' -and (RuleTo) -eq 'U2') "after delete: $gone; after Ctrl+Z: $(FRect 'U2') has $(In 'U2'); rule to: $(RuleTo)"
Undo
Check 'U2 Ctrl+Z with nothing left to undo changes nothing' ((In 'U2') -eq 'u2-a.txt,u2-b.txt,u2-c.txt') "U2: $(FRect 'U2') has $(In 'U2')"
$lines = @(PfLogAll | Where-Object { $_ -match 'undo step noted|undone|nothing to undo|deleted fence put back|undo notice' } | ForEach-Object { $_.Substring([math]::Min(28, $_.Length)) })
Log "undo log ($($lines.Count)): $($lines -join ' / ')"
PfQuit
Get-ChildItem $PfDesk -Filter 'u2-*' -ErrorAction SilentlyContinue | Remove-Item -Force

# ---- C5 Desktop folder moved while running
# the known folder changes, then the files move), on the C: drive; moved back at the end.
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class KF {
  [DllImport("shell32.dll")] static extern int SHSetKnownFolderPath(ref Guid id, uint flags, IntPtr token, [MarshalAs(UnmanagedType.LPWStr)] string path);
  [DllImport("shell32.dll")] static extern void SHChangeNotify(int e, uint f, IntPtr a, IntPtr b);
  public static int SetDesktop(string path) {
    var id = new Guid("B4BFCC3A-DB2C-424C-B029-7FE99A87C641");
    int hr = SHSetKnownFolderPath(ref id, 0, IntPtr.Zero, path);
    SHChangeNotify(0x08000000, 0, IntPtr.Zero, IntPtr.Zero);
    return hr;
  }
}
'@
$old = "$env:USERPROFILE\Desktop"
$new = "$env:USERPROFILE\DesktopMoved"
function Where5() { return (@(PfJson item list) | Where-Object { $_ -and $_.fileName -like 'c5-*' } | Sort-Object fileName | ForEach-Object { "$($_.fileName)=$($_.fenceTitle)" }) -join ' ' }
function Dupes() { return (@(PfJson item list) | Where-Object { $_ } | Group-Object fileName | Where-Object { $_.Count -gt 1 } | ForEach-Object { $_.Name }) -join ',' }
function MoveAll($from, $to) { foreach ($e in @(Get-ChildItem -LiteralPath $from -Force)) { if ($e.Name -ne 'desktop.ini') { Move-Item -LiteralPath $e.FullName -Destination $to -Force } } }

Case 'C5 setup'
Get-ChildItem $old -Filter 'c5-*' -ErrorAction SilentlyContinue | Remove-Item -Force
PfStart
Pf fence create --title C5 --rect '300,200,330,230' | Out-Null
foreach ($n in 'a', 'b', 'c') { Set-Content "$old\c5-$n.txt" 'c5' }
Set-Content "$old\c5-rule.txt" 'c5'
Start-Sleep -Seconds 2
Pf item move "$old\c5-a.txt" "$old\c5-b.txt" "$old\c5-c.txt" --to C5 | Out-Null
Start-Sleep -Seconds 1
$before = Where5
Log "before: $before"
Check 'setup: three icons in C5 by hand, one placed by a rule' ($before -match 'c5-a.txt=C5' -and $before -match 'c5-rule.txt=Files') $before

Case 'C5 Desktop folder moved while running'
New-Item -ItemType Directory -Force $new | Out-Null
$hr = [KF]::SetDesktop($new)
Start-Sleep -Milliseconds 500
MoveAll $old $new
Start-Sleep -Seconds 8
$after = Where5
$d = Dupes
Shot 'C5-after-move'
Log "after the move (hr=$hr): $after; dupes: $d"
Check 'C5 after the move the hand-placed icons stay in C5' ($after -eq $before) "before: $before | after: $after | dupes: $d | known folder now $([Environment]::GetFolderPath('Desktop'))"
Check 'C5 no icon twice after the move' (-not $d) "dupes: $d"
PfQuit
PfStart
Start-Sleep -Seconds 2
$restart = Where5
$d2 = Dupes
Check 'C5 after a restart they are still in C5, no duplicates' ($restart -eq $before -and -not $d2) "before: $before | after restart: $restart | dupes: $d2"
$lines = @(PfLogAll | Where-Object { $_ -match 'desktop folder moved|records migrated' } | ForEach-Object { $_.Substring([math]::Min(28, $_.Length)) })
Log "migration log: $($lines -join ' / ')"
PfQuit

Case 'C5 cleanup: the Desktop goes back'
$hr = [KF]::SetDesktop($old)
Start-Sleep -Milliseconds 500
MoveAll $new $old
Remove-Item -LiteralPath "$new\desktop.ini" -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $new -Force -Recurse -ErrorAction SilentlyContinue
Get-ChildItem $old -Filter 'c5-*' -ErrorAction SilentlyContinue | Remove-Item -Force
Check 'cleanup: Desktop back at its place' ([Environment]::GetFolderPath('Desktop') -eq $old -and -not (Test-Path $new)) "desktop=$([Environment]::GetFolderPath('Desktop')) hr=$hr leftover=$(Test-Path $new)"
