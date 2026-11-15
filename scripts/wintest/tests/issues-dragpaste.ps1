# Issues round 2026-11-15, package dragpaste.
# Part A (feedback #162): in a fence sorted 手动, an icon dragged over a program's shortcut or a
#   folder and let go beside its icon or on its name goes to that place in the order. Only the
#   icon itself opens the dragged file with the program (#198) or takes it into the folder.
# Part B (GitHub #232, feedback #169): files copied somewhere else paste into a fence: a file
#   copied in File Explorer (it worked), a file copied inside a .zip (a virtual file, no file
#   path on the clipboard), each by Ctrl+V and by the fence menu's 粘贴, and Ctrl+V on an empty
#   spot of the desktop.
# Parts: run.sh --var Part=A or --var Part=B (about 2 min each); default AB.
if (-not $Part) { $Part = 'AB' }
trap { PfTrap $_; continue }

$Ip = "$PfRun\ip"
$Recv = "$Ip\recv.txt"
$Cap = "$Ip\bin\cap.exe"

function IpClean() {
  Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue | Where-Object { $_.Name -like 'ip *' } |
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
}
# A program that writes down what it was started with (one arg= line per argument, then --).
function MakeCap($path) {
  $cs = @"
using System; using System.IO; using System.Text;
public static class IpCap {
  public static void Main(string[] args) {
    var sb = new StringBuilder();
    foreach (var a in args) sb.Append("arg=" + a + "\n");
    File.AppendAllText(@"$Recv", sb.ToString() + "--\n", new UTF8Encoding(false));
  }
}
"@
  Add-Type -TypeDefinition $cs -OutputAssembly $path -OutputType WindowsApplication
}
# WScript.Shell cannot save a shortcut with a Chinese name: made under an ASCII name, then renamed.
function NewLink($path, $target) {
  $tmp = Join-Path (Split-Path $path) "iptmp$([guid]::NewGuid().ToString('N').Substring(0, 6)).lnk"
  $s = (New-Object -ComObject WScript.Shell).CreateShortcut($tmp); $s.TargetPath = $target; $s.Save()
  Rename-Item -LiteralPath $tmp -NewName (Split-Path -Leaf $path)
}
function ExOpen($path, $x, $y, $w, $h) {
  Start-Process explorer.exe "`"$path`""
  for ($i = 0; $i -lt 60; $i++) {
    foreach ($win in @((New-Object -ComObject Shell.Application).Windows())) {
      $p = try { $win.Document.Folder.Self.Path } catch { '' }
      if ($p -eq $path) {
        $hw = [IntPtr]$win.HWND
        [L]::MoveWindow($hw, $x, $y, $w, $h, $true) | Out-Null
        Start-Sleep -Milliseconds 1200
        return $hw
      }
    }
    Start-Sleep -Milliseconds 250
  }
  Log "ExOpen: no window for $path"
  return [IntPtr]::Zero
}
function ExItem($hw, $leaf) {
  $root = [Windows.Automation.AutomationElement]::FromHandle($hw)
  $isItem = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::ListItem)
  for ($k = 0; $k -lt 20; $k++) {
    foreach ($n in @($leaf, [IO.Path]::GetFileNameWithoutExtension($leaf))) {
      $byName = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::NameProperty, $n)
      $el = $root.FindFirst([Windows.Automation.TreeScope]::Descendants, (New-Object Windows.Automation.AndCondition($isItem, $byName)))
      if ($el) { return $el }
    }
    Start-Sleep -Milliseconds 250
  }
  Log "ExItem: '$leaf' not in the window"
  return $null
}
function ExPt($el) { $r = $el.Current.BoundingRectangle; return @([int]($r.Left + 40), [int](($r.Top + $r.Bottom) / 2)) }
# How many times the program ran since the last ClearRecv, waiting up to $sec for a first run.
function Runs($sec = 3) {
  for ($i = 0; $i -lt $sec * 5; $i++) { if (Test-Path -LiteralPath $Recv) { break }; Start-Sleep -Milliseconds 200 }
  if (-not (Test-Path -LiteralPath $Recv)) { return 0 }
  Start-Sleep -Milliseconds 300
  return @(Get-Content -LiteralPath $Recv -Encoding UTF8 | Where-Object { $_ -eq '--' }).Count
}
function ClearRecv() { Remove-Item -LiteralPath $Recv -Force -ErrorAction SilentlyContinue }
function Order($t) { return ((@(PfJson item list --fence $t) | Where-Object { $_ -and $_.name } | ForEach-Object { $_.name }) -join ' | ') }
function Names($t) { return @((Order $t) -split ' \| ') }
function IndexIn($t, $name) { $k = 0; foreach ($n in @(Names $t)) { if ("$n" -eq "$name") { return $k }; $k++ }; return -1 }
# Points of an icon by its place in the fence's order: the icon's middle (CellPt), the room left
# of the icon (6 px in from the cell's left edge) and its name.
function IconPt($t, $name) { $i = IndexIn $t $name; if ($i -lt 0) { Log "IconPt: '$name' not in '$t' [$(Order $t)]"; return $null }; return (CellPt $t $i) }
function Pitch($t) { $f = FenceGet $t; $cols = if ($f.fit -and $f.fit.columns) { [int]$f.fit.columns } else { [math]::Max(1, [math]::Floor($f.windowRect.w / 80)) }; return ($f.windowRect.w / $cols) }
function BesidePt($t, $name) { $p = IconPt $t $name; if (-not $p) { return $null }; return @([int]($p[0] - (Pitch $t) / 2 + 6), $p[1]) }
function NamePt($t, $name) { $p = IconPt $t $name; if (-not $p) { return $null }; return @($p[0], ($p[1] + 34)) }
function IpDrag($from, $to, $shot = '') {
  [G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 150
  MDown; Start-Sleep -Milliseconds 200
  Glide $from[0] $from[1] ($from[0] + 12) ($from[1] + 12) 3 30
  Glide ($from[0] + 12) ($from[1] + 12) $to[0] $to[1] 16 35
  Start-Sleep -Milliseconds 500
  [G]::MoveTo($to[0] + 1, $to[1]); Start-Sleep -Milliseconds 500
  if ($shot) { Shot $shot }
  MUp; Start-Sleep -Milliseconds 1500
}

if ($Part -match 'A') {
  Case 'A setup'
  CloseWindows
  IpClean
  Remove-Item -Recurse -Force $Ip -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force "$Ip\bin" | Out-Null
  MakeCap $Cap
  # The order the fence keeps: program, folder, program, file, file, program.
  $Seq = @('ip 甲 程序.lnk', 'ip 丙 资料夹', 'ip 乙 程序.lnk', 'ip 丁 报告.txt', 'ip 己 预算表 2026.txt', 'ip 戊 程序.lnk')
  NewLink "$PfDesk\ip 甲 程序.lnk" $Cap
  NewLink "$PfDesk\ip 乙 程序.lnk" $Cap
  NewLink "$PfDesk\ip 戊 程序.lnk" $Cap
  New-Item -ItemType Directory -Force "$PfDesk\ip 丙 资料夹" | Out-Null
  Set-Content -LiteralPath "$PfDesk\ip 丁 报告.txt" 'a report' -Encoding UTF8
  Set-Content -LiteralPath "$PfDesk\ip 己 预算表 2026.txt" 'a budget' -Encoding UTF8
  PfStart -Lang 'zh-CN'
  Start-Sleep -Seconds 2
  ClearFences | Out-Null
  ClearPfNotices
  # Three rows: the icon of another fence (A4) makes seven.
  $r = FreeRect 240 360 300 120
  Pf fence create --title Apps --rect "$($r[0]),$($r[1]),240,360" | Out-Null
  $r2 = FreeRect 240 160 700 120
  Pf fence create --title Bravo --rect "$($r2[0]),$($r2[1]),240,160" | Out-Null
  Pf fence set Apps sort manual | Out-Null
  Start-Sleep -Milliseconds 600
  foreach ($n in $Seq) { Pf item move (Join-Path $PfDesk $n) --to Apps | Out-Null; Start-Sleep -Milliseconds 300 }
  Set-Content -LiteralPath "$PfDesk\ip 庚 清单.txt" 'a list' -Encoding UTF8
  Start-Sleep -Seconds 1
  Pf item move "$PfDesk\ip 庚 清单.txt" --to Bravo | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse
  # The names the fence shows (no .lnk; .txt shows with the box's default "hide extensions").
  $script:A = @{}; foreach ($n in (Names 'Apps')) { $script:A[$n.Substring(0, 4)] = $n }
  Log "Apps [$(Order 'Apps')] Bravo [$(Order 'Bravo')]"
  Check 'A setup: six icons in Apps in the order they came' ((Names 'Apps').Count -eq 6 -and (IndexIn 'Apps' $script:A['ip 甲']) -eq 0 -and (IndexIn 'Apps' $script:A['ip 戊']) -eq 5) "[$(Order 'Apps')]"
  Shot 'A0-start'

  Case 'A1 let go beside a program''s icon: the icon goes there, the program does not run'
  ClearRecv
  IpDrag (IconPt 'Apps' $A['ip 己']) (BesidePt 'Apps' $A['ip 甲']) 'A1-hold'
  $runs = Runs 3
  Check 'A1 the program did not run' ($runs -eq 0) "runs $runs"
  Check 'A1 the icon is first now' ((IndexIn 'Apps' $A['ip 己']) -eq 0) "[$(Order 'Apps')]"
  Shot 'A1-after'

  Case 'A2 let go on a program''s name: the icon goes beside it'
  ClearRecv
  $before = IndexIn 'Apps' $A['ip 乙']
  IpDrag (IconPt 'Apps' $A['ip 丁']) (NamePt 'Apps' $A['ip 乙']) 'A2-hold'
  $runs = Runs 3
  $at = IndexIn 'Apps' $A['ip 丁']; $p = IndexIn 'Apps' $A['ip 乙']
  Check 'A2 the program did not run' ($runs -eq 0) "runs $runs"
  Check 'A2 the icon stands next to that program now' ([math]::Abs($at - $p) -eq 1) "[$(Order 'Apps')]"

  Case 'A3 let go beside a folder''s icon: the icon goes there, not into the folder'
  IpDrag (IconPt 'Apps' $A['ip 丁']) (BesidePt 'Apps' $A['ip 丙']) 'A3-hold'
  $inFolder = Test-Path -LiteralPath "$PfDesk\ip 丙 资料夹\ip 丁 报告.txt"
  Check 'A3 the file is still on the desktop' (-not $inFolder -and (Test-Path -LiteralPath "$PfDesk\ip 丁 报告.txt")) "in the folder: $inFolder"
  Check 'A3 the icon stands just before the folder' ((IndexIn 'Apps' $A['ip 丁']) -eq (IndexIn 'Apps' $A['ip 丙']) - 1) "[$(Order 'Apps')]"

  Case 'A4 an icon of another fence let go beside a program''s icon lands there'
  ClearRecv
  $g = @(Names 'Bravo')[0]
  IpDrag (IconPt 'Bravo' $g) (BesidePt 'Apps' $A['ip 乙'])
  $runs = Runs 3
  Check 'A4 the program did not run' ($runs -eq 0) "runs $runs"
  Check 'A4 the icon stands just before that program in Apps' ((IndexIn 'Apps' $g) -ge 0 -and (IndexIn 'Apps' $g) -eq (IndexIn 'Apps' $A['ip 乙']) - 1) "[$(Order 'Apps')]"

  Case 'A5 let go on the middle of a program''s icon: it opens the file (#198)'
  ClearRecv
  $order0 = Order 'Apps'
  IpDrag (IconPt 'Apps' $A['ip 己']) (IconPt 'Apps' $A['ip 戊']) 'A5-hold'
  $runs = Runs 6
  $got = @(Get-Content -LiteralPath $Recv -Encoding UTF8 -ErrorAction SilentlyContinue | Where-Object { $_ -like 'arg=*' })
  Check 'A5 the program ran once with the file' ($runs -eq 1 -and ($got -join ' ') -match '预算表 2026\.txt') "runs $runs args $($got -join ' ')"
  Check 'A5 the order did not change' ((Order 'Apps') -eq $order0) "[$(Order 'Apps')] was [$order0]"

  Case 'A6 let go on the middle of a folder''s icon: the file goes into the folder'
  IpDrag (IconPt 'Apps' $A['ip 丁']) (IconPt 'Apps' $A['ip 丙']) 'A6-hold'
  Start-Sleep -Seconds 1
  $inFolder = Test-Path -LiteralPath "$PfDesk\ip 丙 资料夹\ip 丁 报告.txt"
  Check 'A6 the file is in the folder' $inFolder "desktop: $(Test-Path -LiteralPath "$PfDesk\ip 丁 报告.txt")"
  Get-Process IpCap, cap -ErrorAction SilentlyContinue | Stop-Process -Force
}

if ($Part -match 'B') {
  Case 'B setup'
  CloseWindows
  IpClean
  $Src = "$Ip\src"
  Remove-Item -Recurse -Force $Src -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force "$Src\zipped" | Out-Null
  Set-Content -LiteralPath "$Src\ip 外来 报告.txt" 'from elsewhere' -Encoding UTF8
  Set-Content -LiteralPath "$Src\zipped\ip 压缩内 清单.txt" 'from a zip' -Encoding UTF8
  Compress-Archive -Path "$Src\zipped\*" -DestinationPath "$Src\ip 压缩包.zip" -Force
  Remove-Item -Recurse -Force "$Src\zipped"
  PfStart -Lang 'zh-CN'
  Start-Sleep -Seconds 2
  ClearFences | Out-Null
  ClearPfNotices
  $r = FreeRect 300 260 120 120
  Pf fence create --title Bravo --rect "$($r[0]),$($r[1]),300,260" | Out-Null
  Start-Sleep -Seconds 1
  $script:Inbox = (@(PfJson fence list) | Where-Object { $_ -and $_.kind -eq 'inbox' } | Select-Object -First 1).title
  Log "fences: Bravo, inbox '$Inbox'"
  # The Explorer windows stand on the right half, one above the other, away from Bravo and the
  # free desktop spot.
  $script:ExPlain = ExOpen $Src 820 20 760 400
  $script:ExZip = ExOpen "$Src\ip 压缩包.zip" 820 440 760 400
  Log "Explorer windows: $(Explorers)"

  function ClipFormats() { try { return ([Windows.Forms.Clipboard]::GetDataObject().GetFormats() -join ', ') } catch { return "error $_" } }
  # Selects one item in an Explorer window and presses Ctrl+C there.
  function CopyIn($hw, $leaf) {
    [Windows.Forms.Clipboard]::Clear()
    $el = ExItem $hw $leaf
    if (-not $el) { return $false }
    $p = ExPt $el
    [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 400
    Chord 0x11 0x43; Start-Sleep -Milliseconds 800
    Log "copied '$leaf': clipboard $(ClipFormats)"
    return $true
  }
  function FileAt($leaf, $sec = 6) {
    for ($i = 0; $i -lt $sec * 4; $i++) { if (Test-Path -LiteralPath (Join-Path $PfDesk $leaf)) { return $true }; Start-Sleep -Milliseconds 250 }
    return $false
  }
  function DropPasted($leaf) { Remove-Item -LiteralPath (Join-Path $PfDesk $leaf) -Force -ErrorAction SilentlyContinue; Start-Sleep -Milliseconds 800 }
  function ClickBravo() { $w = (FenceGet 'Bravo').windowRect; [G]::Click($w.x + $w.w - 30, $w.y + $w.h - 24); Start-Sleep -Milliseconds 500 }
  function BravoMenu() { $w = (FenceGet 'Bravo').windowRect; [G]::RClick($w.x + $w.w - 30, $w.y + $w.h - 24); return (MenuWin 5) }
  $Plain = 'ip 外来 报告.txt'; $Zipped = 'ip 压缩内 清单.txt'

  Case 'B1 a file copied in File Explorer, Ctrl+V in a fence'
  $ok = CopyIn $ExPlain $Plain
  ClickBravo; Chord 0x11 0x56
  $there = FileAt $Plain
  Check 'B1 the file is on the desktop' ($ok -and $there) "copied $ok"
  Check 'B1 it is in the fence pasted into' ((WaitItemFence $Plain 'Bravo' 6) -eq 'Bravo') "in '$(ItemFence $Plain)'"
  DropPasted $Plain

  Case 'B2 a file copied inside a .zip, Ctrl+V in a fence'
  $ok = CopyIn $ExZip $Zipped
  ClickBravo; Chord 0x11 0x56
  $there = FileAt $Zipped 8
  Shot 'B2-after'
  Check 'B2 the file is on the desktop' ($ok -and $there) "copied $ok; log: $((@(PfLogAll | Where-Object { $_ -match 'paste|clipboard' } | Select-Object -Last 3)) -join ' || ')"
  Check 'B2 it is in the fence pasted into' ((WaitItemFence $Zipped 'Bravo' 6) -eq 'Bravo') "in '$(ItemFence $Zipped)'"
  DropPasted $Zipped

  Case 'B3 a file copied inside a .zip, the fence menu''s 粘贴'
  $ok = CopyIn $ExZip $Zipped
  $m = BravoMenu
  $texts = MenuTexts $m
  Log "B3 menu: $texts"
  Check 'B3 the menu''s 粘贴 can be picked' ($texts -match '(^|\| )粘贴(\(P\))?( \||$)' -and $texts -notmatch '粘贴(\(P\))? \(off\)') $texts
  MenuPick '^粘贴' $m | Out-Null
  $there = FileAt $Zipped 8
  Check 'B3 the file is on the desktop' $there ''
  Check 'B3 it is in the fence pasted into' ((WaitItemFence $Zipped 'Bravo' 6) -eq 'Bravo') "in '$(ItemFence $Zipped)'"
  DropPasted $Zipped

  Case 'B4 a file copied in File Explorer, Ctrl+V on an empty spot of the desktop'
  $ok = CopyIn $ExPlain $Plain
  $p = DeskClick 500 700
  Log "B4 clicked the desktop at $($p -join ','); foreground $(Fg)"
  Chord 0x11 0x56
  $there = FileAt $Plain
  Shot 'B4-after'
  Check 'B4 the file is on the desktop' ($ok -and $there) "foreground $(Fg)"
  Log "B4 the pasted file is in '$(ItemFence $Plain)'"
  DropPasted $Plain

  Case 'B5 a file copied inside a .zip, Ctrl+V on an empty spot of the desktop'
  $ok = CopyIn $ExZip $Zipped
  $p = DeskClick 500 700
  Chord 0x11 0x56
  $there = FileAt $Zipped 8
  Check 'B5 the file is on the desktop' ($ok -and $there) "foreground $(Fg)"
  DropPasted $Zipped
  CloseWindows
}

Case 'cleanup'
IpClean
CloseWindows
