# A native resize cannot take over the HWND while an upward roll is changing its height.
trap { PfTrap $_; continue }

$animationsWereEnabled = [Anim]::Get()
try {
  [Anim]::Set($true)
  PfStart
  Pf fence create --title RollResize --rect '400,340,360,320' | Out-Null
  Pf fence set RollResize autoHeight false | Out-Null
  Pf fence set RollResize expandUpward true | Out-Null
  $hwnd = [G]::FindWindowW('PecoFence.Fence', 'RollResize')
  $open = Rect $hwnd
  $openHeight = $open.B - $open.T
  Pf fence roll RollResize | Out-Null
  Start-Sleep -Milliseconds 450
  $rolled = Rect $hwnd
  $rolledHeight = $rolled.B - $rolled.T
  Check 'The test fence is rolled to its title' ($rolledHeight -gt 0 -and $rolledHeight -lt $openHeight - 10) "rolled=$rolledHeight open=$openHeight"

  # Start the command without waiting for the CLI response. Sample a stable native rectangle
  # between the rolled and open heights so a late hit test cannot pass as an in-flight one.
  $unroll = Start-Process "$PfApp\pecofence-cli.exe" -ArgumentList '--instance', $PfInst, 'fence', 'unroll', 'RollResize' -PassThru -WindowStyle Hidden
  $deadline = [Diagnostics.Stopwatch]::StartNew()
  $hit = $null
  $sampleHeight = $null
  $sample = $null
  while ($deadline.ElapsedMilliseconds -lt 5000) {
    $r = Rect $hwnd
    $height = $r.B - $r.T
    if ($height -gt $rolledHeight + 5 -and $height -lt $openHeight - 5) {
      $edgeHit = NcHit $hwnd ([int](($r.L + $r.R) / 2)) ($r.T + 1)
      $after = Rect $hwnd
      if ($r.L -eq $after.L -and $r.T -eq $after.T -and $r.R -eq $after.R -and $r.B -eq $after.B) {
        $hit = $edgeHit
        $sampleHeight = $height
        $sample = $r
        break
      }
    }
    Start-Sleep -Milliseconds 1
  }
  Check 'The fence is expanding upward from the rolled title' ($sample -and $sample.T -lt $rolled.T -and $sample.B -eq $rolled.B) "rolled top=$($rolled.T) bottom=$($rolled.B) sampled top=$($sample.T) bottom=$($sample.B)"
  Check 'An expanding fence does not offer its top resize edge' ($hit -eq 1) "hit=$hit height=$sampleHeight rolled=$rolledHeight open=$openHeight"
  Check 'The unroll command succeeded' ($unroll.WaitForExit(5000) -and $unroll.ExitCode -eq 0)
  Start-Sleep -Milliseconds 450
  $landed = Rect $hwnd
  $settledHit = NcHit $hwnd ([int](($landed.L + $landed.R) / 2)) ($landed.T + 1)
  Check 'The top resize edge returns after the fence is open' ($settledHit -eq 12 -and $landed.B - $landed.T -eq $openHeight) "hit=$settledHit height=$($landed.B - $landed.T)"
} finally {
  [Anim]::Set($animationsWereEnabled)
}
