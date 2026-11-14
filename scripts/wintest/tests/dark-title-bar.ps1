# Issue #219, Windows 10 1809 / Server 2019 (build 17763): in dark mode the settings window kept
# a white title bar over its dark page. The dark title bar attribute of DWM has the number 19
# on Windows 10 1809 to 1909 and 20 from 2004 on; PecoFence asked only for 20.
# Windows 11 paints the caption in the page's colour, Windows 10 in its own dark caption colour:
# either way the strip between the title and the buttons is dark, and in light mode it is light.
# Needs a box of that build to fail without the fix: a `manual` line in suite.txt.
function CaptionGrey($sw) {
  $r = Rect $sw
  $b = [Px]::Grab($r.L + 320, $r.T + 10, $r.R - $r.L - 520, 12)
  $m = MeanRgb $b; $b.Dispose()
  return [int](($m[0] + $m[1] + $m[2]) / 3)
}

Case 'settings window, dark mode'
CloseWindows
PfStart
Pf settings set theme dark | Out-Null
Start-Sleep -Seconds 1
$ok = PfSettings
$sw = [G]::FindWindowW('PecoFence.Settings', [NullString]::Value)
Check 'the settings window opens' ($ok -and $sw -ne [IntPtr]::Zero) "window=$sw"
Start-Sleep -Seconds 2
$dark = CaptionGrey $sw
Shot 'settings-dark'
Check 'its title bar is dark in dark mode' ($dark -lt 90) "mean grey of the title bar strip: $dark"

Case 'settings window, light mode'
Pf settings set theme light | Out-Null
Start-Sleep -Seconds 2
$light = CaptionGrey $sw
Shot 'settings-light'
Check 'and light in light mode' ($light -gt 180) "mean grey of the title bar strip: $light"
CloseWindows
