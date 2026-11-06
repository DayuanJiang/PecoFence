# scripts/wintest/user.ps1: the user's own PecoFence (installed or portable, no test instance),
# setup and uninstall by their buttons, the tray icon and menu, the welcome notice. Loaded by
# lib.ps1. Set $script:CliExe to the copy under test (default: the installed one).

$script:CliExe = "$env:LOCALAPPDATA\Programs\PecoFence\pecofence-cli.exe"

# A user's PecoFence (installed or portable, no PECOFENCE_INSTANCE): its CLI, its log, starting it
# the way Explorer does, waiting for it to quit.
function Ipf {
  $a = @($args | ForEach-Object { $_ })
  $r = & $script:CliExe @a 2>&1 | Out-String
  Log ("user cli " + ($a -join ' ') + " -> " + $r.Trim())
  return $r
}
# JSON only when the CLI printed some (an error goes to stderr as JSON too, with a prefix).
function IpfJson { $r = (Ipf @args).Trim(); if ($r -match '^[\[\{]') { return ($r | ConvertFrom-Json) }; return $null }
function UserLog() { return "$env:LOCALAPPDATA\PecoFence\pecofence.log" }
function StartUser($exePath) {
  Start-Process explorer.exe "`"$exePath`""
  $h = WaitWin 'PecoFence.Control' 30 -Hidden
  Start-Sleep -Seconds 4
  return $h
}
function WaitNoPf($sec = 15) {
  for ($i = 0; $i -lt $sec * 5; $i++) { if (-not (Get-Process pecofence -ErrorAction SilentlyContinue)) { return $true }; Start-Sleep -Milliseconds 200 }
  return $false
}

# A real click on a child button by its caption (Inno wizard, message boxes: UI Automation finds
# no Inno buttons on this box), the text of a message box.
function ClickButton($top, $regex) {
  $list = [W]::Buttons($top)
  foreach ($line in ($list -split "`n")) {
    if (-not $line) { continue }
    $p = $line -split "`t"
    if ($p[1] -match $regex -and $p[2] -eq 'True') {
      $r = $p[3] -split ','
      [G]::Click([int](([int]$r[0] + [int]$r[2]) / 2), [int](([int]$r[1] + [int]$r[3]) / 2))
      Log "button '$($p[1])' in $([W]::Cls($top))"
      return $p[1]
    }
  }
  Log ("no button '$regex' in $([W]::Cls($top)), only: " + (($list -split "`n" | ForEach-Object { ($_ -split "`t")[1] }) -join ' | '))
  return $null
}
function DlgText($hwnd) { return [W]::Statics($hwnd).Trim() }
# Message boxes of a setup or uninstaller (any #32770 that is not ours, Explorer's or the job's).
function SetupBoxes() { return @([W]::Tops('#32770') | Where-Object { [W]::ProcOf($_) -notmatch '^(pecofence|explorer|powershell|notepad|conhost)' }) }

# Inno setup run interactively: language OK, the "currently running" prompt (OnRunning closes the
# app, then OK), Next / Install / Finish. Returns the exit code and the pages seen.
function Wizard($exe, $logName, [scriptblock]$OnRunning) {
  $seen = New-Object System.Collections.ArrayList
  $p = Start-Process $exe -ArgumentList "/LOG=`"$PfRun\$logName`"" -PassThru
  $t0 = Get-Date
  while (-not $p.HasExited -and ((Get-Date) - $t0).TotalSeconds -lt 120) {
    Start-Sleep -Milliseconds 800
    $lang = [G]::FindWindowW('TSelectLanguageForm', [NullString]::Value)
    if ($lang -ne [IntPtr]::Zero -and [G]::IsWindowVisible($lang)) { [void]$seen.Add("language $(ClickButton $lang '^OK$')"); continue }
    $boxes = @(SetupBoxes)
    if ($boxes.Count) {
      $txt = DlgText $boxes[0]
      if ($txt -match 'currently running') {
        [void]$seen.Add('app running prompt'); Shot "$logName-running"
        try { & $OnRunning } catch { Log "closing the app failed: $($_.Exception.Message)" }
        Start-Sleep -Seconds 1; [void](ClickButton $boxes[0] '^OK$'); continue
      }
      [void]$seen.Add("box '$txt' $(ClickButton $boxes[0] '^(OK|Yes)$')"); continue
    }
    $wiz = [G]::FindWindowW('TWizardForm', [NullString]::Value)
    if ($wiz -ne [IntPtr]::Zero -and [G]::IsWindowVisible($wiz)) {
      $n = ClickButton $wiz '^(Next|Install|Finish)'
      if ($n) { [void]$seen.Add($n); Start-Sleep -Milliseconds 700 }
    }
  }
  $p.WaitForExit(5000) | Out-Null
  return [pscustomobject]@{ code = $p.ExitCode; exited = $p.HasExited; seen = ($seen -join ' > ') }
}
# The Inno uninstaller run interactively (it hands over to a copy in %TEMP% and exits at once).
function Uninstall($unins, [scriptblock]$OnRunning) {
  $seen = New-Object System.Collections.ArrayList
  Start-Process $unins
  $t0 = Get-Date; $doneAt = $null
  while (((Get-Date) - $t0).TotalSeconds -lt 150) {
    Start-Sleep -Milliseconds 800
    $boxes = @(SetupBoxes)
    if ($boxes.Count) {
      $txt = DlgText $boxes[0]
      if ($txt -match 'currently running') { [void]$seen.Add('app running prompt'); Shot 'uninstall-running'; try { & $OnRunning } catch { Log "closing the app failed: $($_.Exception.Message)" }; Start-Sleep -Seconds 1; [void](ClickButton $boxes[0] '^OK$'); continue }
      if ($txt -match 'sure') { [void]$seen.Add("confirm $(ClickButton $boxes[0] '^Yes$')"); continue }
      [void]$seen.Add("box '$txt' $(ClickButton $boxes[0] '^OK$')")
      if ($txt -match 'removed') { $doneAt = Get-Date }
      continue
    }
    if ($doneAt -and ((Get-Date) - $doneAt).TotalSeconds -gt 3) { break }
  }
  return ($seen -join ' > ')
}
function SilentUninstall($unins) {
  Start-Process $unins -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART' -Wait
  for ($i = 0; $i -lt 60 -and (Test-Path (Join-Path (Split-Path $unins) 'pecofence.exe')); $i++) { Start-Sleep -Milliseconds 500 }
}
function SilentSetup($exe, $logName) {
  $p = Start-Process $exe -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', "/LOG=$PfRun\$logName" -PassThru
  $p.WaitForExit(180000) | Out-Null
  return $p.ExitCode
}

# The tray icon (on the taskbar or under Show Hidden Icons) and its menu, with the real mouse.
# A UI Automation element with a real rectangle on screen (hidden tray icons have NaN rects).
function OnScreen($el) { $r = $el.Current.BoundingRectangle; return ((-not $el.Current.IsOffscreen) -and $r.Width -gt 0 -and $r.Height -gt 0 -and -not [double]::IsNaN($r.X)) }
function TrayIconRect() {
  $any = [Windows.Automation.Condition]::TrueCondition
  for ($try = 0; $try -lt 3; $try++) {
    $tray = [Windows.Automation.AutomationElement]::FromHandle([G]::FindWindowW('Shell_TrayWnd', [NullString]::Value))
    $all = @($tray.FindAll([Windows.Automation.TreeScope]::Descendants, $any))
    $pf = $all | Where-Object { $_.Current.Name -match '^PecoFence' -and $_.Current.Name -notmatch 'running window' -and (OnScreen $_) } | Select-Object -First 1
    if ($pf) { return $pf.Current.BoundingRectangle }
    $ov = [G]::FindWindowW('TopLevelWindowForOverflowXamlIsland', [NullString]::Value)
    if ($ov -eq [IntPtr]::Zero -or -not [G]::IsWindowVisible($ov)) {
      $chev = $all | Where-Object { $_.Current.Name -match 'hidden icons|chevron' -and (OnScreen $_) } | Select-Object -First 1
      if ($chev) {
        $r = $chev.Current.BoundingRectangle
        [G]::Click([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2))
        $ov = WaitWin 'TopLevelWindowForOverflowXamlIsland' 3
      }
    }
    if ($ov -ne [IntPtr]::Zero -and [G]::IsWindowVisible($ov)) {
      $o = @([Windows.Automation.AutomationElement]::FromHandle($ov).FindAll([Windows.Automation.TreeScope]::Descendants, $any))
      $pf = $o | Where-Object { $_.Current.Name -match '^PecoFence' -and (OnScreen $_) } | Select-Object -First 1
      if ($pf) { return $pf.Current.BoundingRectangle }
    }
    Start-Sleep -Seconds 1
  }
  Log ("tray: no PecoFence icon; taskbar names: " + (($all | Where-Object { $_.Current.Name } | ForEach-Object { "$($_.Current.Name) [$($_.Current.ControlType.ProgrammaticName)]" }) -join ' | '))
  return $null
}
function TrayMenu() {
  [G]::MoveTo(700, 450); Start-Sleep -Milliseconds 200
  $r = TrayIconRect
  if (-not $r) { return [IntPtr]::Zero }
  [G]::RClick([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2))
  return (MenuWin 5)
}
function TrayPick($regex) {
  $m = TrayMenu
  if ($m -eq [IntPtr]::Zero) { Log "tray menu did not open for '$regex'"; return $null }
  Start-Sleep -Milliseconds 300
  $script:LastTrayMenu = MenuTexts $m
  $t = MenuPick $regex $m
  Start-Sleep -Milliseconds 600
  return $t
}

# The welcome notice's close box (x, 24 DIP in from the top right corner).
function CloseNotice() {
  $n = [G]::FindWindowW('PecoFence.Notice', [NullString]::Value)
  if ($n -eq [IntPtr]::Zero -or -not [G]::IsWindowVisible($n)) { return $true }
  $r = Rect $n
  [G]::Click($r.R - 24, $r.T + 24); Start-Sleep -Milliseconds 300
  [G]::MoveTo(700, 450)
  return (WaitGoneWin 'PecoFence.Notice' 3)
}
