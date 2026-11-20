# Issues round 2026-11-20, Peek.
#   A  GitHub #238: a file dragged out of a fence during the Peek onto a File Explorer folder that
#      already has that name: Explorer asks 「替换或跳过文件」 while our drag still waits for its
#      answer. The dialog must take the real pointer (the Peek's dimmer covered it, so only the
#      taskbar answered), and the Peek ends. A drop without a question ends the Peek as before.
#   B  GitHub #238, second part: with 自由摆放, a file made right after a right-click on the
#      empty desktop (desktop menu 新建 / 粘贴) lands in the cell under that click, as on the
#      Windows desktop; files that come without such a click take the first free cell as before.
#   C  Feedback #189, #190: 速览 › 快捷键 › 「自定义…」 picked with the mouse in the list opens the
#      recording box (also while every combination is taken by other programs), and a
#      combination pressed then is saved; picked with the keyboard the same. Debug build (CDP).
# Default: all parts.
if (-not $Part) { $Part = 'ABC' }
trap { PfTrap $_; continue }

Add-Type @'
using System; using System.Runtime.InteropServices;
public static class PkH {
  [DllImport("user32.dll")] public static extern bool RegisterHotKey(IntPtr h, int id, uint mods, uint vk);
  [DllImport("user32.dll")] public static extern bool UnregisterHotKey(IntPtr h, int id);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
}
'@

function PeekOn() { return (@([W]::Tops('PecoFence.PeekDimmer')).Count -gt 0) }
function ClashDialog() { return @([W]::Tops('OperationStatusWindow')) | Select-Object -First 1 }

# ---- A: a name clash in Explorer after a drag out of a peeked fence
function PartA([bool]$dim) {
  trap { PfTrap $_; continue }
  Case "A: drag out of the Peek into a name clash (dim $dim)"
  CloseWindows
  $dest = 'C:\pf\pk238'
  Remove-Item -Recurse -Force $dest -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force $dest | Out-Null
  $tag = if ($dim) { 'd' } else { 'n' }
  $clash = "pk238clash$tag.txt"; $plain = "pk238plain$tag.txt"
  Set-Content -LiteralPath (Join-Path $dest $clash) 'old'
  DeskFile $clash | Out-Null
  DeskFile $plain | Out-Null
  PfStart
  Pf settings set peek.dim ($(if ($dim) { 'true' } else { 'false' })) | Out-Null
  Start-Sleep -Seconds 3
  Start-Process explorer.exe $dest
  $ex = $null
  for ($i = 0; $i -lt 50 -and -not $ex; $i++) {
    Start-Sleep -Milliseconds 200
    $ex = @([W]::Tops('CabinetWClass')) | Select-Object -First 1
  }
  [L]::MoveWindow($ex, 20, 40, 760, 600, $true) | Out-Null
  Start-Sleep -Milliseconds 800
  $fence = ItemFence $clash
  $p = ItemPt $fence ([IO.Path]::GetFileNameWithoutExtension($clash))
  Check 'setup: Explorer open and the file in a fence' ($ex -and $p) "explorer=$ex fence='$fence' at=$($p -join ',')"
  ParkMouse
  PeekKey; Start-Sleep -Milliseconds 900
  Check 'the Peek is on' (PeekOn) ''
  Shot "a-$tag-peek"
  DragTo $p[0] $p[1] 400 380 20
  $dlg = $null
  for ($i = 0; $i -lt 30 -and -not $dlg; $i++) { Start-Sleep -Milliseconds 200; $dlg = ClashDialog }
  Start-Sleep -Milliseconds 700
  Shot "a-$tag-dialog"
  Check 'Explorer asks about the name clash' ([bool]$dlg) "dialog=$dlg"
  $skip = $null
  if ($dlg) {
    try { $skip = UiaFind $dlg 'Skip this file' } catch { Log "uia: $($_.Exception.Message)" }
  }
  $ok = $false; $why = 'no dialog'
  if ($skip) {
    $b = $skip.Current.BoundingRectangle
    $x = [int]($b.X + $b.Width / 2); $y = [int]($b.Y + $b.Height / 2)
    $under = [W]::RootClassAt($x, $y)
    [G]::Click($x, $y)
    for ($i = 0; $i -lt 20 -and (ClashDialog); $i++) { Start-Sleep -Milliseconds 200 }
    $ok = -not (ClashDialog)
    $why = "skip at $x,$y window there=$under"
  } elseif ($dlg) { $why = 'no Skip button found' }
  Check 'a real click on 「跳过此文件」 reaches the dialog during the Peek' $ok $why
  Start-Sleep -Milliseconds 800
  Check 'the Peek has ended' (-not (PeekOn)) "dimmer visible=$(PeekOn)"
  Check 'the file was skipped: both copies stay' ((Get-Content -LiteralPath (Join-Path $dest $clash)) -eq 'old' -and (Test-Path (Join-Path $PfDesk $clash))) ''
  Check 'PecoFence answers' ((PfAlive) -and ([string](Pf fence list)).Length -gt 0) ''
  # The answer was clicked over the desktop (the dialog stands where nothing else is once it
  # closes): the skipped file stays in its fence, it is not taken as dropped on the desktop.
  $now = WaitItemFence $clash $fence 3
  Check 'the skipped file stays in its fence' ($now -eq $fence) "fence now='$now' was='$fence'"
  # Left as it was found, whatever happened above.
  $d = ClashDialog
  if ($d) { [PkH]::PostMessageW($d, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; Start-Sleep -Milliseconds 600 }
  if (PeekOn) { [G]::Press(0x1B); Start-Sleep -Milliseconds 500 }

  # The plain drop (no question) ends the Peek, and the file is moved.
  $p = ItemPt (ItemFence $plain) ([IO.Path]::GetFileNameWithoutExtension($plain))
  PeekKey; Start-Sleep -Milliseconds 900
  $on = PeekOn
  DragTo $p[0] $p[1] 400 380 20
  Start-Sleep -Milliseconds 1500
  Check 'a drop without a question: moved, the Peek ended' ($on -and -not (PeekOn) -and (Test-Path (Join-Path $dest $plain)) -and -not (Test-Path (Join-Path $PfDesk $plain))) "peek was on=$on now=$(PeekOn)"
  if (PeekOn) { [G]::Press(0x1B); Start-Sleep -Milliseconds 500 }
  CloseWindows
  Remove-Item -LiteralPath (Join-Path $PfDesk $clash) -ErrorAction SilentlyContinue
  Remove-Item -Recurse -Force $dest -ErrorAction SilentlyContinue
}

# ---- B: a new file under the right-click on the empty desktop (自由摆放)
function CellOf($name) {
  $f = FenceGet $script:InboxId
  $it = @(TJson item list) | Where-Object { $_.fileName -eq $name } | Select-Object -First 1
  if (-not $f -or -not $it -or -not $f.desktopCells) { return $null }
  $c = $f.desktopCells."$($it.id)"
  if (-not $c) { return $null }
  return "$($c[0]),$($c[1])"
}
# Windows 11's desktop menu (a XAML window) closes on Esc once it has the keyboard; press it
# until no such menu is in front.
function CloseDeskMenu() {
  for ($i = 0; $i -lt 4; $i++) {
    $fg = [W]::Cls([L]::GetForegroundWindow())
    if ($fg -notlike 'Xaml*' -and $fg -ne '#32768') { break }
    [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  }
  CloseMenus
  Log "desk menu closed; in front: $([W]::Cls([L]::GetForegroundWindow()))"
}
function WaitCell($name, $sec = 8) {
  for ($i = 0; $i -lt $sec * 4; $i++) { $c = CellOf $name; if ($c) { return $c }; Start-Sleep -Milliseconds 250 }
  return $null
}
function PartB() {
  trap { PfTrap $_; continue }
  Case 'B: a new file lands under the right-click on the desktop (自由摆放)'
  CloseWindows
  PfStart
  Start-Sleep -Seconds 3
  $inbox = @(PfJson fence list) | Where-Object { $_.kind -eq 'inbox' } | Select-Object -First 1
  $script:InboxId = $inbox.id
  $f = FenceGet $inbox.id
  Check 'setup: 「桌面」 is arranged like the desktop with 自由摆放' ($f.desktopLayout -and $f.freePlacement) "desktopLayout=$($f.desktopLayout) freePlacement=$($f.freePlacement)"
  # An empty spot well inside the desktop, away from the first column and from the fences.
  $at = FreeSpot -NearX 520 -NearY 420
  if (-not $at) { $at = @(520, 420) }
  $wr = $f.windowRect
  Log "inbox window $($wr.x),$($wr.y),$($wr.w),$($wr.h); right-click at $($at -join ',')"
  # Cell of a point: 80 x 96 cells from the work area's top left, 4 px in (see ItemPt).
  $want = "$([int][Math]::Floor(($at[0] - 4) / 80)),$([int][Math]::Floor(($at[1] - 4) / 96))"
  [G]::RClick($at[0], $at[1]); Start-Sleep -Milliseconds 900
  Shot 'b-menu'
  CloseDeskMenu
  # The menu's 新建 › 文本文档 writes the file at once; the test writes it itself, then the
  # name the menu would give is not needed. A type no rule takes, so it goes to 「桌面」 (a text
  # file goes to 「文件与文档」 by the first-run rules, wherever the click was).
  $n1 = "pk238new $(Get-Date -Format HHmmss).pk238"
  Set-Content -LiteralPath (Join-Path $PfDesk $n1) 'x'
  $c1 = WaitCell $n1
  Shot 'b-new-file'
  Check 'a file made right after the right-click takes the cell under the click' ($c1 -eq $want) "cell=$c1 want=$want"
  # Later files, with no click before them: the first free cell from the top left, as before.
  Start-Sleep -Seconds 1
  $n2 = "pk238later $(Get-Date -Format HHmmss).pk238"
  Set-Content -LiteralPath (Join-Path $PfDesk $n2) 'x'
  $c2 = WaitCell $n2
  Check 'the next file without a click does not go there too' ($c2 -and $c2 -ne $want) "cell=$c2"
  # A right-click long ago does not count.
  [G]::RClick($at[0] + 160, $at[1]); Start-Sleep -Milliseconds 700
  CloseDeskMenu
  Start-Sleep -Seconds 32
  $n3 = "pk238old $(Get-Date -Format HHmmss).pk238"
  Set-Content -LiteralPath (Join-Path $PfDesk $n3) 'x'
  $c3 = WaitCell $n3
  $old = "$([int][Math]::Floor(($at[0] + 160 - 4) / 80)),$([int][Math]::Floor(($at[1] - 4) / 96))"
  Check 'a file half a minute after the click takes the first free cell' ($c3 -and $c3 -ne $old) "cell=$c3 click cell=$old"
  foreach ($n in $n1, $n2, $n3) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -ErrorAction SilentlyContinue }
}

# ---- C: 速览 › 快捷键 › 自定义…
function EditState() {
  return Js "(() => { const e = document.getElementById('peekHotkeyEdit'), b = document.getElementById('peekHotkey'); return { shown: !e.hidden, recording: b.classList.contains('recording'), value: document.getElementById('peekHotkeyChoice').value, focus: document.activeElement ? document.activeElement.id : '' }; })()"
}
function StateText($s) { return "shown=$($s.shown) recording=$($s.recording) value=$($s.value) focus=$($s.focus)" }
# Picks 自定义… with the real pointer: a click opens the list, a click on its last row.
function PickCustomByMouse() {
  PageClick '#peekHotkeyChoice'; Start-Sleep -Milliseconds 700
  $r = Js "(() => { const o = document.querySelector('#peekHotkeyChoice option[value=custom]'); const b = o.getBoundingClientRect(), d = devicePixelRatio; return { x: (b.left + b.width / 2) * d, y: (b.top + b.height / 2) * d, w: b.width }; })()"
  Shot 'c-list-open'
  if ($r -and $r.w -gt 0) {
    $o = ClientOrigin ([G]::FindWindowW('PecoFence.Settings', [NullString]::Value))
    [G]::Click([int]($o.X + $r.x), [int]($o.Y + $r.y))
    Log "clicked the list row 自定义… at $([int]($o.X + $r.x)),$([int]($o.Y + $r.y))"
  } else {
    # The system's own list window: its last row with the keys.
    [G]::Press(0x23); Start-Sleep -Milliseconds 150; [G]::Press(0x0D)
    Log 'system list: End, Enter'
  }
  Start-Sleep -Milliseconds 900
}
function RecordCombo($mods, $vk) {
  foreach ($m in $mods) { [G]::KeyDown($m) }
  Start-Sleep -Milliseconds 120; [G]::Press($vk); Start-Sleep -Milliseconds 120
  foreach ($m in $mods) { [G]::KeyUp($m) }
  Start-Sleep -Milliseconds 900
}
function CustomHotkey() { return ([string](Pf settings get peek.customHotkey)).Trim().Trim('"') }
function PartC([bool]$taken) {
  trap { PfTrap $_; continue }
  Case "C: 速览 › 快捷键 › 自定义… (other programs hold every combination: $taken)"
  Check 'a debug build (the page is reached over CDP)' $PfDebugBuild
  CloseWindows
  $held = @()
  if ($taken) {
    # Win+Alt+Space and the fallback Ctrl+Alt+Space held by this job, as by other programs.
    foreach ($k in @(@(9, 0x20), @(3, 0x20))) {
      $id = 0x7100 + $held.Count
      if ([PkH]::RegisterHotKey([IntPtr]::Zero, $id, [uint32]($k[0] -bor 0x4000), [uint32]$k[1])) { $held += $id }
    }
    Log "held by the job: $($held.Count) of 2"
  }
  $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
  # Fresh settings: Ctrl+Alt+Space chosen, held by the job like its fallback Win+Alt+Space.
  if (PfAlive) { PfQuit }
  Remove-Item -Recurse -Force "$PfApp\config" -ErrorAction SilentlyContinue
  PfStart -Lang zh-CN
  Start-Sleep -Seconds 2
  ParkMouse
  Check 'settings open over CDP' (PfSettings)
  PageClick 'nav a[data-page=peek]'; Start-Sleep -Milliseconds 700
  $styled = Js "CSS.supports('appearance', 'base-select')"
  $help = Js "(() => { const h = document.getElementById('peekHotkeyHelp'); return h.className + ' | ' + h.textContent; })()"
  Log "list drawn by the page (base-select): $styled; help: $help"
  if ($taken) { Check 'setup: no combination is free (the help line says so)' ($help -like '*error*') "help=$help held=$($held.Count)" }

  PickCustomByMouse
  $s = EditState
  Shot "c-$taken-after-mouse"
  Check '自定义… by mouse: the recording box shows and records' ($s.shown -and $s.recording -and $s.value -eq 'custom' -and $s.focus -eq 'peekHotkey') (StateText $s)
  RecordCombo @(0x11, 0x10) 0x4B
  $saved = CustomHotkey
  $s = EditState
  Check '自定义… by mouse: Ctrl+Shift+K is saved and the box closes' ($saved -eq 'Ctrl+Shift+K' -and -not $s.recording) "saved=$saved $(StateText $s)"

  # The keyboard: the list has the focus, End picks its last row in place.
  Js "document.getElementById('peekHotkeyChoice').focus()" | Out-Null
  Start-Sleep -Milliseconds 300
  [G]::Press(0x23); Start-Sleep -Milliseconds 900
  $s = EditState
  Check '自定义… by keyboard: the recording box shows and records' ($s.shown -and $s.recording -and $s.value -eq 'custom' -and $s.focus -eq 'peekHotkey') (StateText $s)
  RecordCombo @(0x11, 0x12) 0x4A
  $saved = CustomHotkey
  Check '自定义… by keyboard: Ctrl+Alt+J is saved' ($saved -eq 'Ctrl+Alt+J') "saved=$saved"
  Shot "c-$taken-end"
  foreach ($id in $held) { [PkH]::UnregisterHotKey([IntPtr]::Zero, $id) | Out-Null }
  Remove-Item Env:\PECOFENCE_WEBVIEW_ARGS -ErrorAction SilentlyContinue
}

if ($Part -match 'A') { PartA $true; PartA $false }
if ($Part -match 'B') { PartB }
if ($Part -match 'C') { PartC $false; PartC $true }
