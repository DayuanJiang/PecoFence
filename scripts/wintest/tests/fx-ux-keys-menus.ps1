# Fence fix round 2026-11-07, area ux-keys-menus: keyboard focus between fences and into the
# Peek (U7), the folder fence's title glyph (U13), 「以后把 .pdf 文件都放到这里」 (U12), the
# marquee's 「在此显示文件夹…」 (U14), first-start names after a language change, the icon size
# names and the centred rename box (U16), 栅栏选项… beside the fence (U5) and the hover delay
# setting (U18). Real keys and clicks; outcomes read from the foreground window, dumps, the CLI
# and Explorer's windows. Two parts: run.sh --var Part=A (U7, U13) and --var Part=B (the rest).

if (-not $Part) { $Part = 'AB' }

Add-Type -Namespace Kx -Name W -MemberDefinition @'
[DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindowExW(IntPtr p, IntPtr a, string c, string t);
[DllImport("user32.dll")] public static extern int GetWindowLongW(IntPtr h, int i);
[DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr SendMessageW(IntPtr h, uint m, IntPtr w, string l);
[DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
[DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
[DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
[DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr h, System.Text.StringBuilder s, int n);
public struct RECT { public int L, T, R, B; }
public static string FgText() { var s = new System.Text.StringBuilder(256); GetWindowTextW(GetForegroundWindow(), s, 256); return s.ToString(); }
public static int Height(IntPtr h) { RECT r; GetWindowRect(h, out r); return r.B - r.T; }
'@ -ErrorAction SilentlyContinue

$KxRoot = "$PfRun\kx root"
$KxFiles = @('kx a1.txt', 'kx a2.txt', 'kx b1.txt', 'kx c1.txt', 'kx doc.pdf', 'kx doc2.pdf', 'kx doc3.pdf', 'kx app.lnk', 'kx new.pdf', 'kx new2.pdf', 'kx new3.pdf')
function KxClean() { foreach ($n in $KxFiles) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue } }
# selected count and focused item of fence $t in the next dump.
function KxSel($t) {
  $d = DumpOf $t 3
  $f = if ($d -and $d.raw -match ' focused=(Some\("(?:[^"\\]|\\.)*"\)|None)') { $Matches[1] } else { '?' }
  return "sel=$(DumpVal $d 'selected') focus=$f"
}
function KxSelOf($t) { $d = DumpOf $t 3; return [int](DumpVal $d 'selected') }
function KxFg() { return [Kx.W]::FgText() }
function KxWaitFg($title, $sec = 3) {
  for ($i = 0; $i -lt $sec * 10; $i++) { if ((KxFg) -eq $title) { return $true }; Start-Sleep -Milliseconds 100 }
  return $false
}
function KxTab([switch]$Back) { if ($Back) { [G]::KeyDown(0x10); [G]::Press(0x09); [G]::KeyUp(0x10) } else { [G]::Press(0x09) }; Start-Sleep -Milliseconds 600 }
function KxAlt($vk) { [G]::KeyDown(0x12); Start-Sleep -Milliseconds 30; [G]::Press($vk); Start-Sleep -Milliseconds 30; [G]::KeyUp(0x12); Start-Sleep -Milliseconds 900 }
# The folder the folder fence shows (`portal.current` is left out at its root).
function KxPortalNow() { $f = FenceGet 'kx root'; if ($f.portal.current) { return "$($f.portal.current)" }; return "$($f.portal.path)" }
function KxExplorerCount($path) { return @((Explorers) -split "`n" | Where-Object { $_ -match [regex]::Escape("path=$path") + '$' }).Count }
function KxDimmerOn() { $d = [G]::FindWindowW('PecoFence.PeekDimmer', [NullString]::Value); return ($d -ne [IntPtr]::Zero -and [G]::IsWindowVisible($d)) }
function KxEndPeek() { for ($i = 0; $i -lt 3 -and (KxDimmerOn); $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 600 } }
# A right-click on the title of fence $t ($dx px from its left edge); a submenu row hovered.
function KxTitleMenu($t, $dx = 160) {
  $r = (FenceGet $t).windowRect
  [G]::RClick($r.x + $dx, $r.y + 16)
  $m = MenuWin 5
  if ($m -ne [IntPtr]::Zero) { Log "menu of '$t': $(MenuTexts $m)" } else { Log "no menu on '$t'" }
  return $m
}
function KxSubMenu($m, $regex) {
  $p = MenuPoint $m $regex
  if (-not $p) { Log "no row $regex"; return [IntPtr]::Zero }
  [G]::MoveTo($p[0] - 30, $p[1]); Start-Sleep -Milliseconds 200; [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 900
  $ms = @([W]::Tops('#32768'))
  if ($ms.Count -ge 2) { return $ms[0] }
  return [IntPtr]::Zero
}
function KxFenceHwnd($t) { return (@([W]::Tops('PecoFence.Fence') | Where-Object { [W]::Title($_) -eq $t }) | Select-Object -First 1) }
# A click on the empty desktop: every fence's icons lose their selection.
function KxDeskClick() { [G]::Click(1000, 800); Start-Sleep -Milliseconds 700 }

Case 'setup'
CloseWindows
KxClean
Remove-Item -LiteralPath $KxRoot -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "$KxRoot\Sub A", "$KxRoot\Sub B" | Out-Null
Set-Content "$KxRoot\Sub A\inside.txt" 'x'; Set-Content "$KxRoot\top.txt" 'x'
foreach ($n in 'kx a1.txt', 'kx a2.txt', 'kx b1.txt', 'kx c1.txt') { DeskFile $n | Out-Null }
foreach ($n in 'kx doc.pdf', 'kx doc2.pdf', 'kx doc3.pdf') { Set-Content -LiteralPath (Join-Path $PfDesk $n) '%PDF-1.4' }
DeskLink 'kx app.lnk' | Out-Null
PfStart -Script (DumpScript 250 25)
Pf fence set inbox desktopLayout false | Out-Null
Pf fence roll --all | Out-Null
Pf fence create --title Alpha --rect '100,100,300,250' | Out-Null
Pf fence create --title Beta --rect '450,100,300,250' | Out-Null
Pf fence create --title Gamma --rect '100,420,300,250' | Out-Null
Pf fence create --portal "$KxRoot" --rect '450,420,300,250' | Out-Null
Pf item move "$PfDesk\kx a1.txt" "$PfDesk\kx a2.txt" "$PfDesk\kx doc.pdf" "$PfDesk\kx app.lnk" --to Alpha | Out-Null
Pf item move "$PfDesk\kx b1.txt" "$PfDesk\kx doc2.pdf" --to Beta | Out-Null
Pf item move "$PfDesk\kx c1.txt" "$PfDesk\kx doc3.pdf" --to Gamma | Out-Null
Pf fence set --all sort name | Out-Null
Start-Sleep -Seconds 2
FenceIds
Log "fences: $((@(PfJson fence list) | ForEach-Object { "$($_.title) $($_.windowRect.x),$($_.windowRect.y) rolled=$($_.rolledUp) n=$($_.itemCount)" }) -join ' ; ')"
Check 'four test fences, the first-start ones rolled up' ($Id['Alpha'] -and $Id['Beta'] -and $Id['Gamma'] -and $Id['kx root']) "$($Id.Keys -join ', ')"
[G]::MoveTo(1000, 850)

# Each part is a function with its own trap: a failed line is a failed check and the next line
# of the part runs (a plain if block would skip the rest of the part).
function KxPartA {
trap { PfTrap $_; continue }
Case 'U7 Tab and Shift+Tab move between fences (reading order)'
$p = ItemPt 'Alpha' 'kx a1'
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
Check 'a click in Alpha gives it the keyboard' (KxWaitFg 'Alpha') "fg='$(KxFg)' Alpha $(KxSel 'Alpha')"
$seq = @()
foreach ($want in 'Beta', 'Gamma', 'kx root', 'Alpha') {
  KxTab
  $ok = KxWaitFg $want
  $seq += "$want=$ok($(KxFg); $(KxSel $want))"
  Check "Tab moves the keyboard to $want" $ok "fg='$(KxFg)' $want $(KxSel $want)"
}
Log "tab sequence: $($seq -join ' | ')"
Check 'the fence left behind has no selection' ((KxSelOf 'Gamma') -eq 0) "Gamma $(KxSel 'Gamma')"
KxTab -Back
Check 'Shift+Tab goes back to the previous fence (wraps to kx root)' (KxWaitFg 'kx root') "fg='$(KxFg)'"
KxTab -Back; KxTab -Back
Check 'Shift+Tab twice more: Gamma, then Beta' (KxWaitFg 'Beta') "fg='$(KxFg)' Beta $(KxSel 'Beta')"
Check 'the fence Tab came to selects its first icon' ((KxSel 'Beta') -match 'sel=1 focus=Some\("kx b1"\)') "Beta $(KxSel 'Beta')"
[G]::Press(0x27); Start-Sleep -Milliseconds 500
Check 'arrows work in the fence Tab came to' ((KxSel 'Beta') -match 'focus=Some\("kx doc2"\)') "Beta $(KxSel 'Beta')"
[G]::Press(0x25); Start-Sleep -Milliseconds 400
[G]::Press(0x0D); Start-Sleep -Seconds 2
Check 'Enter opens the icon Tab and the arrows chose' ((NotepadTitles) -match 'kx b1') "notepad: $(NotepadTitles)"
Shot 'u7-tab-beta'
CloseWindows
Case 'U7 Ctrl+Tab still switches tabs only'
KxDeskClick
$p = ItemPt 'Alpha' 'kx a1'; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600
Chord 0x11 0x09; Start-Sleep -Milliseconds 600
Check 'Ctrl+Tab in a fence without tabs leaves the keyboard there' ((KxFg) -eq 'Alpha') "fg='$(KxFg)'"

Case 'U7 the desktop keeps its own keys'
KxDeskClick
Start-Process notepad.exe; Start-Sleep -Seconds 2
[G]::KeyDown(0x5B); [G]::Press(0x44); [G]::KeyUp(0x5B); Start-Sleep -Seconds 2
$fg0 = KxFg
[G]::Press(0x28); Start-Sleep -Milliseconds 300; [G]::Press(0x09); Start-Sleep -Milliseconds 500
$any = (KxSelOf 'Alpha') + (KxSelOf 'Beta') + (KxSelOf 'Gamma')
Check 'after Win+D, arrows and Tab stay with the desktop (no fence takes them)' ($any -eq 0 -and (KxFg) -notin 'Alpha', 'Beta', 'Gamma', 'kx root') "fg before='$fg0' after='$(KxFg)' selected in fences=$any"
[G]::KeyDown(0x5B); [G]::Press(0x44); [G]::KeyUp(0x5B); Start-Sleep -Seconds 1
CloseWindows

Case 'U7 Peek: the first key puts the keyboard into a fence'
# Every way in x the keys that enter: fresh selection each time.
$cases = @(@('Tab', 0x09, $false, 'Alpha', 'kx a1'), @('Shift+Tab', 0x09, $true, 'kx root', 'Sub A'), @('Down', 0x28, $false, 'Alpha', 'kx a1'),
  @('Right', 0x27, $false, 'Alpha', 'kx a1'), @('Up', 0x26, $false, 'Alpha', 'kx a1'), @('Left', 0x25, $false, 'Alpha', 'kx a1'))
foreach ($c in $cases) {
  KxDeskClick
  Start-Process notepad.exe; Start-Sleep -Seconds 2
  PeekKey; Start-Sleep -Milliseconds 1200
  if ($c[2]) { [G]::KeyDown(0x10); [G]::Press($c[1]); [G]::KeyUp(0x10) } else { [G]::Press($c[1]) }
  Start-Sleep -Milliseconds 700
  $s = KxSel $c[3]
  Check "Peek, then $($c[0]): $($c[3]) gets the keyboard on its first icon" ($s -match ('sel=1 focus=Some\("' + [regex]::Escape($c[4]) + '"\)') -and (KxDimmerOn)) "$($c[3]) $s peek=$(KxDimmerOn) fg='$(KxFg)'"
  if ($c[0] -eq 'Tab') { Shot 'u7-peek-tab' }
  KxEndPeek
  CloseWindows
}
Case 'U7 Peek: Tab between fences, arrows, Enter'
KxDeskClick
Start-Process notepad.exe; Start-Sleep -Seconds 2
PeekKey; Start-Sleep -Milliseconds 1200
[G]::Press(0x09); Start-Sleep -Milliseconds 600
[G]::Press(0x09); Start-Sleep -Milliseconds 600
Check 'Peek, Tab, Tab: Beta has the keyboard, Alpha let go' ((KxSel 'Beta') -match 'sel=1' -and (KxSelOf 'Alpha') -eq 0) "Beta $(KxSel 'Beta') Alpha $(KxSel 'Alpha')"
[G]::Press(0x27); Start-Sleep -Milliseconds 500
Check 'Peek: Right moves within Beta' ((KxSel 'Beta') -match 'focus=Some\("kx doc2"\)') "Beta $(KxSel 'Beta')"
[G]::Press(0x25); Start-Sleep -Milliseconds 400
[G]::Press(0x0D); Start-Sleep -Seconds 2
Check 'Peek: Enter opens it and the Peek ends' ((NotepadTitles) -match 'kx b1' -and -not (KxDimmerOn)) "notepad: $(NotepadTitles) peek=$(KxDimmerOn)"
CloseWindows
Case 'U7 Peek: a click still decides where the keys go'
KxDeskClick
PeekKey; Start-Sleep -Milliseconds 1200
$p = ItemPt 'Gamma' 'kx c1'; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600
[G]::Press(0x09); Start-Sleep -Milliseconds 600
Check 'Peek, click in Gamma, Tab: kx root next' ((KxSel 'kx root') -match 'sel=1') "kx root $(KxSel 'kx root') Gamma $(KxSel 'Gamma')"
KxEndPeek

Case 'U7 Alt+Left / Alt+Right in a folder fence'
KxDeskClick
$p = ItemPt 'kx root' 'Sub A'; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600
[G]::Press(0x0D); Start-Sleep -Milliseconds 1200
$in = KxPortalNow
Check 'Enter on Sub A opens it inside the fence' ($in -match 'Sub A$') "current=$in"
KxAlt 0x25
$back = KxPortalNow
Check 'Alt+Left goes back to the folder above' ($back -notmatch 'Sub A$' -and $back -match 'kx root$') "current=$back"
KxAlt 0x27
$fwd = KxPortalNow
Check 'Alt+Right goes forward into Sub A again' ($fwd -match 'Sub A$') "current=$fwd"
[G]::Press(0x08); Start-Sleep -Milliseconds 1200
Check 'Backspace still goes up' ((KxPortalNow) -match 'kx root$') "current=$(KxPortalNow)"
KxAlt 0x26; KxAlt 0x27
Check 'Alt+Up then Alt+Right: back in Sub A' ((KxPortalNow) -match 'Sub A$') "current=$(KxPortalNow)"
Pf fence up 'kx root' | Out-Null; Start-Sleep -Milliseconds 800
Case 'U7 Alt+Left in a folder fence during the Peek'
KxDeskClick
# Enter during the Peek opens a folder in Explorer and ends the Peek: go into Sub A first.
Pf fence open-folder 'kx root' 'Sub A' | Out-Null; Start-Sleep -Milliseconds 800
$in = KxPortalNow
PeekKey; Start-Sleep -Milliseconds 1200
$p = ItemPt 'kx root' 'inside'; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600
KxAlt 0x25
$up = KxPortalNow
KxAlt 0x27
Check 'Peek, click in the folder fence, Alt+Left: up; Alt+Right: forward; the Peek stays' ($in -match 'Sub A$' -and $up -match 'kx root$' -and (KxPortalNow) -match 'Sub A$' -and (KxDimmerOn)) "in=$in up=$up now=$(KxPortalNow) peek=$(KxDimmerOn)"
KxEndPeek
Pf fence up 'kx root' | Out-Null

Case 'U13 the folder glyph opens the folder'
CloseWindows
KxDeskClick
$r = (FenceGet 'kx root').windowRect
$gx = $r.x + 24; $gy = $r.y + 18
[G]::MoveTo($gx, $gy); Start-Sleep -Milliseconds 1500
Crop 'u13-glyph-tip' ($r.x - 10) ($r.y - 40) 330 100 | Out-Null
$tips = OcrText ($r.x - 10) ($r.y + 26) 330 40 'u13-glyph-tip-ocr'
Check 'the glyph has an infotip that says what a click does' ($tips -match 'Open folder in File Explorer') "tips: $tips"
$n0 = KxExplorerCount $KxRoot
[G]::Click($gx, $gy); Start-Sleep -Seconds 3
$n1 = KxExplorerCount $KxRoot
Shot 'u13-glyph-click'
Check 'a single click on the glyph opens the folder in File Explorer' ($n1 -eq $n0 + 1) "explorer windows on the folder: $n0 -> $n1; $(Explorers)"
CloseWindows
DblClick $gx $gy; Start-Sleep -Seconds 3
$n2 = KxExplorerCount $KxRoot
$rolled = (FenceGet 'kx root').rolledUp
Check 'a double-click on the glyph opens it once and does not roll the fence up' ($n2 -eq 1 -and -not $rolled) "explorer windows: $n2 rolled=$rolled"
CloseWindows
Pf fence unroll 'kx root' | Out-Null; Start-Sleep -Milliseconds 800
DblClick ($r.x + 160) $gy; Start-Sleep -Seconds 1
$rolled = (FenceGet 'kx root').rolledUp
Check 'a double-click on the title still rolls the fence up' ($rolled -and (KxExplorerCount $KxRoot) -eq 0) "rolled=$rolled explorer=$(KxExplorerCount $KxRoot)"
Pf fence unroll 'kx root' | Out-Null; Start-Sleep -Milliseconds 800
Pf fence open-folder 'kx root' 'Sub A' | Out-Null; Start-Sleep -Milliseconds 800
[G]::Click($gx, $gy); Start-Sleep -Seconds 2
Check 'inside a subfolder the glyph is still the up button (no Explorer)' ((KxPortalNow) -match 'kx root$' -and (KxExplorerCount $KxRoot) -eq 0 -and (KxExplorerCount "$KxRoot\Sub A") -eq 0) "current=$(KxPortalNow) $(Explorers)"
$m = KxTitleMenu 'kx root' 160
$t = MenuPick 'Open folder in File Explorer' $m; CloseMenus; Start-Sleep -Seconds 3
Check 'the fence menu entry still opens the folder' ((KxExplorerCount $KxRoot) -eq 1) "picked=$t $(Explorers)"
CloseWindows
}

function KxPartB {
trap { PfTrap $_; continue }
Case 'U12 Put all .pdf files here from now on'
KxDeskClick
$rules0 = @(Rules).Count
$m = RClickItem 'Alpha' 'kx doc'
$texts = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
Shot 'u12-item-menu'
Check 'the item menu of a .pdf offers it' ($texts -match 'Put all \.pdf files here from now on') "menu: $texts"
$t = MenuPick 'Put all \.pdf files here' $m
CloseMenus
$notice = WaitWin 'PecoFence.Notice' 5
Start-Sleep -Milliseconds 800
$nr = Rect $notice
$ntext = if ($notice -ne [IntPtr]::Zero) { OcrText $nr.L $nr.T ($nr.R - $nr.L) ($nr.B - $nr.T) 'u12-notice' } else { '' }
Shot 'u12-notice-shown'
$rl = @(Rules)
Check 'a rule .pdf -> Alpha comes first in the list' ($rl.Count -eq $rules0 + 1 -and $rl[0].targetTitle -eq 'Alpha' -and (($rl[0].allOf | ConvertTo-Json -Compress -Depth 5) -match 'pdf')) "rules $rules0 -> $($rl.Count); first: $($rl[0].name) -> $($rl[0].targetTitle) $($rl[0].allOf | ConvertTo-Json -Compress -Depth 5)"
Check 'a notice says where rules are edited' ($ntext -match 'Sorting rules') "notice: $ntext"
CloseNotice | Out-Null
Set-Content -LiteralPath (Join-Path $PfDesk 'kx new.pdf') '%PDF-1.4'
Check 'a new .pdf on the desktop goes to Alpha' ((WaitItemFence 'kx new.pdf' 'Alpha' 12) -eq 'Alpha') "in: $(ItemFence 'kx new.pdf')"
Case 'U12 a rule for the extension already sends it elsewhere'
$m = RClickItem 'Beta' 'kx doc2'
$t = MenuPick 'Put all \.pdf files here' $m
CloseMenus
Start-Sleep -Milliseconds 1200
$box = @([W]::Tops('#32770') | Where-Object { [W]::ProcOf($_) -eq 'pecofence' } | Select-Object -First 1)
$btext = if ($box) { DlgText $box[0] } else { '' }
Shot 'u12-question'
Check 'it asks first, naming the rule and the fence it targets now' ($btext -match 'Alpha' -and $btext -match 'Beta') "box: $btext"
if ($box) { ClickButton $box[0] '^&?Yes$' | Out-Null }; Start-Sleep -Milliseconds 800
CloseNotice | Out-Null
$rl = @(Rules)
Check 'Yes: the same rule now targets Beta (no second rule)' ($rl.Count -eq $rules0 + 1 -and $rl[0].targetTitle -eq 'Beta') "first: $($rl[0].name) -> $($rl[0].targetTitle), rules=$($rl.Count)"
Set-Content -LiteralPath (Join-Path $PfDesk 'kx new2.pdf') '%PDF-1.4'
Check 'a new .pdf now goes to Beta' ((WaitItemFence 'kx new2.pdf' 'Beta' 12) -eq 'Beta') "in: $(ItemFence 'kx new2.pdf')"
$m = RClickItem 'Gamma' 'kx doc3'
$t = MenuPick 'Put all \.pdf files here' $m
CloseMenus
Start-Sleep -Milliseconds 1200
$box = @([W]::Tops('#32770') | Where-Object { [W]::ProcOf($_) -eq 'pecofence' } | Select-Object -First 1)
if ($box) { ClickButton $box[0] '^&?No$' | Out-Null }; Start-Sleep -Milliseconds 800
$rl = @(Rules)
Check 'No: the rule keeps Beta' ($box -and $rl[0].targetTitle -eq 'Beta' -and $rl.Count -eq $rules0 + 1) "first: $($rl[0].targetTitle) rules=$($rl.Count)"
Case 'U12 not offered for shortcuts, folders and folder fences'
$m = RClickItem 'Alpha' 'kx app'
$t1 = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
CloseMenus
$m = RClickItem 'kx root' 'Sub A'
$t2 = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
CloseMenus
$m = RClickItem 'kx root' 'top'
$t3 = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
CloseMenus
Check 'no offer on a shortcut, a folder or a file in a folder fence' ($t1 -and $t2 -and $t3 -and ($t1 + $t2 + $t3) -notmatch 'from now on') "lnk: $t1 || folder: $t2 || portal file: $t3"

Case 'U14 the marquee offers a folder fence in the drawn rectangle'
KxDeskClick
$c0 = @(PfJson fence list).Count
DragTo 820 120 1120 380
$m = MenuWin 5
$texts = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
Shot 'u14-marquee-menu'
Check 'the marquee menu has Create a fence here and Show a folder here...' ($texts -match 'Create a fence here' -and $texts -match 'Show a folder here') "menu: $texts"
$t = MenuPick 'Show a folder here' $m
CloseMenus
$dlg = [IntPtr]::Zero
for ($i = 0; $i -lt 50 -and $dlg -eq [IntPtr]::Zero; $i++) { $dlg = @([W]::Tops('#32770') | Where-Object { [W]::ProcOf($_) -eq 'pecofence' } | Select-Object -First 1)[0]; if (-not $dlg) { $dlg = [IntPtr]::Zero }; Start-Sleep -Milliseconds 100 }
# The folder picker's Folder: box (1152) gets the path, then OK; OK again if it only went in.
$fe = if ($dlg -ne [IntPtr]::Zero) { [QS]::FileNameEdit($dlg, $true) } else { [IntPtr]::Zero }
Log "folder dialog $dlg folder box $fe"
if ($fe -ne [IntPtr]::Zero) {
  # The dialog takes OK only once it has finished opening.
  for ($i = 0; $i -lt 30 -and -not [G]::IsWindowVisible($dlg); $i++) { Start-Sleep -Milliseconds 100 }
  Start-Sleep -Milliseconds 1500
  # Typed into the Folder: box like a user: a click, Ctrl+A, the path, Enter (Enter again if
  # the dialog only went into the folder).
  $er = Rect $fe
  [G]::Click([int](($er.L + $er.R) / 2), [int](($er.T + $er.B) / 2)); Start-Sleep -Milliseconds 300
  Chord 0x11 0x41; Start-Sleep -Milliseconds 200
  [G]::Type("$KxRoot\Sub B"); Start-Sleep -Milliseconds 300
  Log "folder box text: '$([QS]::FileName($dlg))'"
  [G]::Press(0x0D); Start-Sleep -Milliseconds 1500
  # Enter only went into the folder: its Select Folder button picks it.
  if ([G]::IsWindowVisible($dlg)) { Log "folder dialog after Enter: $([QS]::Address($dlg))"; ClickButton $dlg 'Select Folder' | Out-Null; Start-Sleep -Milliseconds 1500 }
}
# Never leave the dialog open: the app waits in it and every later CLI call times out.
if ($dlg -ne [IntPtr]::Zero -and [G]::IsWindowVisible($dlg)) { Log 'folder dialog still open: cancelled'; [Kx.W]::PostMessageW($dlg, 0x111, [IntPtr]2, [IntPtr]::Zero) | Out-Null; Start-Sleep -Seconds 1 }
Start-Sleep -Seconds 2
$nf = @(PfJson fence list) | Where-Object { $_.title -eq 'Sub B' } | Select-Object -First 1
$nr = $nf.windowRect
Shot 'u14-folder-fence-in-rect'
Check 'a folder fence for the picked folder fills the drawn rectangle' ($nf -and [math]::Abs($nr.x - 820) -le 24 -and [math]::Abs($nr.y - 120) -le 24 -and [math]::Abs($nr.w - 300) -le 48 -and @(PfJson fence list).Count -eq $c0 + 1) "dialog=$dlg fence=$($nf.title) rect=$($nr.x),$($nr.y),$($nr.w),$($nr.h)"
if ($nf) { Pf fence delete 'Sub B' | Out-Null }

Case 'U16 rename box centred on the icon'
KxDeskClick
$p = ItemPt 'Gamma' 'kx c1'; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600
[G]::Press(0x71); Start-Sleep -Milliseconds 800
$ren = WaitWin 'PecoFence.Rename' 3
$ed = if ($ren -ne [IntPtr]::Zero) { [Kx.W]::FindWindowExW($ren, [IntPtr]::Zero, 'Edit', [NullString]::Value) } else { [IntPtr]::Zero }
$style = if ($ed -ne [IntPtr]::Zero) { [Kx.W]::GetWindowLongW($ed, -16) } else { 0 }
$rr = Rect $ren
Crop 'u16-rename-box' ($p[0] - 150) ($p[1] - 60) 300 130 | Out-Null
Check 'the rename box text is centred (ES_CENTER) and the box is centred on the icon' (($style -band 1) -eq 1 -and [math]::Abs((($rr.L + $rr.R) / 2) - $p[0]) -le 6) "edit style=0x$('{0:X}' -f $style) box=$($rr.L)..$($rr.R) icon x=$($p[0])"
[G]::Press(0x1B); Start-Sleep -Milliseconds 400
Case 'U16 icon size names match Windows'
Pf fence set Gamma iconSize 96 | Out-Null; Start-Sleep -Milliseconds 800
$m = KxTitleMenu 'Gamma' 160
$sub = KxSubMenu $m '^View'
$rows = if ($sub -ne [IntPtr]::Zero) { @(MenuRows $sub) } else { @() }
$checked = (@($rows | Where-Object { $_.state -band 8 }) | ForEach-Object { $_.text }) -join ', '
$all = ($rows | ForEach-Object { $_.text }) -join ' | '
Check 'View menu: 96 px is Large icons, 64 px Medium-large icons' ($all -match 'Small icons \| Medium icons \| Medium-large icons \| Large icons' -and $checked -match 'Large icons' -and $checked -notmatch 'Medium') "rows: $all; checked: $checked"
CloseMenus
Pf fence set Gamma iconSize 48 | Out-Null
Case 'U16 first-start fence names follow the interface language'
Pf fence rename 'Folders' 'My stuff' | Out-Null
Pf settings set language ja | Out-Null; Start-Sleep -Seconds 2
$ja = (@(PfJson fence list) | ForEach-Object { $_.title }) -join ', '
$rja = (@(Rules) | ForEach-Object { $_.name }) -join ', '
Check 'Japanese: Programs, Files and documents and Desktop renamed; My stuff kept' ($ja -match 'アプリ' -and $ja -match 'ファイルとドキュメント' -and $ja -notmatch 'Programs' -and $ja -notmatch 'Files and documents' -and $ja -match 'My stuff' -and $ja -match 'Alpha') "fences: $ja; rules: $rja"
Pf settings set language en | Out-Null; Start-Sleep -Seconds 2
$en = (@(PfJson fence list) | ForEach-Object { $_.title }) -join ', '
Check 'back to English: the English names again' ($en -match 'Programs' -and $en -match 'Files and documents' -and $en -match 'Desktop' -and $en -match 'My stuff') "fences: $en; rules: $((@(Rules) | ForEach-Object { $_.name }) -join ', ')"
Pf fence rename 'My stuff' 'Folders' | Out-Null

Case 'U5 Fence options open beside the fence'
KxDeskClick
$a = (FenceGet 'Alpha').windowRect
$m = KxTitleMenu 'Alpha' 160
$t = MenuPick '^Fence options' $m
CloseMenus
$sw = WaitWin 'PecoFence.Settings' 20
Start-Sleep -Seconds 3
$sr = Rect $sw
$overlap = -not ($sr.R -le $a.x -or $sr.L -ge $a.x + $a.w -or $sr.B -le $a.y -or $sr.T -ge $a.y + $a.h)
Shot 'u5-options-beside'
Check 'the settings window does not cover Alpha' ($sw -ne [IntPtr]::Zero -and -not $overlap) "settings $($sr.L),$($sr.T),$($sr.R),$($sr.B); Alpha $($a.x),$($a.y),$($a.w)x$($a.h)"
$o = ClientOrigin $sw
$top = OcrText $o.X $o.Y ($sr.R - $sr.L - 16) 360 'u5-options-top'
Check 'Automatic height is on the first screen of the fence options' ($top -match 'Automatic height') "ocr: $top"
[L]::PostMessageW($sw, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; Start-Sleep -Seconds 1
$g = (FenceGet 'Gamma').windowRect
Pf fence move Gamma --x 1180 --y 300 | Out-Null; Start-Sleep -Milliseconds 800
$g = (FenceGet 'Gamma').windowRect
$m = KxTitleMenu 'Gamma' 160
$t = MenuPick '^Fence options' $m
CloseMenus
$sw = WaitWin 'PecoFence.Settings' 20
Start-Sleep -Seconds 3
$sr = Rect $sw
$overlap = -not ($sr.R -le $g.x -or $sr.L -ge $g.x + $g.w -or $sr.B -le $g.y -or $sr.T -ge $g.y + $g.h)
Shot 'u5-options-beside-right-fence'
Check 'a fence on the right: the window opens to its left' (-not $overlap -and $sr.R -le $g.x) "settings $($sr.L),$($sr.T),$($sr.R),$($sr.B); Gamma $($g.x),$($g.y),$($g.w)x$($g.h)"
[L]::PostMessageW($sw, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; Start-Sleep -Seconds 1
Pf fence move Gamma --x 100 --y 420 | Out-Null

Case 'U18 hover delay: fast, normal, slow'
Pf fence roll Beta | Out-Null; Start-Sleep -Seconds 1
$bh = KxFenceHwnd 'Beta'
$b = (FenceGet 'Beta').windowRect
$res = @{}
foreach ($speed in 'fast', 'normal', 'slow') {
  $set = Pf settings set rollUp.hoverSpeed $speed
  [G]::MoveTo(1000, 850); Start-Sleep -Milliseconds 1500
  $h0 = [Kx.W]::Height($bh)
  $sw2 = [Diagnostics.Stopwatch]::StartNew()
  [G]::MoveTo($b.x + 150, $b.y + 16)
  $ms = -1
  while ($sw2.ElapsedMilliseconds -lt 3000) { if ([Kx.W]::Height($bh) -gt $h0 + 20) { $ms = $sw2.ElapsedMilliseconds; break }; Start-Sleep -Milliseconds 10 }
  $res[$speed] = $ms
  Log "hover $speed : expands after $ms ms (h0=$h0) set: $($set.Trim())"
  [G]::MoveTo(1000, 850); Start-Sleep -Milliseconds 1500
}
Check 'fast < normal < slow, about 150 / 400 / 1000 ms' ($res['fast'] -gt 0 -and $res['fast'] -lt $res['normal'] - 100 -and $res['normal'] -lt $res['slow'] - 300) "fast=$($res['fast']) normal=$($res['normal']) slow=$($res['slow'])"
Pf settings set rollUp.hoverSpeed normal | Out-Null
Pf fence unroll Beta | Out-Null
}

if ($Part -match 'A') { KxPartA }
if ($Part -match 'B') { KxPartB }

Case 'end'
PfQuit
CloseWindows
KxClean
Remove-Item -LiteralPath $KxRoot -Recurse -Force -ErrorAction SilentlyContinue
