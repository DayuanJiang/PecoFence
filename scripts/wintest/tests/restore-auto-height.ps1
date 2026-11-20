# A snapshot restore or an import puts a fence with auto height back where the snapshot (or the
# file) has it. Like a display change (feedback #194, monitor-sets-move.ps1), these sync the
# windows to the restored fences first, while the windows still stand where they were: an auto
# height pass that ran then could save that old place into the restored layout, and one that ran
# while a fence glided to its place could save where the glide began.
# Cases: moved and given more items, then restored; only moved (it glides back), then restored;
# moved and given more items, then the exported file imported. Runs on a 1600x900 box or guest.

function RaFence($t) {
  $j = $null
  try { $j = (Pf fence get $t) | ConvertFrom-Json } catch { return $null }
  if ($j.fence) { $j = $j.fence }
  return $j
}
function RaFmt($r) { if (-not $r) { return 'none' }; return "($($r.x),$($r.y) $($r.w)x$($r.h))" }
function RaSame($a, $b) { return $a -and $b -and [math]::Abs($a.x - $b.x) -le 4 -and [math]::Abs($a.y - $b.y) -le 4 -and [math]::Abs($a.w - $b.w) -le 4 -and [math]::Abs($a.h - $b.h) -le 4 }
# Waits until Snap's window has stood still for a second (a glide is over).
function RaSettle() {
  $last = ''; $same = 0
  for ($i = 0; $i -lt 40 -and $same -lt 4; $i++) {
    Start-Sleep -Milliseconds 250
    $now = RaFmt (RaFence 'Snap').windowRect
    if ($now -eq $last) { $same++ } else { $same = 0; $last = $now }
  }
}
function RaCheck($tag, $want) {
  trap { PfTrap $_; continue }
  RaSettle
  $f = RaFence 'Snap'
  Check "${tag}: Snap is back at its place and height" ((RaSame $f.windowRect $want) -and (RaSame $f.rect $want)) "window $(RaFmt $f.windowRect), saved $(RaFmt $f.rect), want $(RaFmt $want)"
  $n = @((PfJson item list --fence Snap)).Count
  Log "$tag items in Snap: $n"
}

Case 'restore with auto height: setup'
PfStart
ClearFences | Out-Null
$RaFiles = 1..18 | ForEach-Object { DeskFile ('Ra file {0:D2}.txt' -f $_) }
Start-Sleep -Seconds 3
Pf fence create --title Snap --rect '120,100,420,260' | Out-Null
Start-Sleep -Milliseconds 500
Pf item move --glob 'Ra file 0*' --to Snap | Out-Null
Pf fence set Snap autoHeight true | Out-Null
Start-Sleep -Seconds 2
$R0 = (RaFence 'Snap').windowRect
Log "Snap at start $(RaFmt $R0)"
$snap = PfJson snapshot save ra-before
$SnapId = if ($snap.snapshot) { $snap.snapshot.id } else { $snap.id }
Check 'a snapshot was saved' ([bool]$SnapId) "id $SnapId"
Pf config export "$PfRun\ra-export.json" | Out-Null
Check 'the configuration was exported' (Test-Path "$PfRun\ra-export.json") ''

function RaMoveAndFill() {
  trap { PfTrap $_; continue }
  Pf fence move Snap --x 900 --y 160 | Out-Null
  Start-Sleep -Milliseconds 500
  Pf item move --glob 'Ra file 1*' --to Snap | Out-Null
  Start-Sleep -Seconds 2
  Log "Snap moved and filled $(RaFmt (RaFence 'Snap').windowRect)"
}

Case 'restore with auto height: moved and filled, snapshot restored'
RaMoveAndFill
Pf snapshot restore $SnapId | Out-Null
RaCheck 'snapshot after a move and more items' $R0
Shot 'ra-restore-filled'

Case 'restore with auto height: only moved, snapshot restored (a glide back)'
Pf fence move Snap --x 900 --y 160 | Out-Null
Start-Sleep -Seconds 2
Log "Snap moved $(RaFmt (RaFence 'Snap').windowRect)"
Pf snapshot restore $SnapId | Out-Null
RaCheck 'snapshot after a move' $R0

Case 'restore with auto height: moved and filled, file imported'
RaMoveAndFill
Pf config import "$PfRun\ra-export.json" | Out-Null
RaCheck 'import after a move and more items' $R0
Shot 'ra-import-filled'

PfQuit
foreach ($p in $RaFiles) { Remove-Item -LiteralPath $p -Force -ErrorAction SilentlyContinue }
