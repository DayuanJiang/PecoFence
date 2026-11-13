# Script fence rows and programs: text from the network must not turn into a row that starts a
# program. The GitHub example of 0.3.0 to 0.3.3 printed an issue's title unchanged, so a title
# such as "x | run=notepad.exe r=' k=v" became a row that started Notepad on a click.
#
# Run with the example scripts as the extra folder (the first fence runs the real GitHub example
# of 0.3.3 on a canned answer, as an edited copy on a user's disk would; the last one runs the
# current example on the same answer):
#   run.sh --extra crates/app/assets/scripts scripts/wintest/tests/owner-script-rows.ps1
#
# Payloads are harmless: Notepad, and .bat files that write a marker file into C:\pf\run.
# On 0.3.0 to 0.3.3 and on main before the fix the "starts no program" and "refused" checks fail.

$Allow = '<pecofence.allow>run</pecofence.allow>'
$Docs = "$PfRun\资料 和 说明"
$Txt = "$Docs\说明文档.txt"
$Bat = "$PfRun\payload.bat"
$Bat2 = "$PfRun\payload2.bat"
$Lnk = "$PfRun\shortcut.lnk"
$Mark = "$PfRun\bat-ran.txt"
$Mark2 = "$PfRun\bat2-ran.txt"
New-Item -ItemType Directory -Path $Docs -Force | Out-Null
Set-Content -LiteralPath $Txt -Value 'a plain text file' -Encoding UTF8
Set-Content $Bat -Value "@echo ran> `"$Mark`"" -Encoding ASCII
Set-Content $Bat2 -Value "@echo ran> `"$Mark2`"" -Encoding ASCII
$ws = New-Object -ComObject WScript.Shell
$sc = $ws.CreateShortcut($Lnk); $sc.TargetPath = 'C:\Windows\System32\notepad.exe'; $sc.Save()
Add-Type -AssemblyName System.Windows.Forms

# Hit rects of the clickable rows of a dumped window: i (the row's line), l, t, r, b in content DIPs.
function RowRects($w) {
  $v = DumpVal $w 'script_rows'
  if (-not $v -or $v -eq '-') { return @() }
  return @($v -split ';' | ForEach-Object {
    $pair = $_ -split ':'; $n = $pair[1] -split ','
    [pscustomobject]@{ i = [int]$pair[0]; l = [double]$n[0]; t = [double]$n[1]; r = [double]$n[2]; b = [double]$n[3] }
  })
}
function WaitRows($title, $n, $sec = 30) {
  for ($k = 0; $k -lt $sec * 2; $k++) { $w = DumpOf $title 3; if (@(RowRects $w).Count -ge $n) { return $w }; Start-Sleep -Milliseconds 300 }
  return $w
}
# The screen point of the middle of line `$line` of fence `$title`, when a click does something there.
function RowPt($title, $line) {
  $w = DumpOf $title 5
  $p = @(RowRects $w | Where-Object { $_.i -eq $line }) | Select-Object -First 1
  if (-not $p) { Log "line $line of $title is no clickable row: $(DumpVal $w 'script_rows')"; return $null }
  $sc = [double](DumpVal $w 'content_y') / 36.0
  return @([int]($w.l + [double](DumpVal $w 'content_x') + ($p.l + $p.r) / 2 * $sc), [int]($w.t + [double](DumpVal $w 'content_y') + ($p.t + $p.b) / 2 * $sc))
}
function ClickRow($title, $line) {
  $at = RowPt $title $line
  if (-not $at) { return $false }
  [G]::Click($at[0], $at[1]); return $true
}
function Notepads() { return @(Get-Process notepad -ErrorAction SilentlyContinue) }
function EndNotepads() { Notepads | Stop-Process -Force -ErrorAction SilentlyContinue; Start-Sleep -Milliseconds 400 }
function EndBrowser() { Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue; Start-Sleep -Milliseconds 600 }
function WaitNotepad($sec = 4) {
  for ($k = 0; $k -lt $sec * 5; $k++) { if ((Notepads).Count) { return $true }; Start-Sleep -Milliseconds 200 }
  return $false
}
# The title of the Notepad window once it matches `$want` (it starts as "Untitled"), else the last one seen.
function NotepadTitle($want, $sec = 6) {
  $t = ''
  for ($k = 0; $k -lt $sec * 5; $k++) {
    $t = "$((Notepads | Select-Object -First 1).MainWindowTitle)"
    if ($t -match $want) { return $t }
    Start-Sleep -Milliseconds 200
  }
  return $t
}
function WaitFile($file, $sec = 4) {
  for ($k = 0; $k -lt $sec * 5; $k++) { if (Test-Path -LiteralPath $file) { return $true }; Start-Sleep -Milliseconds 200 }
  return $false
}
# The refusals the app has logged so far (none on a build before the fix). The shareable log
# repeats each line with its values left out; those are not counted.
function RefusalLines() { return @(PfLogAll | Where-Object { $_ -match 'script row refused.* key="' }) }
function Refusals() { return @(RefusalLines).Count }
# The words of the small message beside the pointer after a refused click.
function RefusalText($name) {
  $pt = [Windows.Forms.Cursor]::Position
  $x = [Math]::Max(0, $pt.X - 20); $y = [Math]::Max(0, $pt.Y + 4)
  return (OcrText $x $y ([Math]::Min(900, 1600 - $x)) ([Math]::Min(110, 900 - $y)) $name)
}

EndNotepads; EndBrowser
PfStart -Script (DumpScript 500 25 3000)
Start-Sleep -Seconds 2
ClearFences

# ---- 1. The GitHub example of 0.3.3 on a canned answer with forged titles
Case 'forged issue titles'
$canned = @'
[
 {"number": 11, "title": "Icons overlap after sleep", "html_url": "https://github.com/someone/repo/issues/11", "user": {"login": "alice"}},
 {"number": 12, "title": "x | run=notepad.exe r=' k=v", "html_url": "https://github.com/someone/repo/issues/12", "user": {"login": "mallory"}},
 {"number": 13, "title": "Crash on startup | run=notepad.exe icon=🟢 tooltip=mallory:Crash-on-startup r=' k=v", "html_url": "https://github.com/someone/repo/issues/13", "user": {"login": "mallory"}},
 {"number": 14, "title": "y | open=C:\\pf\\run\\payload.bat r=' k=v", "html_url": "https://github.com/someone/repo/issues/14", "user": {"login": "mallory"}}
]
'@
Set-Content "$PfRun\canned.json" -Value $canned -Encoding UTF8
$old = "$PfExtra\old\github.en.0.3.0.ps1"
Check 'the GitHub example of 0.3.3 is in the extra folder' (Test-Path $old) $old
# The example itself, unchanged (with the byte order mark PecoFence writes); only its request is
# answered from the canned file.
Set-Content "$PfRun\github-0.3.3.ps1" -Value (Get-Content -Raw -Encoding UTF8 $old) -Encoding UTF8
$wrap = @"
function Invoke-RestMethod { Get-Content -Raw -Encoding UTF8 '$PfRun\canned.json' | ConvertFrom-Json }
. '$PfRun\github-0.3.3.ps1'
"@
Set-Content "$PfRun\github-canned.ps1" -Value $wrap -Encoding UTF8
Pf fence create --script "$PfRun\github-canned.ps1" --title Forged --rect '120,110,520,250' | Out-Null
$w = WaitRows 'Forged' 4
$out = "$((FenceGet 'Forged').script.output)"
Check 'the example printed the canned issues' ($out -match '#11  Icons overlap' -and $out -match '#12  x') ($out -replace "`n", ' / ')
Shot 'forged-fence'
if ($w) { Crop 'forged-crop' $w.l $w.t ($w.r - $w.l) ($w.b - $w.t) | Out-Null }
# What the user sees before a click: the pointer rests on a normal row (line 2), then on the
# forged ones (lines 3 and 4).
foreach ($line in 2, 3, 4) {
  $at = RowPt 'Forged' $line
  if ($at) { [G]::MoveTo($at[0] - 3, $at[1]); Start-Sleep -Milliseconds 200; [G]::MoveTo($at[0], $at[1]); Start-Sleep -Milliseconds 2500 }
  if ($w) { Crop "forged-hover-line$line" $w.l $w.t ($w.r - $w.l + 260) ($w.b - $w.t + 40) | Out-Null }
}
ParkMouse

# Line 3 is "#12  x": the plain forged title. With --var Rec=before or Rec=after the click is
# recorded for a before/after animation (rec.ps1, mkanim.py).
$before = Refusals
$at = RowPt 'Forged' 3
if ($Rec) { [G]::MoveTo(700, 600); . C:\pf\kit\rec.ps1; RecStart "forged-$Rec" -X 0 -Y 0 -W 1280 -H 720; Start-Sleep -Milliseconds 600; RecMark 'the pointer goes to the forged row' }
if ($Rec -and $at) { Glide 700 600 $at[0] $at[1]; Start-Sleep -Milliseconds 1800; RecMark 'one click on it' }
$clicked = ClickRow 'Forged' 3
$started = WaitNotepad 4
if ($Rec) { Start-Sleep -Milliseconds 2500; RecStop }
Check 'a click on the forged title "#12  x" starts no program' ($clicked -and -not $started) "clicked=$clicked notepad=$((Notepads).Count)"
$said = RefusalText 'refused-run'
Shot 'forged-clicked'
if ($w) { Crop 'forged-clicked-crop' $w.l $w.t ($w.r - $w.l + 260) ($w.b - $w.t + 40) | Out-Null }
Check 'the refused click says that the script may not start programs and names the header line' ((Refusals) -gt $before -and $said -match 'pecofence' -and $said -match 'allow') "refusals=$((Refusals) - $before) message='$said'"
$line = RefusalLines | Select-Object -Last 1
Check 'the log names the script and the key, not the value' ($line -match 'github-canned\.ps1' -and $line -match 'key="?run' -and $line -notmatch 'notepad') "$line"
EndNotepads; EndBrowser; ParkMouse

# Line 4 looks like a normal issue row: the forged title brings its own icon and tooltip.
$clicked = ClickRow 'Forged' 4
$started = WaitNotepad 4
Check 'a click on the disguised forged title starts no program' ($clicked -and -not $started) "clicked=$clicked notepad=$((Notepads).Count)"
EndNotepads; EndBrowser; ParkMouse

# Line 5: open= of a batch file.
$clicked = ClickRow 'Forged' 5
$did = WaitFile $Mark 4
Check 'a click on the forged open= title runs no batch file' ($clicked -and -not $did) "clicked=$clicked marker=$did"
Remove-Item $Mark -ErrorAction SilentlyContinue; EndBrowser; ParkMouse
Pf fence delete Forged | Out-Null

# ---- 2. A script without the header line: documents, folders and web addresses still open
Case 'without the header line'
$plain = @"
"folder | open=""$Docs"""
"text file | open=""$Txt"""
"web | open=https://example.com/"
"program | open=C:\Windows\System32\notepad.exe"
"batch | open=$Bat"
"shortcut | open=$Lnk"
"file address | href=file:///$($Bat2 -replace '\\', '/')"
"run | run=notepad.exe"
"copy | copy=PF-ROW-COPIED"
"@
Set-Content "$PfRun\plain.ps1" -Value $plain -Encoding UTF8
Pf fence create --script "$PfRun\plain.ps1" --title Plain --rect '120,110,420,330' | Out-Null
$w = WaitRows 'Plain' 9
Shot 'plain-fence'
$before = Refusals

$clicked = ClickRow 'Plain' 0
for ($k = 0; $k -lt 30 -and (Explorers) -notmatch '说明'; $k++) { Start-Sleep -Milliseconds 200 }
Check 'open= of a folder opens it' ($clicked -and (Explorers) -match '说明') "clicked=$clicked explorer windows: $(Explorers)"
CloseWindows; ParkMouse

$clicked = ClickRow 'Plain' 1
$started = WaitNotepad 6
$title = NotepadTitle '说明文档'
Check 'open= of a .txt opens it in Notepad' ($clicked -and $started -and $title -match '说明文档') "clicked=$clicked title='$title'"
EndNotepads; ParkMouse

$t0 = Get-Date
$clicked = ClickRow 'Plain' 2
$edge = 0
for ($k = 0; $k -lt 40 -and -not $edge; $k++) { Start-Sleep -Milliseconds 250; $edge = @(Get-Process msedge -ErrorAction SilentlyContinue | Where-Object { $_.StartTime -ge $t0 }).Count }
Check 'open= of an https address opens the browser' ($clicked -and $edge -gt 0) "clicked=$clicked new msedge processes=$edge"
Start-Sleep -Seconds 2; EndBrowser; ParkMouse
Check 'a folder, a .txt and a web address were not refused' ((Refusals) -eq $before) "refusals=$((Refusals) - $before)"

$before = Refusals
$clicked = ClickRow 'Plain' 3
$started = WaitNotepad 4
Check 'open= of an .exe is refused' ($clicked -and -not $started -and (Refusals) -eq $before + 1) "clicked=$clicked notepad=$((Notepads).Count) refusals=$((Refusals) - $before)"
EndNotepads; ParkMouse

$before = Refusals
$clicked = ClickRow 'Plain' 4
$did = WaitFile $Mark 4
Check 'open= of a .bat is refused' ($clicked -and -not $did -and (Refusals) -eq $before + 1) "clicked=$clicked marker=$did refusals=$((Refusals) - $before)"
Remove-Item $Mark -ErrorAction SilentlyContinue; ParkMouse

$before = Refusals
$clicked = ClickRow 'Plain' 5
$started = WaitNotepad 4
Check 'open= of a .lnk is refused' ($clicked -and -not $started -and (Refusals) -eq $before + 1) "clicked=$clicked notepad=$((Notepads).Count) refusals=$((Refusals) - $before)"
EndNotepads; ParkMouse

$before = Refusals
$clicked = ClickRow 'Plain' 6
$did = WaitFile $Mark2 4
Check 'href=file:///... is refused' ($clicked -and -not $did -and (Refusals) -eq $before + 1) "clicked=$clicked marker=$did refusals=$((Refusals) - $before)"
Remove-Item $Mark2 -ErrorAction SilentlyContinue; ParkMouse

$before = Refusals
$clicked = ClickRow 'Plain' 7
$started = WaitNotepad 4
$said = RefusalText 'refused-plain-run'
Shot 'plain-run-refused'
Check 'run= is refused and the message shows' ($clicked -and -not $started -and (Refusals) -eq $before + 1 -and $said -match 'allow') "clicked=$clicked notepad=$((Notepads).Count) refusals=$((Refusals) - $before) message='$said'"
EndNotepads; ParkMouse

Set-Clipboard -Value 'nothing yet'
$clicked = ClickRow 'Plain' 8
Start-Sleep -Milliseconds 800
Check 'copy= still copies' ($clicked -and "$(Get-Clipboard)".Trim() -eq 'PF-ROW-COPIED') "clip='$(Get-Clipboard)'"
ParkMouse
Pf fence delete Plain | Out-Null

# ---- 3. A script whose header allows it: everything works as before
Case 'with the header line'
$allowed = @"
# $Allow
"run | run=notepad.exe"
"run with a file | run=notepad.exe param1=""$Txt"""
"batch | open=$Bat"
"shortcut | open=$Lnk"
"file address | href=file:///$($Bat2 -replace '\\', '/')"
"@
Set-Content "$PfRun\allowed.ps1" -Value $allowed -Encoding UTF8
Pf fence create --script "$PfRun\allowed.ps1" --title Allowed --rect '120,110,420,250' | Out-Null
$w = WaitRows 'Allowed' 5
Shot 'allowed-fence'
$before = Refusals

$clicked = ClickRow 'Allowed' 0
$started = WaitNotepad 6
Check 'run= starts the program' ($clicked -and $started) "clicked=$clicked notepad=$((Notepads).Count)"
EndNotepads; ParkMouse

$clicked = ClickRow 'Allowed' 1
$started = WaitNotepad 6
$title = NotepadTitle '说明文档'
Check 'run= with param1= passes the argument' ($clicked -and $started -and $title -match '说明文档') "clicked=$clicked title='$title'"
EndNotepads; ParkMouse

$clicked = ClickRow 'Allowed' 2
$did = WaitFile $Mark 6
Check 'open= of a .bat runs it' ($clicked -and $did) "clicked=$clicked marker=$did"
ParkMouse

$clicked = ClickRow 'Allowed' 3
$started = WaitNotepad 6
Check 'open= of a .lnk opens its target' ($clicked -and $started) "clicked=$clicked notepad=$((Notepads).Count)"
EndNotepads; ParkMouse

$clicked = ClickRow 'Allowed' 4
$did = WaitFile $Mark2 6
Check 'href=file:///... opens the file' ($clicked -and $did) "clicked=$clicked marker=$did"
Check 'nothing was refused' ((Refusals) -eq $before) "refusals=$((Refusals) - $before)"
ParkMouse
Pf fence delete Allowed | Out-Null
EndNotepads

# ---- 4. The current GitHub example on the same canned answer: forged titles are plain text
Case 'the current GitHub example'
$now = "$PfExtra\github.en.ps1"
Check 'the current GitHub example is in the extra folder' (Test-Path $now) $now
Set-Content "$PfRun\github-now.ps1" -Value (Get-Content -Raw -Encoding UTF8 $now) -Encoding UTF8
$wrap = @"
function Invoke-RestMethod { Get-Content -Raw -Encoding UTF8 '$PfRun\canned.json' | ConvertFrom-Json }
. '$PfRun\github-now.ps1'
"@
Set-Content "$PfRun\github-canned-now.ps1" -Value $wrap -Encoding UTF8
Pf fence create --script "$PfRun\github-canned-now.ps1" --title Cleaned --rect '120,110,720,270' | Out-Null
$w = WaitRows 'Cleaned' 6
$out = "$((FenceGet 'Cleaned').script.output)"
$issues = @($out -split "`n" | Where-Object { $_ -match '^#1[1-4]  ' })
# Text without a bar or a quote, then the script's own parameters: the link to that issue, the
# icon and the tooltip.
$odd = @($issues | Where-Object { $_ -notmatch '^#(\d+)  [^|"'']* \| href=https://github\.com/DayuanJiang/PecoFence/issues/\1 icon=\S+ tooltip="[^|"'']*"$' })
Check 'each issue is a row of plain text with the link to that issue' ($issues.Count -eq 4 -and $odd.Count -eq 0) ($out -replace "`n", ' / ')
$q = [char]0x2019
Check 'the forged title shows as text: the bar and the quote are replaced' ($out -match "#12  x ¦ run=notepad\.exe r=$q k=v \| href=") (($issues | Select-Object -Skip 1 -First 1) -join '')
Shot 'cleaned-fence'
$at = RowPt 'Cleaned' 3
if ($at) { [G]::MoveTo($at[0] - 3, $at[1]); Start-Sleep -Milliseconds 200; [G]::MoveTo($at[0], $at[1]); Start-Sleep -Milliseconds 2500 }
if ($w) { Crop 'cleaned-hover-line3' $w.l $w.t ($w.r - $w.l + 60) ($w.b - $w.t + 40) | Out-Null }
ParkMouse
$before = Refusals
$t0 = Get-Date
$clicked = ClickRow 'Cleaned' 3
$edge = 0
for ($k = 0; $k -lt 40 -and -not $edge; $k++) { Start-Sleep -Milliseconds 250; $edge = @(Get-Process msedge -ErrorAction SilentlyContinue | Where-Object { $_.StartTime -ge $t0 }).Count }
Check 'a click on it opens the browser, starts no program and is not refused' ($clicked -and $edge -gt 0 -and (Notepads).Count -eq 0 -and (Refusals) -eq $before) "clicked=$clicked new msedge processes=$edge notepad=$((Notepads).Count) refusals=$((Refusals) - $before)"
Start-Sleep -Seconds 2; EndBrowser; ParkMouse
Pf fence delete Cleaned | Out-Null
