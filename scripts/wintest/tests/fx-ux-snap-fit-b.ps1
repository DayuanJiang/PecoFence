# Fence fix round 2026-11-07, area ux-snap-fit, part B: where new icons land in 手动 order (U8,
# fb#125), files dropped on a shortcut to a folder (U9, fb#121), a portal folder deleted and made
# again (U20).

function FR($t) { return (FenceGet $t).windowRect }
function Order($t) { return ((@(PfJson item list --fence $t) | Where-Object { $_ -and $_.name } | ForEach-Object { $_.name }) -join ', ') }
function ClearFences() {
  foreach ($f in @(PfJson fence list)) { if ($f -and $f.title -and $f.kind -ne 'inbox') { Pf fence delete "$($f.id)" | Out-Null } }
  Start-Sleep -Seconds 1
}
$made = New-Object System.Collections.ArrayList
function Made($p) { [void]$made.Add($p); return $p }
$target = "$env:USERPROFILE\fx-u9-target"
$portal = "$env:USERPROFILE\fx-u20-portal"

Case 'setup'
CloseWindows
[G]::SystemParametersInfoW(0x25, 1, [NullString]::Value, 3) | Out-Null
foreach ($n in 'fxb Alpha.txt', 'fxb Delta.txt', 'fxb Kilo.txt') { Set-Content -LiteralPath (Made (Join-Path $PfDesk $n)) 'fx' }
Remove-Item -Recurse -Force $target, $portal -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $target | Out-Null
$lnk = Made (Join-Path $PfDesk 'fxb Target link.lnk')
$s = (New-Object -ComObject WScript.Shell).CreateShortcut($lnk); $s.TargetPath = $target; $s.Save()
PfStart -Script (DumpScript 250 12)
ClearFences
Pf fence create --title Docs --rect '480,120,300,330' | Out-Null
Pf item move "$PfDesk\fxb Alpha.txt" "$PfDesk\fxb Delta.txt" "$PfDesk\fxb Kilo.txt" --to Docs | Out-Null
Pf rule add --name 'fxb txt to Docs' --ext txt --to Docs --index 0 | Out-Null
Start-Sleep -Seconds 2
[G]::MoveTo(1100, 830)

Case 'U8 new files in a fence never arranged by hand (手动)'
$o0 = Order 'Docs'
Log "Docs sort $((FenceGet 'Docs').sort): [$o0]"
$steps = @()
$want = $o0
foreach ($n in 'fxb Golf', 'fxb Bravo', 'fxb Zulu') {
  Set-Content -LiteralPath (Made (Join-Path $PfDesk "$n.txt")) 'fx'; Start-Sleep -Seconds 3
  $want = "$want, $n"
  $steps += "+ ${n}: [$(Order 'Docs')]"
}
Shot 'u8-new-files-last'
$o1 = Order 'Docs'
Check 'each new file goes after the icons shown, the ones shown keep their places' ($o1 -eq $want) "$($steps -join ' | ')"
PfStart -Script (DumpScript 250 12)
Check 'the order stays after a restart' ((Order 'Docs') -eq $want) "[$(Order 'Docs')]"
Pf fence set Docs sort name | Out-Null; Start-Sleep -Milliseconds 800
$byName = Order 'Docs'
Pf fence set Docs sort manual | Out-Null; Start-Sleep -Milliseconds 800
Check 'sorted by name it follows the names; back to 手动 the order comes back' ($byName -eq 'fxb Alpha, fxb Bravo, fxb Delta, fxb Golf, fxb Kilo, fxb Zulu' -and (Order 'Docs') -eq $want) "name [$byName], manual [$(Order 'Docs')]"

Case 'U9 a file dropped on a shortcut to a folder'
Pf fence create --title Links --rect '900,120,300,250' | Out-Null
Pf fence create --title Src --rect '900,500,300,250' | Out-Null
Pf item move $lnk --to Links | Out-Null
Pf fence set Src sort name | Out-Null; Pf fence set Links sort name | Out-Null
Start-Sleep -Seconds 1
$ok = 0; $rows = @()
foreach ($run in 1, 2, 3) {
  $f = Made (Join-Path $PfDesk "fxb drop $run.txt")
  Set-Content -LiteralPath $f 'fx'; Start-Sleep -Seconds 2
  Pf item move $f --to Src | Out-Null; Start-Sleep -Seconds 1
  $p = ItemPt 'Src' "fxb drop $run"; $q = ItemPt 'Links' 'fxb Target link'
  DragTo $p[0] $p[1] $q[0] $q[1] 20
  Start-Sleep -Seconds 2
  if ($run -eq 1) { Shot 'u9-after-drop' }
  $in = Test-Path "$target\fxb drop $run.txt"; $desk = Test-Path $f
  if ($in -and -not $desk) { $ok++ }
  $rows += "run ${run}: in the folder $in, on the desktop $desk, fence '$(ItemFence "fxb drop $run.txt")'"
}
Check 'a file dropped on the shortcut moves into its folder (3 drops)' ($ok -eq 3) "$($rows -join ' | ')"
# Ctrl held: a copy, the file stays in its fence.
$f = Made (Join-Path $PfDesk 'fxb drop c.txt')
Set-Content -LiteralPath $f 'fx'; Start-Sleep -Seconds 2
Pf item move $f --to Src | Out-Null; Start-Sleep -Seconds 1
$p = ItemPt 'Src' 'fxb drop c'; $q = ItemPt 'Links' 'fxb Target link'
[G]::KeyDown(0x11); DragTo $p[0] $p[1] $q[0] $q[1] 20; [G]::KeyUp(0x11)
Start-Sleep -Seconds 2
Check 'with Ctrl held it is copied there and stays in its fence' ((Test-Path "$target\fxb drop c.txt") -and (Test-Path $f) -and (ItemFence 'fxb drop c.txt') -eq 'Src') "in the folder $(Test-Path "$target\fxb drop c.txt"), on the desktop $(Test-Path $f), fence '$(ItemFence 'fxb drop c.txt')'"
Check 'the shortcut itself stays in its fence' ((ItemFence 'fxb Target link.lnk') -eq 'Links') "fence '$(ItemFence 'fxb Target link.lnk')'"

Case 'U20 a portal folder deleted and made again'
New-Item -ItemType Directory -Force $portal | Out-Null
1..5 | ForEach-Object { Set-Content "$portal\f$_.txt" 'fx' }
Pf fence create --portal $portal --rect '480,520,300,250' | Out-Null
Start-Sleep -Seconds 2
Log "portal: $((FenceGet 'fx-u20-portal').itemCount) items"
$ok = 0; $rows = @()
foreach ($run in 1, 2, 3) {
  Remove-Item -Recurse -Force $portal
  $gone = -1
  for ($i = 0; $i -lt 40; $i++) { if ((FenceGet 'fx-u20-portal').itemCount -eq 0) { $gone = $i * 250; break }; Start-Sleep -Milliseconds 250 }
  Start-Sleep -Seconds 2
  New-Item -ItemType Directory -Force $portal | Out-Null
  1..5 | ForEach-Object { Set-Content "$portal\f$_.txt" 'fx' }
  $sw = [Diagnostics.Stopwatch]::StartNew(); $back = -1
  while ($sw.Elapsed.TotalSeconds -lt 70) { if ((FenceGet 'fx-u20-portal').itemCount -eq 5) { $back = $sw.ElapsedMilliseconds; break }; Start-Sleep -Milliseconds 250 }
  if ($back -ge 0 -and $back -lt 3000) { $ok++ }
  $rows += "run ${run}: empty after $gone ms; files back after $back ms"
}
Shot 'u20-portal-back'
Check 'the portal shows the folder again within 3 s (3 runs)' ($ok -eq 3) "$($rows -join ' | ')"

Case 'end'
PfQuit
foreach ($p in $made) { Remove-Item -LiteralPath $p -Force -ErrorAction SilentlyContinue }
Remove-Item -Recurse -Force $target, $portal -ErrorAction SilentlyContinue
