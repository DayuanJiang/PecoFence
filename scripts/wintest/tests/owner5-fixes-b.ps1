# Owner round 2026-11-12, fifth batch, package ux5-fixes-b: what two readers of the merged
# branch found (notes/fix/ux5-read-b and ux5-read-c). Parts (run.sh --var Part=...):
#   S  a configuration from somebody else: its scripts start only after the user's own turn of
#      the 小工具 switch. Not by a new note or clock (page, command line), and not when a
#      snapshot brings the imported layout back later. The script is harmless: a .cmd that
#      writes a marker file into the job's run folder.
#   D  a snapshot deleted on the settings page and brought back with 撤销 keeps the note
#      texts and script paths of its fences
# S and D drive the settings page over CDP: run them on a debug build.
# Runs on a 1600x900 box at 100 %.
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'SD' }
trap { PfTrap $_; continue }

function O5Tools() { return (PfJson settings get).tools }
function O5WaitFile($path, $sec) { for ($i = 0; $i -lt $sec * 2 -and -not (Test-Path $path); $i++) { Start-Sleep -Milliseconds 500 }; return (Test-Path $path) }
function O5Fences($title) { return @(PfJson fence list | Where-Object { $_ -and $_.title -eq $title }).Count }
function O5Newest() { return @(Snaps | Sort-Object { [long]$_.ts } -Descending)[0] }
# A first start on "another computer", in Chinese, with a note of the user's own.
function O5Another() {
  PfFresh
  Start-Sleep -Seconds 3
  Pf settings set language zh-CN | Out-Null
  CloseNotice | Out-Null
  Pf fence create --tool note --title '我的便签' --text '自己的' --rect '420,140,300,200' | Out-Null
  Start-Sleep -Seconds 1
}

Case 'setup'
CloseWindows
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'

# ---- S: imported scripts wait for the user's own turn of the switch
function O5Import() {
  trap { PfTrap $_; continue }
  Case 'S0 somebody''s configuration with a script fence'
  $Mark = "$PfRun\o5-script-ran.txt"
  $S = "$PfRun\o5-script.cmd"
  Set-Content $S -Value "@echo ran>> `"$Mark`"`r`n@echo hello" -Encoding ASCII
  PfStart -Lang zh-CN
  Start-Sleep -Seconds 3
  Pf fence create --script $S --title 'Runner' --rect '420,440,300,160' | Out-Null
  Check 'the script fence runs its script on the computer it was made on' (O5WaitFile $Mark 15) ''
  $theirs = "$PfRun\o5-theirs.json"; Pf config export $theirs | Out-Null

  Case 'S1 imported on another computer, then a clock made by the command line'
  O5Another
  $rep = PfJson config import $theirs; Start-Sleep -Seconds 3
  Remove-Item $Mark -ErrorAction SilentlyContinue
  Check 'the import holds the script: 小工具 is off and the reply counts it' ((O5Tools).enabled -eq $false -and $rep.scriptsHeld -eq 1) "tools.enabled=$((O5Tools).enabled) reply=$($rep | ConvertTo-Json -Compress)"
  $out = "$(Pf fence create --tool clock --title '时钟' --rect '820,140,240,140')"
  Start-Sleep -Seconds 6
  Check 'S1 a clock made by the command line starts no imported script' (-not (Test-Path $Mark)) "script ran=$(Test-Path $Mark) tools.enabled=$((O5Tools).enabled)"
  Check 'S1 the command line says which script waits' ($out -match 'o5-script\.cmd' -and (O5Fences '时钟') -eq 0) $out.Trim()

  # A first start again: a script that ran a moment ago in this run is not due for its
  # interval, whoever turns the switch.
  Case 'S2 the same file, then 新建便签 and the clock''s 添加 on the settings page'
  O5Another
  Pf config import $theirs | Out-Null; Start-Sleep -Seconds 3
  Remove-Item $Mark -ErrorAction SilentlyContinue
  Check 'settings open over CDP' (PfSettings)
  PageClick 'nav a[data-page=tools]'; Start-Sleep -Milliseconds 600
  $held = "$(Js "(document.getElementById('toolsHeld') || { textContent: '' }).textContent")"
  Shot 's2-tools-page-held'
  Check 'S2 the card of the 小工具 switch lists the script that waits' ($held -match 'o5-script\.cmd') "card: [$held]"
  PageClick '#newNote'; Start-Sleep -Seconds 5
  $toast = PageToast
  Check 'S2 新建便签 starts no imported script' (-not (Test-Path $Mark)) "script ran=$(Test-Path $Mark) tools.enabled=$((O5Tools).enabled) notice=[$($toast.text)]"
  Check 'S2 the page says that the scripts wait for the switch' ($toast.text -match '脚本' -and (O5Fences '便签') -eq 0) "notice=[$($toast.text)] new notes=$(O5Fences '便签')"
  if (-not (Test-Path $Mark)) {
    PageClick '#newClock'; Start-Sleep -Seconds 5
    Check 'S2 the clock''s 添加 starts no imported script' (-not (Test-Path $Mark)) "script ran=$(Test-Path $Mark) tools.enabled=$((O5Tools).enabled)"
  }

  Case 'S3 the user turns 小工具 on himself'
  if (-not (O5Tools).enabled) { PageClick '[data-bind="tools.enabled"]' }
  Check 'S3 the script starts with the user''s own turn of the switch' (O5WaitFile $Mark 15) "tools.enabled=$((O5Tools).enabled)"
  $hidden = Js "(document.getElementById('toolsHeld') || { hidden: true }).hidden"
  Check 'S3 the card no longer lists a waiting script' ($hidden -eq $true) "hidden=$hidden"
  PageClick '#newNote'; Start-Sleep -Seconds 2
  [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  Check 'S3 新建便签 makes a note again' ((O5Fences '便签') -ge 1) "notes titled 便签: $(O5Fences '便签')"
  $json = [IO.File]::ReadAllText((PfJson status).configPath)
  Check 'S3 the saved configuration holds no script any more' ($json -notmatch '"held"') ''

  Case 'S4 the import undone, 小工具 on for the own note, then the snapshot of the imported layout restored'
  O5Another
  Pf config import $theirs | Out-Null; Start-Sleep -Seconds 3
  $undo = O5Newest
  Pf snapshot restore "$($undo.id)" | Out-Null; Start-Sleep -Seconds 3
  Remove-Item $Mark -ErrorAction SilentlyContinue
  Pf settings set tools.enabled true | Out-Null; Start-Sleep -Seconds 6
  Check 'S4 the user''s own layout is back and 小工具 on: the imported script does not run' ((O5Fences 'Runner') -eq 0 -and (O5Fences '我的便签') -eq 1 -and -not (Test-Path $Mark)) "Runner=$(O5Fences 'Runner') own note=$(O5Fences '我的便签') script ran=$(Test-Path $Mark)"
  $imp = O5Newest
  $rep = PfJson snapshot restore "$($imp.id)"; Start-Sleep -Seconds 2
  Check 'S4 the snapshot taken before that restore brings the imported script fence back' ((O5Fences 'Runner') -eq 1) "snapshot '$($imp.name)'"
  # The minute housekeeping starts every script that is due.
  $ran = O5WaitFile $Mark 80
  Check 'S4 the script that came back with the snapshot does not run' (-not $ran) "script ran=$ran within 80 s; tools.enabled=$((O5Tools).enabled)"
  Check 'S4 小工具 is off again and the reply counts the script' ((O5Tools).enabled -eq $false -and $rep.scriptsHeld -eq 1) "tools.enabled=$((O5Tools).enabled) reply=$($rep | ConvertTo-Json -Compress)"
  Pf settings set tools.enabled true | Out-Null
  Check 'S4 it starts with the user''s own turn of the switch' (O5WaitFile $Mark 15) ''
  PfQuit
  Remove-Item $Mark, $S, $theirs -ErrorAction SilentlyContinue
}

# ---- D: a deleted snapshot's undo
function O5Deleted() {
  trap { PfTrap $_; continue }
  Case 'D0 a note and a script fence that only one snapshot still has'
  $Hello = "$PfRun\o5-hello.cmd"; Set-Content $Hello -Value '@echo hello' -Encoding ASCII
  PfFresh
  Start-Sleep -Seconds 3
  Pf settings set language zh-CN | Out-Null
  CloseNotice | Out-Null
  Pf fence create --tool note --title '买菜' --text '[ ] 牛奶 和 鸡蛋' --rect '420,140,300,200' | Out-Null
  Pf fence create --script $Hello --title '问候' --rect '420,380,300,160' | Out-Null
  Start-Sleep -Seconds 2
  $mine = "$((PfJson snapshot save '有便签和脚本').snapshot.id)"
  Pf fence delete '买菜' | Out-Null; Pf fence delete '问候' | Out-Null
  # The command line's automatic snapshots have the two fences too: they go, so that the saved
  # snapshot is the only place the note and the script are still wanted.
  foreach ($s in @(Snaps | Where-Object { "$($_.id)" -ne $mine })) { Pf snapshot delete "$($s.id)" | Out-Null }
  Start-Sleep -Seconds 2
  Check 'one snapshot is left and the two fences are gone' (@(Snaps).Count -eq 1 -and (O5Fences '买菜') -eq 0 -and (O5Fences '问候') -eq 0) "snapshots=$(@(Snaps).Count)"
  Check 'settings open over CDP' (PfSettings)
  PageClick 'nav a[data-page=layout]'; Start-Sleep -Milliseconds 600

  Case 'D1 the snapshot deleted on the page, a second later 撤销'
  PageClick "#snapList .card[data-row-id=`"$mine`"] [data-act=del]"
  # The configuration is saved 0.8 s after the delete.
  Start-Sleep -Milliseconds 2500
  Check 'the snapshot is gone and the notice offers 撤销' (@(Snaps).Count -eq 0 -and (Js "!document.getElementById('toastUndo').hidden")) "snapshots=$(@(Snaps).Count) notice=[$((PageToast).text)]"
  PageClick '#toastUndo'; Start-Sleep -Seconds 1
  Check '撤销 puts the snapshot back in the list' (@(Snaps).Count -eq 1) "snapshots=$(@(Snaps).Count)"
  Pf snapshot restore $mine | Out-Null; Start-Sleep -Seconds 2
  $note = "$((FenceGet '买菜').noteText)"; $path = "$((FenceGet '问候').script.path)"
  Shot 'd1-restored'
  Check 'D1 the note restored from that snapshot has its text' ($note -match '牛奶') "note text: [$note]"
  Check 'D1 the script fence restored from that snapshot has its script' ($path -match 'o5-hello') "script: [$path]"
  PfQuit
  Remove-Item $Hello -ErrorAction SilentlyContinue
}

if ($Part -match 'S') { O5Import }
if ($Part -match 'D') { O5Deleted }

Case 'cleanup'
if (PfAlive) { PfQuit }
CloseWindows
