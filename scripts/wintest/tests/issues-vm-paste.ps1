# Issues round 2026-11-20, paste reports.
# Part V (GitHub #241 part 2): files copied in another machine's window (a remote desktop or a
#   Hyper-V enhanced session) paste on the desktop with Ctrl+V. The clipboard then holds
#   Windows' remote clipboard (rdpclip): file names (FileGroupDescriptorW) and contents read
#   from the other machine only when a paste asks, no file paths. Here the box itself is the
#   remote side: the job asks for it by writing C:\pf\clip-want.txt, tests/free-placement.sh
#   puts a file on the RDP client's clipboard on the dev box, and rdpclip offers it in pf's
#   session. Run it through that script:
#     tests/free-placement.sh --box ec2-2 --part V --steps issues-vm-paste.ps1 [--bin ...]
#   (with plain run.sh the clipboard must be set by hand first, see free-placement.sh).
# Part P (feedback #183): a file cut or copied in a fence while Peek shows the fences, Peek ended
#   by a click on its dim, pastes into a File Explorer window; the same with the item menu's
#   Cut (剪切) without Peek.
# Parts: --var Part=V or P (about 1 min each); default VP.
if (-not $Part) { $Part = 'VP' }
trap { PfTrap $_; continue }

$Vp = "$PfRun\vp"
# The file the dev box puts on the RDP client's clipboard (tests/free-placement.sh).
$Remote = 'ip vm 远程文件.txt'

Add-Type @'
using System; using System.Text; using System.Runtime.InteropServices; using System.Collections.Generic;
public static class VpClip {
  [DllImport("user32.dll")] static extern bool OpenClipboard(IntPtr h);
  [DllImport("user32.dll")] static extern bool CloseClipboard();
  [DllImport("user32.dll")] static extern uint EnumClipboardFormats(uint f);
  [DllImport("user32.dll")] static extern IntPtr GetClipboardOwner();
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClipboardFormatNameW(uint f, StringBuilder s, int n);
  public static string Formats() {
    if (!OpenClipboard(IntPtr.Zero)) return "(clipboard busy)";
    var names = new List<string>();
    try {
      for (uint f = EnumClipboardFormats(0); f != 0; f = EnumClipboardFormats(f)) {
        var sb = new StringBuilder(128);
        names.Add(GetClipboardFormatNameW(f, sb, 128) > 0 ? sb.ToString() : ("#" + f));
      }
    } finally { CloseClipboard(); }
    return names.Count == 0 ? "(empty)" : string.Join(", ", names.ToArray());
  }
  public static string Owner() {
    uint pid; GetWindowThreadProcessId(GetClipboardOwner(), out pid);
    try { return System.Diagnostics.Process.GetProcessById((int)pid).ProcessName; } catch { return "pid " + pid; }
  }
}
'@

function VpFileAt($leaf, $sec = 10) {
  for ($i = 0; $i -lt $sec * 4; $i++) { if (Test-Path -LiteralPath (Join-Path $PfDesk $leaf)) { return $true }; Start-Sleep -Milliseconds 250 }
  return $false
}
function VpDrop($leaf) { Remove-Item -LiteralPath (Join-Path $PfDesk $leaf) -Force -ErrorAction SilentlyContinue; Start-Sleep -Milliseconds 800 }
function VpClickBravo() { $w = (FenceGet 'Bravo').windowRect; [G]::Click($w.x + $w.w - 30, $w.y + $w.h - 24); Start-Sleep -Milliseconds 500 }
function VpPasteLog() { return ((@(PfLogAll | Where-Object { $_ -match 'paste|Ctrl\+V|clipboard' } | Select-Object -Last 3)) -join ' || ') }
function VpSetup() {
  trap { PfTrap $_; continue }
  CloseWindows
  Remove-Item -Recurse -Force $Vp -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force "$Vp\dest" | Out-Null
  Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue | Where-Object { $_.Name -like 'ip *' } | Remove-Item -Force -ErrorAction SilentlyContinue
  PfStart -Lang 'zh-CN'
  Start-Sleep -Seconds 2
  ClearFences | Out-Null
  $r = FreeRect 300 260 120 120
  Pf fence create --title Bravo --rect "$($r[0]),$($r[1]),300,260" | Out-Null
  Start-Sleep -Seconds 1
  $script:Inbox = (@(PfJson fence list) | Where-Object { $_ -and $_.kind -eq 'inbox' } | Select-Object -First 1).title
  Log "fences: Bravo, inbox '$Inbox'"
}

function PartV() {
  trap { PfTrap $_; continue }
  Case 'V setup'
  VpSetup
  # Ask the dev box for the remote clipboard and wait until rdpclip offers the file.
  Set-Content -LiteralPath 'C:\pf\clip-want.txt' 'files' -Encoding ASCII
  $f = ''
  for ($i = 0; $i -lt 60; $i++) {
    $f = [VpClip]::Formats()
    if ($f -match 'FileGroupDescriptorW') { break }
    Start-Sleep -Milliseconds 500
  }
  Remove-Item -LiteralPath 'C:\pf\clip-want.txt' -Force -ErrorAction SilentlyContinue
  $owner = [VpClip]::Owner()
  Log "V clipboard: $f; owner $owner"
  Check 'V the clipboard holds the remote file (rdpclip)' (($f -match 'FileGroupDescriptorW') -and ($f -notmatch 'FileNameW') -and $owner -eq 'rdpclip') "formats: $f; owner $owner"

  Case 'V1 a file copied on the other machine, Ctrl+V on an empty spot of the desktop'
  $p = DeskClick 500 700
  Log "V1 clicked the desktop at $($p -join ','); foreground $(Fg)"
  Chord 0x11 0x56
  $there = VpFileAt $Remote 12
  Shot 'V1-after'
  Check 'V1 the file is on the desktop' $there "foreground $(Fg); log: $(VpPasteLog)"
  Check 'V1 it is in 「桌面」' ((WaitItemFence $Remote $Inbox 6) -eq $Inbox) "in '$(ItemFence $Remote)'"
  $text = if ($there) { (Get-Content -LiteralPath (Join-Path $PfDesk $Remote) -Raw -Encoding UTF8) } else { '' }
  Check 'V1 it has the other machine''s contents' ("$text" -match 'from the other machine') "contents '$text'"
  VpDrop $Remote

  Case 'V2 a file copied on the other machine, Ctrl+V in a fence'
  VpClickBravo
  Chord 0x11 0x56
  $there = VpFileAt $Remote 12
  Check 'V2 the file is on the desktop' $there "log: $(VpPasteLog)"
  Check 'V2 it is in the fence pasted into' ((WaitItemFence $Remote 'Bravo' 6) -eq 'Bravo') "in '$(ItemFence $Remote)'"
  VpDrop $Remote

  Case 'V3 a file copied on the other machine, the fence menu''s 粘贴'
  $w = (FenceGet 'Bravo').windowRect
  [G]::RClick($w.x + $w.w - 30, $w.y + $w.h - 24)
  $m = MenuWin 5
  $texts = MenuTexts $m
  Log "V3 menu: $texts"
  $on = $texts -match '粘贴' -and $texts -notmatch '粘贴[^|]*\(off\)'
  Check 'V3 the menu''s 粘贴 can be picked' $on $texts
  if ($on) { MenuPick '^粘贴' $m | Out-Null } else { CloseMenus }
  $there = VpFileAt $Remote 12
  Check 'V3 the file is on the desktop' $there "log: $(VpPasteLog)"
  Check 'V3 it is in the fence pasted into' ((WaitItemFence $Remote 'Bravo' 6) -eq 'Bravo') "in '$(ItemFence $Remote)'"
  VpDrop $Remote
}

# A File Explorer window on $path at the given place; its handle.
function VpExplorer($path, $x, $y, $w, $h) {
  Start-Process explorer.exe "`"$path`""
  for ($i = 0; $i -lt 60; $i++) {
    foreach ($win in @((New-Object -ComObject Shell.Application).Windows())) {
      $p = try { $win.Document.Folder.Self.Path } catch { '' }
      if ($p -eq $path) { $hw = [IntPtr]$win.HWND; [L]::MoveWindow($hw, $x, $y, $w, $h, $true) | Out-Null; Start-Sleep -Milliseconds 1200; return $hw }
    }
    Start-Sleep -Milliseconds 250
  }
  Log "VpExplorer: no window for $path"
  return [IntPtr]::Zero
}
# Ctrl+V into the Explorer window: a click on the empty part of its list first.
function VpPasteInExplorer($hw) {
  $wr = Rect $hw
  [G]::Click($wr.L + 300, $wr.B - 60); Start-Sleep -Milliseconds 600
  Log "paste in Explorer: foreground $(Fg)"
  Chord 0x11 0x56
}
function VpInDest($leaf, $sec = 8) {
  for ($i = 0; $i -lt $sec * 4; $i++) { if (Test-Path -LiteralPath (Join-Path "$Vp\dest" $leaf)) { return $true }; Start-Sleep -Milliseconds 250 }
  return $false
}

function PartP() {
  trap { PfTrap $_; continue }
  Case 'P setup'
  VpSetup
  $script:CutA = 'ip 剪切 甲.txt'; $script:CopyB = 'ip 复制 乙.txt'; $script:CutC = 'ip 菜单剪切 丙.txt'
  foreach ($n in $CutA, $CopyB, $CutC) { Set-Content -LiteralPath (Join-Path $PfDesk $n) "file $n" -Encoding UTF8 }
  Start-Sleep -Seconds 3
  foreach ($n in $CutA, $CopyB, $CutC) { Pf item move (Join-Path $PfDesk $n) --to Bravo | Out-Null }
  Start-Sleep -Seconds 1
  Log "Bravo: $((@(PfJson item list --fence Bravo) | Where-Object { $_ -and $_.name } | ForEach-Object { $_.name }) -join ' | ')"
  # The Explorer window stands on the right half, away from Bravo; a spot of bare desktop for
  # the click on Peek's dim, taken before Peek covers the desktop.
  $script:Ex = VpExplorer "$Vp\dest" 820 300 700 420
  $script:Dim = FreeSpot 500 760

  Case 'P1 Ctrl+X in a fence while Peek shows it, Peek ended by a click on the dim, Ctrl+V in Explorer'
  PeekKey; Start-Sleep -Milliseconds 900
  $p = ItemPt 'Bravo' ([IO.Path]::GetFileNameWithoutExtension($CutA))
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 400
  Chord 0x11 0x58; Start-Sleep -Milliseconds 600
  $files = @(ClipFiles); $eff = ClipEffect
  Log "P1 clipboard: $($files -join ', ') effect $eff"
  [G]::Click($Dim[0], $Dim[1]); Start-Sleep -Milliseconds 900
  VpPasteInExplorer $Ex
  $moved = VpInDest $CutA
  Start-Sleep -Milliseconds 800
  $left = Test-Path -LiteralPath (Join-Path $PfDesk $CutA)
  Shot 'P1-after'
  Check 'P1 the cut file is in the Explorer folder' $moved "clipboard $($files -join ', ') effect $eff"
  Check 'P1 it left the desktop' (-not $left) "still on the desktop $left"

  Case 'P2 Ctrl+C in a fence while Peek shows it, Peek ended by a click on the dim, Ctrl+V in Explorer'
  PeekKey; Start-Sleep -Milliseconds 900
  $p = ItemPt 'Bravo' ([IO.Path]::GetFileNameWithoutExtension($CopyB))
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 400
  Chord 0x11 0x43; Start-Sleep -Milliseconds 600
  $files = @(ClipFiles); $eff = ClipEffect
  Log "P2 clipboard: $($files -join ', ') effect $eff"
  [G]::Click($Dim[0], $Dim[1]); Start-Sleep -Milliseconds 900
  VpPasteInExplorer $Ex
  $copied = VpInDest $CopyB
  Check 'P2 the copy is in the Explorer folder' $copied "clipboard $($files -join ', ') effect $eff"
  Check 'P2 the file stays on the desktop' (Test-Path -LiteralPath (Join-Path $PfDesk $CopyB)) ''

  Case 'P3 the item menu''s Cut, Ctrl+V in Explorer'
  $m = RClickItem 'Bravo' ([IO.Path]::GetFileNameWithoutExtension($CutC))
  $texts = MenuTexts $m
  # The shell's part of the menu is in Windows' language (English on the boxes).
  $picked = MenuPick '^(剪切|Cut)' $m
  if (-not $picked) { Log "P3 menu: $texts"; CloseMenus }
  Start-Sleep -Milliseconds 600
  $files = @(ClipFiles); $eff = ClipEffect
  VpPasteInExplorer $Ex
  $moved = VpInDest $CutC
  Start-Sleep -Milliseconds 800
  Check 'P3 the cut file is in the Explorer folder' $moved "picked '$picked'; clipboard $($files -join ', ') effect $eff"
  Check 'P3 it left the desktop' (-not (Test-Path -LiteralPath (Join-Path $PfDesk $CutC))) ''
  CloseWindows
}

if ($Part -match 'V') { PartV }
if ($Part -match 'P') { PartP }

Case 'cleanup'
Remove-Item -LiteralPath 'C:\pf\clip-want.txt' -Force -ErrorAction SilentlyContinue
Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue | Where-Object { $_.Name -like 'ip *' } | Remove-Item -Force -ErrorAction SilentlyContinue
CloseWindows
