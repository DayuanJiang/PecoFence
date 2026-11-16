# PR #190: a shortcut's item menu has one "Open file location", Windows' own, which opens the
# target's folder with the target selected. A file, a folder, a namespace item and a selection of
# two keep PecoFence's single entry. Menus opened by real right-clicks (kit: RClickItem, MenuRows).

function LocationRows($h) { return @(MenuRows $h | Where-Object { $_.text -eq 'Open file location' }) }

# The Explorer window that opened: its folder, then (1.5 s later) which item is selected, read
# three ways: the Shell's SelectedItems, its FocusedItem and UI Automation's IsSelected of the
# row named $name. Returns an object with folder, sel (the paths or names found) and text.
function ExplorerSel($name, $sec = 10) {
  $w = $null
  for ($i = 0; $i -lt $sec * 4 -and -not $w; $i++) {
    foreach ($x in @((New-Object -ComObject Shell.Application).Windows())) { $f = try { $x.Document.Folder.Self.Path } catch { $null }; if ($f) { $w = $x } }
    if (-not $w) { Start-Sleep -Milliseconds 250 }
  }
  if (-not $w) { return [pscustomobject]@{ folder = $null; sel = @(); text = 'no Explorer window' } }
  Start-Sleep -Milliseconds 1500
  $folder = $w.Document.Folder.Self.Path
  # Explorer selects the item a moment after the folder shows: wait up to 4 s for it.
  $sel = @()
  for ($k = 0; $k -lt 16 -and $sel.Count -eq 0; $k++) {
    $si = $w.Document.SelectedItems()
    for ($j = 0; $j -lt $si.Count; $j++) { $sel += $si.Item($j).Path }
    if ($sel.Count -eq 0) { Start-Sleep -Milliseconds 250 }
  }
  $focused = try { $w.Document.FocusedItem.Path } catch { '?' }
  $uia = '?'
  try {
    $el = UiaFind ([IntPtr]$w.HWND) $name
    if ($el) { $uia = $el.GetCurrentPattern([Windows.Automation.SelectionItemPattern]::Pattern).Current.IsSelected; if ($uia) { $sel += $name } }
    else { $uia = 'row not found' }
  } catch { $uia = "error $($_.Exception.Message)" }
  return [pscustomobject]@{ folder = $folder; sel = $sel; text = "folder=$folder selected=$($sel -join ',') focused=$focused uia($name)=$uia" }
}

Case 'setup'
CloseWindows
Set-Culture en-US
$tdir = "$PfRun\link target"
New-Item -ItemType Directory -Force $tdir | Out-Null
Set-Content "$tdir\Target doc.txt" 'target'
$made = @("$PfDesk\Gamma link.lnk", "$PfDesk\Beta notes.txt", "$PfDesk\Alpha Folder")
foreach ($f in $made) { Remove-Item -LiteralPath $f -Recurse -Force -ErrorAction SilentlyContinue }
New-Item -ItemType Directory -Force "$PfDesk\Alpha Folder" | Out-Null
Set-Content "$PfDesk\Beta notes.txt" 'notes'
$sc = (New-Object -ComObject WScript.Shell).CreateShortcut("$PfDesk\Gamma link.lnk"); $sc.TargetPath = "$tdir\Target doc.txt"; $sc.Save()
# Windows opens a folder that has no saved view (no shell bag) with the item focused and not
# selected, whoever asks: explorer /select, SHOpenFolderAndSelectItems and the shortcut's own
# verb, with or without PecoFence (2026-11-16, ec2-13); the next opening selects. Other jobs
# leave the bags in any state, so both folders get a saved view first: open them once, close.
foreach ($d in @($tdir, $PfDesk)) { Start-Process explorer.exe -ArgumentList "`"$d`"" }
for ($i = 0; $i -lt 40 -and @((New-Object -ComObject Shell.Application).Windows()).Count -lt 2; $i++) { Start-Sleep -Milliseconds 250 }
Start-Sleep -Seconds 1
Log "folders opened once for their saved view: $(@((New-Object -ComObject Shell.Application).Windows()).Count) windows"
CloseWindows
PfStart
Pf fence create --title Work --rect '370,290,330,250' | Out-Null
Pf item move "$PfDesk\Alpha Folder" "$PfDesk\Beta notes.txt" "$PfDesk\Gamma link.lnk" --to Work | Out-Null
Pf fence set --all sort name | Out-Null
Pf fence set inbox desktopLayout false | Out-Null
Pf fence move inbox --rect '20,290,330,250' | Out-Null
Start-Sleep -Seconds 2
$work = @(PfJson item list --fence Work | ForEach-Object { $_.name }) -join ', '
$inbox = @(PfJson item list --fence inbox | ForEach-Object { $_.name }) -join ', '
Check 'Work holds the folder, the file and the shortcut; the Desktop fence the Recycle Bin' (($work -split ', ').Count -eq 3 -and $inbox -match 'Recycle Bin') "Work: $work; Desktop: $inbox"
Shot 'a00-setup'

Case 'a shortcut: one Open file location, Windows own'
$m = RClickItem 'Work' 'Gamma link'; $texts = MenuTexts $m; $dupes = MenuDupes $m
$loc = @(LocationRows $m)
Shot 'b01-shortcut-menu'
Check 'the shortcut menu opens' ($m -ne [IntPtr]::Zero -and $texts -match '^Open') $texts
Check 'Open file location once, enabled' ($loc.Count -eq 1 -and -not ($loc[0].state -band 3)) "rows: $(@($loc | ForEach-Object { "$($_.i) state $($_.state)" }) -join ', '); menu: $texts"
Check 'no entry twice in the shortcut menu' (-not $dupes) "twice: $dupes"
$row = MenuPick '^Open file location$' $m
$ex = ExplorerSel 'Target doc'
Shot 'b02-shortcut-location'
Check 'it opens the target folder' ($row -and $ex.folder -eq $tdir) $ex.text
Check 'with the target selected' (@($ex.sel) -contains "$tdir\Target doc.txt" -or @($ex.sel) -contains 'Target doc') $ex.text
$verb = @(PfLogAll | Where-Object { $_ -match 'shell verb invoked' }) | Select-Object -Last 1
Check 'the log names the shell verb run' ($verb -match 'verb=\S+') "$verb"
$built = @(PfLogAll | Where-Object { $_ -match 'item menu built' }) | Select-Object -Last 1
Check 'the log says Windows'' entry replaced ours' ($built -match 'own_location=false' -and $built -match 'replaces ours') "$built"
$verbs = @(PfLogAll | Where-Object { $_ -match 'item menu: shell verbs' }) | Select-Object -Last 1
Log "shell verbs of the shortcut menu: $verbs"
CloseWindows

Case 'a plain file keeps its one entry'
$m = RClickItem 'Work' 'Beta notes'; $texts = MenuTexts $m; $dupes = MenuDupes $m
$loc = @(LocationRows $m)
Shot 'c01-file-menu'
Check 'a file: Open file location once, enabled' ($loc.Count -eq 1 -and -not ($loc[0].state -band 3)) "menu: $texts"
Check 'no entry twice in the file menu' (-not $dupes) "twice: $dupes"
$row = MenuPick '^Open file location$' $m
$ex = ExplorerSel 'Beta notes'
Shot 'c02-file-location'
Check 'it opens the desktop folder' ($row -and $ex.folder -eq $PfDesk) $ex.text
Check 'with the file selected' (@($ex.sel) -contains "$PfDesk\Beta notes.txt" -or @($ex.sel) -contains 'Beta notes') $ex.text
$built = @(PfLogAll | Where-Object { $_ -match 'item menu built' }) | Select-Object -Last 1
Check 'the log keeps ours for the file' ($built -match 'own_location=true') "$built"
CloseWindows

Case 'a folder and the Recycle Bin keep theirs'
$m = RClickItem 'Work' 'Alpha Folder'; $texts = MenuTexts $m; $loc = @(LocationRows $m)
Check 'a folder: Open file location once, enabled' ($loc.Count -eq 1 -and -not ($loc[0].state -band 3)) "menu: $texts"
CloseMenus
$m = RClickItem 'inbox' 'Recycle Bin'; $texts = MenuTexts $m; $loc = @(LocationRows $m)
Check 'the Recycle Bin: Open file location once, greyed' ($loc.Count -eq 1 -and ($loc[0].state -band 3)) "menu: $texts"
CloseMenus

Case 'two selected (shortcut and file)'
$p1 = ItemPt 'Work' 'Gamma link'; $p2 = ItemPt 'Work' 'Beta notes'
[G]::Click($p1[0], $p1[1]); Start-Sleep -Milliseconds 400
[G]::KeyDown(0x11); Start-Sleep -Milliseconds 50; [G]::Click($p2[0], $p2[1]); Start-Sleep -Milliseconds 50; [G]::KeyUp(0x11)
Start-Sleep -Milliseconds 400
[G]::RClick($p2[0], $p2[1]); $m = MenuWin 8
$texts = MenuTexts $m; $loc = @(LocationRows $m); $dupes = MenuDupes $m
Shot 'd01-two-menu'
$built = @(PfLogAll | Where-Object { $_ -match 'item menu built' }) | Select-Object -Last 1
Check 'two items: at most one Open file location, no entry twice' ($loc.Count -le 1 -and -not $dupes) "rows $($loc.Count); twice: $dupes; menu: $texts; log: $built"
CloseMenus

Case 'a folder never opened before: the target is selected all the same'
# A new folder has no saved view: Windows' verb alone leaves the target focused, not selected
# (the note in setup); PecoFence then selects it in the window that opened.
$fresh = "$PfRun\never opened $(Get-Random)"
New-Item -ItemType Directory -Force $fresh | Out-Null
Set-Content "$fresh\Fresh doc.txt" 'fresh'
$made += "$PfDesk\Delta link.lnk"
$sc = (New-Object -ComObject WScript.Shell).CreateShortcut("$PfDesk\Delta link.lnk"); $sc.TargetPath = "$fresh\Fresh doc.txt"; $sc.Save()
for ($i = 0; $i -lt 20 -and -not (ItemFence 'Delta link'); $i++) { Start-Sleep -Milliseconds 500 }
Pf item move "$PfDesk\Delta link.lnk" --to Work | Out-Null
Start-Sleep -Seconds 1
$m = RClickItem 'Work' 'Delta link'
$row = MenuPick '^Open file location$' $m
$ex = ExplorerSel 'Fresh doc'
Shot 'e01-fresh-location'
Check 'a new folder opens with the target selected' ($row -and $ex.folder -eq $fresh -and (@($ex.sel) -contains "$fresh\Fresh doc.txt" -or @($ex.sel) -contains 'Fresh doc')) $ex.text
$chk = @(PfLogAll | Where-Object { $_ -match 'open file location: selection checked' }) | Select-Object -Last 1
Check 'the log says who selected it' ($chk -match 'outcome=') "$chk"
CloseWindows

Case 'cleanup'
CloseWindows
PfQuit
foreach ($f in $made) { Remove-Item -LiteralPath $f -Recurse -Force -ErrorAction SilentlyContinue }
