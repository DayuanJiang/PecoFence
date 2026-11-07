# #186 tool fences (小工具): a note made with 新建便签 (test-script new-note), typing in its
# editor, ticking a to-do with a click, notes off and on, a clock, a script fence and the weather
# example, and the note text after a restart.
Case 'setup'
CloseWindows
Set-Culture en-US
PfStart -Script @('sleep 9000', 'new-note 760 120', 'sleep 3600000')
function NoteFence() { foreach ($f in @((PfJson fence list).fences) + @(PfJson fence list)) { if ($f -and $f.tool -eq 'note') { return $f } }; return $null }
function FenceWindows() { return @(([L]::Tops('pecofence') -split "`n") | Where-Object { $_ -match '\| PecoFence\.Fence \|' }) }
function WaitScript($title, $sec) {
  for ($i = 0; $i -lt $sec; $i++) {
    $f = PfJson fence get $title
    $s = if ($f.fence) { $f.fence.script } else { $f.script }
    if ($s -and $s.outcome) { return $s }
    Start-Sleep -Seconds 1
  }
  return $s
}

Case 'new note'
$ed = WaitWin 'PecoFence.NoteEdit' 25
Check 'new-note opens the note editor' ($ed -ne [IntPtr]::Zero) "editor=$ed"
$r = Rect $ed
[G]::Click([int](($r.L + $r.R) / 2), [int]($r.T + 20)); Start-Sleep -Milliseconds 400
[G]::Type("[ ] pay the rent`n[ ] write the notes`nplain line"); Start-Sleep -Milliseconds 300
[G]::Press(0x1B); Start-Sleep -Milliseconds 800
$note = NoteFence
Check 'Esc closes the editor and saves the text' ($note.noteText -eq "[ ] pay the rent`n[ ] write the notes`nplain line") ("noteText=" + ($note.noteText -replace "`n", ' / '))
Shot 't01-note'

Case 'tick a to-do'
$wr = $note.windowRect
# The first to-do's box: 14 DIP from the left edge, below the 36 DIP title row (100 % box).
[G]::Click($wr.x + 22, $wr.y + 36 + 11); Start-Sleep -Milliseconds 700
[G]::MoveTo(500, 700)
$note = NoteFence
Check 'a click on the box ticks the first to-do' ($note.noteText -like '`[x`] pay the rent*') ("noteText=" + ($note.noteText -replace "`n", ' / '))
Check 'the click does not open the editor' ((WaitWin 'PecoFence.NoteEdit' 1) -eq [IntPtr]::Zero)

Case 'notes off and on'
$before = (FenceWindows).Count
Pf settings set tools.enabled false | Out-Null; Start-Sleep -Seconds 1
$off = (FenceWindows).Count
Pf settings set tools.enabled true | Out-Null; Start-Sleep -Seconds 1
$on = (FenceWindows).Count
$note = NoteFence
Check 'switched off, the note window goes' ($off -eq $before - 1) "windows: before=$before off=$off"
Check 'switched on again, it comes back with its text' ($on -eq $before -and $note.noteText -like '`[x`] pay the rent*') "on=$on noteText=$($note.noteText -replace "`n", ' / ')"

Case 'clock'
Pf fence create --tool clock --title Clock --rect '1100,120,260,140' | Out-Null; Start-Sleep -Seconds 2
$c = PfJson fence get Clock
$tool = if ($c.fence) { $c.fence.tool } else { $c.tool }
Check 'a clock fence is created' ($tool -eq 'clock') "tool=$tool"
Check 'its window shows' (@(FenceWindows | Where-Object { $_ -like '*| Clock |*' }).Count -eq 1) ((FenceWindows) -join ' ; ')
Shot 't04-clock'

Case 'script fence'
Set-Content C:\pf\run\hello.ps1 -Value 'Write-Output "hello from a script"; Write-Output "line two"' -Encoding UTF8
Pf fence create --script C:\pf\run\hello.ps1 --title Hello --rect '1100,300,300,160' | Out-Null
$s = WaitScript 'Hello' 30
Check 'the script runs and its output shows' ($s.outcome -eq 'done' -and $s.output -like '*hello from a script*') "outcome=$($s.outcome) output=$($s.output)"
Shot 't05-script'

Case 'restart keeps everything'
PfQuit
PfStart -Script @('sleep 4000', 'new-script-example 1100 500', 'sleep 3600000')
Start-Sleep -Seconds 3
$note = NoteFence
Check 'the note text survives a restart' ($note.noteText -like '`[x`] pay the rent*') ("noteText=" + ($note.noteText -replace "`n", ' / '))
Check 'the clock survives a restart' ($null -ne (PfFence 'Clock'))

Case 'weather example'
$w = $null
foreach ($f in @((PfJson fence list).fences) + @(PfJson fence list)) { if ($f -and $f.tool -eq 'script' -and $f.title -ne 'Hello') { $w = $f } }
Check 'the weather example is added' ($null -ne $w) "title=$($w.title)"
if ($w) {
  $s = WaitScript $w.title 40
  Check 'the weather example prints something (needs the internet)' ($s.outcome -eq 'done' -and $s.output) "outcome=$($s.outcome) output=$(("$($s.output)" -split "`n")[0])"
}
Shot 't07-weather'

Case 'cleanup'
PfQuit
