# End-to-end smoke test for pecofence-cli against an isolated PecoFence instance.
#
#   powershell -File scripts/cli-smoke.ps1                 # uses target/debug binaries
#   powershell -File scripts/cli-smoke.ps1 -Profile release
#
# Copies pecofence.exe, pecofence-watchdog.exe, pecofence-cli.exe and WebView2Loader.dll into a
# scratch folder under .cache/, points LOCALAPPDATA/APPDATA there, and starts the app as
# `PECOFENCE_INSTANCE=clitest --portable --no-hide-icons`, so the user's configuration and
# desktop icons are never touched. Then it drives the instance with the CLI and asserts the JSON
# replies and exit codes. Exit code 0 = all checks passed.
param(
    [ValidateSet("debug", "release")] [string]$Profile = "debug",
    [int]$LifetimeMs = 120000
)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$bin = Join-Path $root "target/$Profile"
foreach ($f in "pecofence.exe", "pecofence-watchdog.exe", "pecofence-cli.exe") {
    if (-not (Test-Path (Join-Path $bin $f))) {
        throw "missing $bin/$f - build first: cargo build -p pecofence -p pecofence-watchdog -p pecofence-cli" + $(if ($Profile -eq "release") { " --release" } else { "" })
    }
}

$instance = "clitest-" + [guid]::NewGuid().ToString("N").Substring(0, 8)
$stage = Join-Path $root ".cache/$instance"
New-Item -ItemType Directory -Path $stage | Out-Null
foreach ($f in "pecofence.exe", "pecofence-watchdog.exe", "pecofence-cli.exe") {
    Copy-Item (Join-Path $bin $f) $stage
}
Copy-Item (Join-Path $root "third_party/webview2/WebView2Loader.x64.dll") (Join-Path $stage "WebView2Loader.dll")
# Redirect the app's data folders for this process only; restored in the finally block so a
# dot-sourced or `&`-invoked run does not leave the caller's session pointing at a scratch dir.
$savedEnv = @{ PECOFENCE_INSTANCE = $env:PECOFENCE_INSTANCE; LOCALAPPDATA = $env:LOCALAPPDATA; APPDATA = $env:APPDATA; RUST_LOG = $env:RUST_LOG }
$env:PECOFENCE_INSTANCE = $instance
$env:LOCALAPPDATA = Join-Path $stage "local-appdata"
$env:APPDATA = Join-Path $stage "roaming-appdata"
$env:RUST_LOG = "info"
$cli = Join-Path $stage "pecofence-cli.exe"

$script:failures = 0
$script:checks = 0

function Invoke-Cli {
    # Windows PowerShell treats native stderr as an error under ErrorActionPreference=Stop, so
    # run the CLI as a child process with both streams redirected.
    param([string[]]$CliArgs)
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $cli
    $psi.Arguments = ((@("--compact") + $CliArgs) | ForEach-Object { '"' + ($_ -replace '"', '\"') + '"' }) -join ' '
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.StandardOutputEncoding = [Text.Encoding]::UTF8
    $psi.StandardErrorEncoding = [Text.Encoding]::UTF8
    $p = [System.Diagnostics.Process]::Start($psi)
    $outTask = $p.StandardOutput.ReadToEndAsync()
    $errTask = $p.StandardError.ReadToEndAsync()
    $p.WaitForExit()
    [pscustomobject]@{ Code = $p.ExitCode; Out = $outTask.Result.Trim(); Err = $errTask.Result.Trim() }
}

function Check {
    param([string]$Name, [bool]$Ok, [string]$Detail = "")
    $script:checks++
    if ($Ok) { Write-Host "  ok   $Name" }
    else { $script:failures++; Write-Host "  FAIL $Name  $Detail" -ForegroundColor Red }
}

function Json($text) {
    # Windows PowerShell returns a JSON array as ONE object; enumerate so @(Json ...) gets elements.
    if ([string]::IsNullOrWhiteSpace($text)) { return $null }
    try { $v = ConvertFrom-Json -InputObject $text } catch { return $null }
    if ($v -is [System.Array]) { foreach ($e in $v) { $e } } else { $v }
}

# 0. Nothing running yet: exit 3 / not_running.
$r = Invoke-Cli @("--instance", "nosuch-$instance", "status")
$e = Json $r.Err
Check "not running -> exit 3" ($r.Code -eq 3) "code=$($r.Code) err=$($r.Err)"
Check "not running -> error.code not_running" ($e.error.code -eq "not_running") $r.Err

# 1. Start the isolated instance.
$app = Start-Process -FilePath (Join-Path $stage "pecofence.exe") -WorkingDirectory $stage `
    -ArgumentList @("--portable", "--no-hide-icons", "--exit-after", "$LifetimeMs") -PassThru
try {
    $deadline = (Get-Date).AddSeconds(30)
    do {
        Start-Sleep -Milliseconds 250
        $r = Invoke-Cli @("status")
    } while ($r.Code -ne 0 -and (Get-Date) -lt $deadline)
    $status = Json $r.Out
    Check "status reachable" ($r.Code -eq 0) "code=$($r.Code) err=$($r.Err)"
    Check "status.instance = $instance" ($status.instance -eq $instance) $r.Out
    Check "status.appVersion present" (-not [string]::IsNullOrEmpty($status.appVersion)) $r.Out

    # Concurrency: several clients at once must all succeed (no pipe-instance gap).
    $jobs = 1..4 | ForEach-Object {
        Start-Job -ScriptBlock {
            param($cli, $inst, $la, $ra)
            $env:PECOFENCE_INSTANCE = $inst; $env:LOCALAPPDATA = $la; $env:APPDATA = $ra
            & $cli --compact status | Out-Null
            $LASTEXITCODE
        } -ArgumentList $cli, $instance, $env:LOCALAPPDATA, $env:APPDATA
    }
    $codes = $jobs | Wait-Job | Receive-Job
    $jobs | Remove-Job
    Check "4 concurrent status calls all exit 0" (($codes | Where-Object { $_ -ne 0 }).Count -eq 0) "codes=$codes"

    $r = Invoke-Cli @("monitor", "list")
    $mons = Json $r.Out
    Check "monitor list has >= 1 monitor" ($r.Code -eq 0 -and @($mons).Count -ge 1) $r.Out
    $work = @($mons)[0].workArea
    $x = $work.x + 100; $y = $work.y + 100

    $r = Invoke-Cli @("fence", "list")
    $before = @(Json $r.Out)
    Check "fence list ok" ($r.Code -eq 0) $r.Err

    # 2. Create / get / move / resize.
    $title = "CliTest $instance"
    $r = Invoke-Cli @("fence", "create", "--title", $title, "--rect", "$x,$y,400,300")
    $c = Json $r.Out
    Check "fence create changed" ($r.Code -eq 0 -and $c.changed -eq $true) "$($r.Out) $($r.Err)"
    $id = $c.fence.id
    Check "fence create returns id" (-not [string]::IsNullOrEmpty($id)) $r.Out
    Check "fence create rect w=400" ($c.fence.rect.w -eq 400) $r.Out

    $r = Invoke-Cli @("fence", "get", $title)
    $g = Json $r.Out
    Check "fence get by title" ($r.Code -eq 0 -and $g.id -eq $id) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "get", $id.Substring(0, 8))
    Check "fence get by id prefix" ($r.Code -eq 0 -and (Json $r.Out).id -eq $id) "$($r.Out) $($r.Err)"

    $r = Invoke-Cli @("fence", "move", $id, "--x", "$($x + 200)")
    $m = Json $r.Out
    Check "fence move --x" ($r.Code -eq 0 -and $m.changed -eq $true -and $m.fence.rect.x -eq ($x + 200)) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "resize", $id, "--w", "500")
    $m = Json $r.Out
    Check "fence resize --w" ($r.Code -eq 0 -and $m.fence.rect.w -eq 500) "$($r.Out) $($r.Err)"

    # 3. Options: idempotency and validation.
    $r = Invoke-Cli @("fence", "set", $id, "layout", "list")
    Check "fence set layout list" ($r.Code -eq 0 -and (Json $r.Out).fence.layout -eq "list") "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "set", $id, "iconSize", "64")
    Check "fence set iconSize 64 changed" ($r.Code -eq 0 -and (Json $r.Out).changed -eq $true) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "set", $id, "iconSize", "64")
    Check "fence set iconSize 64 again -> changed:false" ($r.Code -eq 0 -and (Json $r.Out).changed -eq $false) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "set", $id, "iconSize", "50")
    $e = Json $r.Err
    Check "fence set iconSize 50 -> invalid_value" ($r.Code -eq 1 -and $e.error.code -eq "invalid_value") "code=$($r.Code) $($r.Err)"
    $r = Invoke-Cli @("fence", "set", $id, "nosuchprop", "1")
    $e = Json $r.Err
    Check "fence set unknown prop -> invalid_value with allowed" ($e.error.code -eq "invalid_value" -and $e.error.details.allowed.Count -gt 5) $r.Err

    $r = Invoke-Cli @("fence", "roll", $id)
    Check "fence roll" ($r.Code -eq 0 -and (Json $r.Out).fence.rolledUp -eq $true) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "roll", $id)
    Check "fence roll again -> changed:false" ($r.Code -eq 0 -and (Json $r.Out).changed -eq $false) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "unroll", $id)
    Check "fence unroll" ($r.Code -eq 0 -and (Json $r.Out).fence.rolledUp -eq $false) "$($r.Out) $($r.Err)"

    # Issue #125: with expandUpward a fence opens upward from its title row where it fits
    # there, else downward; the title row stays where it is, and switching never moves it.
    Start-Sleep -Milliseconds 800  # the unroll above lands first
    # The icon size change fitted the empty fence to one row (74 px at 100 %), less than the
    # title row + 40 px an expand opens to at least, so the rect would not stay exact. 200 px
    # at y 200 fits on the screen on either side of its title row, down to a 1024x768 screen;
    # 400 px wide it stays clear of the first-run fences on the right, which it would stop
    # below where they cannot slide further up.
    $m = Json (Invoke-Cli @("fence", "move", $id, "--rect", "$($x + 200),$($work.y + 200),400,200")).Out
    $rect = $m.fence.rect
    Start-Sleep -Milliseconds 400
    $r = Invoke-Cli @("fence", "set", $id, "expandUpward", "true")
    $u = Json $r.Out
    Check "fence set expandUpward true: nothing moves" ($r.Code -eq 0 -and $u.fence.expandUpward -eq $true -and $u.fence.windowRect.y -eq $rect.y -and $u.fence.windowRect.h -eq $rect.h) "$($r.Out) $($r.Err)"
    Invoke-Cli @("fence", "roll", $id) | Out-Null
    Start-Sleep -Milliseconds 800
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    $row = $w.windowRect
    $rowBottom = $row.y + $row.h
    Check "expandUpward roll: the title row stays where it was" ($w.rolledUp -eq $true -and $row.y -eq $rect.y -and $row.h -lt $rect.h) "$($w | ConvertTo-Json -Compress -Depth 4)"
    Check "expandUpward roll: the saved rect stands on the row" (($w.rect.y + $w.rect.h) -eq $rowBottom -and $w.rect.h -eq $rect.h -and $w.bottomAnchored -eq $true) "$($w.rect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "unroll", $id) | Out-Null
    Start-Sleep -Milliseconds 800
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "expandUpward unroll: opens upward, where it fits above the row" ($w.rolledUp -eq $false -and ($w.windowRect.y + $w.windowRect.h) -eq $rowBottom -and $w.windowRect.h -eq $rect.h) "$($w.windowRect | ConvertTo-Json -Compress)"
    # fence resize --h moves the top edge of a fence that opened upward, not its bottom one.
    Invoke-Cli @("fence", "resize", $id, "--h", "$($rect.h - 40)") | Out-Null
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "expandUpward resize --h keeps the bottom edge of a fence that opened upward" ($w.rect.h -eq ($rect.h - 40) -and ($w.rect.y + $w.rect.h) -eq $rowBottom) "$($w.rect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "resize", $id, "--h", "$($rect.h)") | Out-Null
    Invoke-Cli @("fence", "roll", $id) | Out-Null
    Start-Sleep -Milliseconds 800
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "expandUpward roll: back at the same title row" ($w.rolledUp -eq $true -and $w.windowRect.y -eq $row.y -and $w.windowRect.h -eq $row.h) "$($w.windowRect | ConvertTo-Json -Compress)"
    # Switched while rolled, the title row stays where it is: the saved rect hangs below it, or
    # stands on it again.
    $r = Invoke-Cli @("fence", "set", $id, "expandUpward", "false")
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "expandUpward off while rolled: title row stays, rect hangs below it" ($r.Code -eq 0 -and $w.expandUpward -eq $false -and $w.windowRect.y -eq $row.y -and $w.rect.y -eq $row.y -and $w.rect.h -eq $rect.h) "$($w | ConvertTo-Json -Compress -Depth 4)"
    $r = Invoke-Cli @("fence", "set", $id, "expandUpward", "true")
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "expandUpward on while rolled: title row stays, rect stands on it" ($r.Code -eq 0 -and $w.windowRect.y -eq $row.y -and ($w.rect.y + $w.rect.h) -eq $rowBottom -and $w.rect.h -eq $rect.h) "$($w | ConvertTo-Json -Compress -Depth 4)"
    # Switched during a roll, the roll lands on the title row it came from.
    Invoke-Cli @("fence", "unroll", $id) | Out-Null
    Start-Sleep -Milliseconds 800
    Invoke-Cli @("fence", "set", $id, "expandUpward", "false") | Out-Null
    Invoke-Cli @("fence", "roll", $id) | Out-Null
    Invoke-Cli @("fence", "set", $id, "expandUpward", "true") | Out-Null
    Start-Sleep -Milliseconds 800
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "expandUpward switched during a roll: lands on the same title row" ($w.rolledUp -eq $true -and $w.windowRect.y -eq $row.y -and ($w.rect.y + $w.rect.h) -eq $rowBottom -and $w.rect.h -eq $rect.h) "$($w | ConvertTo-Json -Compress -Depth 4)"
    Invoke-Cli @("fence", "set", $id, "expandUpward", "false") | Out-Null
    Invoke-Cli @("fence", "unroll", $id) | Out-Null
    Start-Sleep -Milliseconds 800
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "expandUpward off: opens downward from the row, like any fence" ($w.rolledUp -eq $false -and $w.windowRect.y -eq $rect.y -and $w.windowRect.h -eq $rect.h) "$($w.windowRect | ConvertTo-Json -Compress)"

    $r = Invoke-Cli @("fence", "rename", $id, "$title renamed")
    Check "fence rename" ($r.Code -eq 0 -and (Json $r.Out).fence.title -eq "$title renamed") "$($r.Out) $($r.Err)"

    # 3b. Review follow-ups: colour values, --string, --all, inbox alias, rect validation.
    $r = Invoke-Cli @("fence", "set", $id, "tint", "#FF8800")
    Check "fence set tint #FF8800" ($r.Code -eq 0 -and (Json $r.Out).fence.tint -eq "#FF8800") "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "set", $id, "titleOnHover", "hover")
    Check "fence set titleOnHover hover" ($r.Code -eq 0 -and (Json $r.Out).fence.titleOnHover -eq "hover") "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "set", $id, "titleOnHover", "true")
    $e = Json $r.Err
    Check "fence set titleOnHover true -> invalid_value" ($r.Code -eq 1 -and $e.error.code -eq "invalid_value" -and ($e.error.details.allowed -contains "always")) "code=$($r.Code) $($r.Err)"
    $r = Invoke-Cli @("fence", "set", $id, "titleOnHover", "default")
    Check "fence set titleOnHover default" ($r.Code -eq 0 -and (Json $r.Out).fence.titleOnHover -eq "default") "$($r.Out) $($r.Err)"
    # Issue #40: title-row-only tint, a tab bar colour of its own, extra-large titles.
    $r = Invoke-Cli @("fence", "set", $id, "tintTitleOnly", "true")
    Check "fence set tintTitleOnly true" ($r.Code -eq 0 -and (Json $r.Out).fence.tintTitleOnly -eq $true) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "set", $id, "tabColor", "#E74856")
    Check "fence set tabColor #E74856" ($r.Code -eq 0 -and (Json $r.Out).fence.tabColor -eq "#E74856") "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "set", $id, "titleSize", "extraLarge")
    Check "fence set titleSize extraLarge" ($r.Code -eq 0 -and (Json $r.Out).fence.titleSize -eq "extraLarge") "$($r.Out) $($r.Err)"
    foreach ($p in @(@("tintTitleOnly", "false"), @("tabColor", "null"), @("titleSize", "normal"))) {
        Invoke-Cli @("fence", "set", $id, $p[0], $p[1]) | Out-Null
    }
    # A title row at the bottom prefers to open upward. Moved between the top and the
    # bottom, an expandUpward that follows the title bar turns with it and the rolled row stays.
    Invoke-Cli @("fence", "roll", $id) | Out-Null
    Start-Sleep -Milliseconds 800
    $row = (Json (Invoke-Cli @("fence", "get", $id)).Out).windowRect
    $rowBottom = $row.y + $row.h
    $r = Invoke-Cli @("fence", "set", $id, "titlePosition", "bottom")
    $f = (Json $r.Out).fence
    Check "fence set titlePosition bottom -> expandUpward follows it on" ($r.Code -eq 0 -and $f.titlePosition -eq "bottom" -and $f.expandUpward -eq $true) "$($r.Out) $($r.Err)"
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "titlePosition bottom while rolled: the row stays, the rect stands on it" ($w.windowRect.y -eq $row.y -and ($w.rect.y + $w.rect.h) -eq $rowBottom -and $w.bottomAnchored -eq $true) "$($w | ConvertTo-Json -Compress -Depth 4)"
    Invoke-Cli @("fence", "unroll", $id) | Out-Null
    Start-Sleep -Milliseconds 800
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "a bottom title row opens upward where it fits above the row" ($w.rolledUp -eq $false -and ($w.windowRect.y + $w.windowRect.h) -eq $rowBottom -and $w.windowRect.h -eq $rect.h) "$($w.windowRect | ConvertTo-Json -Compress)"
    $r = Invoke-Cli @("fence", "set", $id, "titleAlign", "right")
    Check "fence set titleAlign right" ($r.Code -eq 0 -and (Json $r.Out).fence.titleAlign -eq "right") "$($r.Out) $($r.Err)"
    Invoke-Cli @("fence", "roll", $id) | Out-Null
    Start-Sleep -Milliseconds 800
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "a bottom title row rolls back to the same row" ($w.rolledUp -eq $true -and $w.windowRect.y -eq $row.y -and $w.windowRect.h -eq $row.h) "$($w.windowRect | ConvertTo-Json -Compress)"
    $r = Invoke-Cli @("fence", "set", $id, "titlePosition", "top")
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "titlePosition top -> expandUpward follows it off, the row stays, the rect hangs below it" ($r.Code -eq 0 -and $w.expandUpward -eq $false -and $w.windowRect.y -eq $row.y -and $w.rect.y -eq $row.y -and $w.bottomAnchored -eq $false) "$($w | ConvertTo-Json -Compress -Depth 4)"
    # Every move sets expandUpward to follow the new edge: a round trip ends at the edge's own.
    Invoke-Cli @("fence", "set", $id, "expandUpward", "true") | Out-Null
    Invoke-Cli @("fence", "set", $id, "titlePosition", "bottom") | Out-Null
    $f = (Json (Invoke-Cli @("fence", "set", $id, "titlePosition", "top")).Out).fence
    Check "expandUpward on at the top, to the bottom and back: off" ($f.expandUpward -eq $false) "$($f | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "set", $id, "titlePosition", "bottom") | Out-Null
    Invoke-Cli @("fence", "set", $id, "expandUpward", "false") | Out-Null
    Invoke-Cli @("fence", "set", $id, "titlePosition", "top") | Out-Null
    $f = (Json (Invoke-Cli @("fence", "set", $id, "titlePosition", "bottom")).Out).fence
    Check "expandUpward off at the bottom, to the top and back: on" ($f.expandUpward -eq $true) "$($f | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "set", $id, "titlePosition", "top") | Out-Null
    # On a side the title bar is a column the fence rolls up into, opening away from it.
    $r = Invoke-Cli @("fence", "set", $id, "titlePosition", "right")
    $f = (Json $r.Out).fence
    Check "fence set titlePosition right -> title column at the right of the rect" ($r.Code -eq 0 -and $f.titlePosition -eq "right" -and ($f.windowRect.x + $f.windowRect.w) -eq ($f.rect.x + $f.rect.w) -and $f.windowRect.w -lt $f.rect.w) "$($r.Out) $($r.Err)"
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "a right column with room on its left reports rightAnchored" ($w.rightAnchored -eq $true -and $w.bottomAnchored -eq $false) "$($w | ConvertTo-Json -Compress -Depth 4)"
    # fence resize --w keeps the right edge of a column that sits at the right of its rect.
    $was = $w
    Invoke-Cli @("fence", "resize", $id, "--w", "$($was.rect.w - 40)") | Out-Null
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "resize --w keeps the right edge of a rightAnchored column" ($w.rect.w -eq ($was.rect.w - 40) -and ($w.rect.x + $w.rect.w) -eq ($was.rect.x + $was.rect.w) -and ($w.windowRect.x + $w.windowRect.w) -eq ($was.windowRect.x + $was.windowRect.w) -and $w.rightAnchored -eq $true) "before=$($was.rect | ConvertTo-Json -Compress) $($was.windowRect | ConvertTo-Json -Compress) after=$($w.rect | ConvertTo-Json -Compress) $($w.windowRect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "resize", $id, "--w", "$($was.rect.w)") | Out-Null
    $r = Invoke-Cli @("fence", "set", $id, "titlePosition", "left")
    $f = (Json $r.Out).fence
    Check "fence set titlePosition left -> the rolled column stays where it was" ($r.Code -eq 0 -and $f.titlePosition -eq "left" -and ($f.windowRect.x + $f.windowRect.w) -eq ($was.windowRect.x + $was.windowRect.w) -and $f.windowRect.w -lt $f.rect.w) "$($r.Out) $($r.Err)"
    # A fence 50 px right of the column leaves too little room there: the column opens to the
    # left, as wide as it fits there, and rolls back into the same column.
    $col = $f.windowRect
    $r = Invoke-Cli @("fence", "create", "--title", "Beside $instance", "--rect", "$($col.x + $col.w + 50),$($f.rect.y),200,$($f.rect.h)")
    $beside = (Json $r.Out).fence.id
    Start-Sleep -Milliseconds 400
    Invoke-Cli @("fence", "unroll", $id) | Out-Null
    Start-Sleep -Milliseconds 800
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "a left column with too little room on its right opens to the left" ($w.rolledUp -eq $false -and $w.rightAnchored -eq $true -and ($w.windowRect.x + $w.windowRect.w) -eq ($col.x + $col.w) -and $w.windowRect.w -gt ($col.w + 50) -and $w.windowRect.w -le $rect.w) "col=$($col | ConvertTo-Json -Compress) window=$($w.windowRect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "roll", $id) | Out-Null
    Start-Sleep -Milliseconds 800
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "the column rolls back into the same spot, with the fence's full width saved" ($w.rolledUp -eq $true -and $w.windowRect.x -eq $col.x -and $w.windowRect.w -eq $col.w -and $w.rect.w -eq $rect.w) "col=$($col | ConvertTo-Json -Compress) now=$($w.windowRect | ConvertTo-Json -Compress) rect=$($w.rect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "delete", $beside) | Out-Null
    $r = Invoke-Cli @("fence", "set", $id, "collapsedSize", "full")
    $f = (Json $r.Out).fence
    Check "fence set collapsedSize full -> the column keeps the fence's height" ($r.Code -eq 0 -and $f.collapsedSize -eq "full" -and -not $f.collapsedShrinks -and $f.windowRect.h -eq $f.rect.h) "$($r.Out) $($r.Err)"
    # A title row shrinks only when asked to. A short title keeps the shrunk row narrower than
    # the fence at any scale.
    Invoke-Cli @("fence", "rename", $id, "Edges") | Out-Null
    Invoke-Cli @("fence", "set", $id, "titlePosition", "top") | Out-Null
    $r = Invoke-Cli @("fence", "set", $id, "collapsedSize", "fit")
    $f = (Json $r.Out).fence
    Check "fence set collapsedSize fit -> a rolled title row aligned right shrinks to the right" ($r.Code -eq 0 -and $f.collapsedShrinks -and ($f.windowRect.x + $f.windowRect.w) -eq ($f.rect.x + $f.rect.w) -and $f.windowRect.w -lt $f.rect.w) "$($r.Out) $($r.Err)"
    # The shrunk row fits its title again after a rename or a larger title size.
    $narrow = $f.windowRect
    Invoke-Cli @("fence", "rename", $id, "Edges and corners") | Out-Null
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "a rolled shrunk row grows with a longer title" ($w.windowRect.w -gt $narrow.w -and ($w.windowRect.x + $w.windowRect.w) -eq ($w.rect.x + $w.rect.w)) "before=$($narrow | ConvertTo-Json -Compress) after=$($w.windowRect | ConvertTo-Json -Compress)"
    $wide = $w.windowRect
    Invoke-Cli @("fence", "set", $id, "titleSize", "large") | Out-Null
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "a rolled shrunk row grows with a larger title size" ($w.windowRect.w -gt $wide.w -or $w.windowRect.w -eq $w.rect.w) "before=$($wide | ConvertTo-Json -Compress) after=$($w.windowRect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "set", $id, "titleSize", "normal") | Out-Null
    Invoke-Cli @("fence", "rename", $id, "Edges") | Out-Null
    # Centred, it shrinks around the fence's centre.
    Invoke-Cli @("fence", "set", $id, "titleAlign", "center") | Out-Null
    $w = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "a rolled shrunk row aligned center keeps the fence's centre" ($w.windowRect.w -lt $w.rect.w -and [math]::Abs(($w.windowRect.x * 2 + $w.windowRect.w) - ($w.rect.x * 2 + $w.rect.w)) -le 2) "rect=$($w.rect | ConvertTo-Json -Compress) window=$($w.windowRect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "set", $id, "titleAlign", "right") | Out-Null
    $r = Invoke-Cli @("fence", "set", $id, "collapsedSize", "default")
    $f = (Json $r.Out).fence
    Check "fence set collapsedSize default -> a rolled title row is as wide as the fence" ($r.Code -eq 0 -and $f.collapsedSize -eq "default" -and $f.windowRect.w -eq $f.rect.w) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "set", $id, "sideTabText", "horizontal")
    Check "fence set sideTabText horizontal" ($r.Code -eq 0 -and (Json $r.Out).fence.sideTabText -eq "horizontal") "$($r.Out) $($r.Err)"
    Invoke-Cli @("fence", "set", $id, "sideTabText", "vertical") | Out-Null
    foreach ($p in @(@("titlePosition", "top"), @("titleAlign", "default"))) {
        Invoke-Cli @("fence", "set", $id, $p[0], $p[1]) | Out-Null
    }
    Invoke-Cli @("fence", "unroll", $id) | Out-Null
    Start-Sleep -Milliseconds 800
    # Issue #39: the inbox cannot be deleted, but it can hide while empty.
    $r = Invoke-Cli @("fence", "delete", "inbox")
    Check "fence delete inbox -> refused with the hideWhenEmpty hint" ($r.Code -eq 1 -and $r.Err -match "hideWhenEmpty") "code=$($r.Code) $($r.Err)"
    $r = Invoke-Cli @("fence", "set", "inbox", "hideWhenEmpty", "true")
    Check "fence set inbox hideWhenEmpty true" ($r.Code -eq 0 -and (Json $r.Out).fence.hideWhenEmpty -eq $true) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "set", "inbox", "hideWhenEmpty", "false")
    Check "fence set inbox hideWhenEmpty false" ($r.Code -eq 0 -and (Json $r.Out).fence.hideWhenEmpty -eq $false) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "set", $id, "title", "2024", "--string")
    Check "fence set title 2024 --string" ($r.Code -eq 0 -and (Json $r.Out).fence.title -eq "2024") "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "set", "--all", "locked", "false")
    $b = Json $r.Out
    Check "fence set --all returns results" ($r.Code -eq 0 -and @($b.results).Count -ge 1 -and $null -ne $b.changed) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "get", "inbox")
    Check "fence get inbox alias" ($r.Code -eq 0 -and (Json $r.Out).kind -eq "inbox") "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "move", $id, "--rect", "999999,999999,300,200")
    Check "off-screen rect -> invalid_value" ($r.Code -eq 1 -and (Json $r.Err).error.code -eq "invalid_value") "code=$($r.Code) $($r.Err)"
    $r = Invoke-Cli @("fence", "create", "--title", "x", "--rect", "2147483000,0,300,200")
    Check "overflowing rect -> invalid_value" ($r.Code -eq 1 -and (Json $r.Err).error.code -eq "invalid_value") "code=$($r.Code) $($r.Err)"
    $r = Invoke-Cli @("settings", "set", "snapping.gapPx", "-4")
    # -4 must reach the app as a value, not as a flag; the gap is 0 to 64, so it is refused.
    Check "settings set negative number reaches the range check" ($r.Code -eq 1 -and (Json $r.Err).error.code -eq "validation_failed" -and $r.Err -match "gapPx -4") "code=$($r.Code) $($r.Err)"
    $r = Invoke-Cli @("rule", "add", "--name", "Empty", "--to", "inbox", "--json", "[]")
    Check "rule add without conditions rejected" ($r.Code -ne 0) "$($r.Out)"

    # 4. Settings.
    # keepIconsHidden only decides what exit does with icons this instance hid (it hides none).
    $r = Invoke-Cli @("settings", "set", "keepIconsHidden", "true")
    Check "settings set keepIconsHidden true" ($r.Code -eq 0 -and (Json $r.Out).settings.keepIconsHidden -eq $true) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("settings", "set", "keepIconsHidden", "false")
    Check "settings set keepIconsHidden false" ($r.Code -eq 0 -and (Json $r.Out).settings.keepIconsHidden -eq $false) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("settings", "get", "peek.enabled")
    Check "settings get peek.enabled is bool" ($r.Code -eq 0 -and ($r.Out -eq "true" -or $r.Out -eq "false")) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("settings", "set", "peek.enabled", "false")
    Check "settings set peek.enabled false" ($r.Code -eq 0 -and (Json $r.Out).settings.peek.enabled -eq $false) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("settings", "set", "peek.enabled", "false")
    Check "settings set same value -> changed:false" ($r.Code -eq 0 -and (Json $r.Out).changed -eq $false) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("settings", "set", "peek.nope", "1")
    $e = Json $r.Err
    Check "settings set unknown path -> invalid_path" ($r.Code -eq 1 -and $e.error.code -eq "invalid_path") "code=$($r.Code) $($r.Err)"
    $r = Invoke-Cli @("settings", "set", "iconSize", "abc")
    $e = Json $r.Err
    Check "settings set wrong type -> validation_failed" ($r.Code -eq 1 -and $e.error.code -eq "validation_failed") "code=$($r.Code) $($r.Err)"

    # 5. Rules.
    $r = Invoke-Cli @("rule", "add", "--name", "Smoke PDFs", "--ext", "pdf", "--to", $id)
    $ra = Json $r.Out
    Check "rule add" ($r.Code -eq 0 -and $ra.changed -eq $true -and $ra.rule.name -eq "Smoke PDFs") "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("rule", "list")
    $rl = Json $r.Out
    Check "rule list contains the rule" ($r.Code -eq 0 -and @($rl.list | Where-Object { $_.name -eq "Smoke PDFs" }).Count -eq 1) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("rule", "disable", "Smoke PDFs")
    Check "rule disable" ($r.Code -eq 0 -and (Json $r.Out).rule.enabled -eq $false) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("rule", "apply")
    Check "rule apply" ($r.Code -eq 0) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("rule", "remove", "Smoke PDFs")
    Check "rule remove" ($r.Code -eq 0 -and (Json $r.Out).changed -eq $true) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("rule", "remove", "Smoke PDFs")
    Check "rule remove again -> rule_not_found" ($r.Code -eq 1 -and (Json $r.Err).error.code -eq "rule_not_found") "code=$($r.Code) $($r.Err)"

    # 6. Snapshots.
    $r = Invoke-Cli @("snapshot", "save", "smoke $instance")
    $s = Json $r.Out
    Check "snapshot save" ($r.Code -eq 0 -and $s.snapshot.name -eq "smoke $instance") "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("snapshot", "list")
    Check "snapshot list contains it" ($r.Code -eq 0 -and @((Json $r.Out) | Where-Object { $_.id -eq $s.snapshot.id }).Count -eq 1) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("snapshot", "delete", $s.snapshot.id)
    Check "snapshot delete" ($r.Code -eq 0) "$($r.Out) $($r.Err)"

    # 7. Items (read-only) and visibility.
    $r = Invoke-Cli @("item", "list")
    Check "item list ok" ($r.Code -eq 0) "$($r.Err)"
    $r = Invoke-Cli @("fence", "hide-all")
    Check "hide-all" ($r.Code -eq 0) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "show-all")
    Check "show-all" ($r.Code -eq 0 -and (Json $r.Out).changed -eq $true) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "show-all")
    Check "show-all again -> changed:false" ($r.Code -eq 0 -and (Json $r.Out).changed -eq $false) "$($r.Out) $($r.Err)"

    # 7a. Item metadata, rename and events, all inside a scratch folder portal (never the desktop).
    $portalDir = Join-Path $stage "portal"
    New-Item -ItemType Directory -Path $portalDir | Out-Null
    Set-Content -LiteralPath (Join-Path $portalDir "Report.pdf") -Value "smoke" -Encoding Ascii
    Set-Content -LiteralPath (Join-Path $portalDir "notes.txt") -Value "smoke" -Encoding Ascii
    $r = Invoke-Cli @("fence", "create", "--portal", $portalDir, "--title", "SmokePortal $instance")
    $pc = Json $r.Out
    $portalId = $pc.fence.id
    Check "portal create" ($r.Code -eq 0 -and $pc.fence.kind -eq "portal") "$($r.Out) $($r.Err)"
    Start-Sleep -Milliseconds 800
    $r = Invoke-Cli @("item", "list", "--fence", $portalId)
    $items = @(Json $r.Out)
    $pdf = $items | Where-Object { $_.fileName -eq "Report.pdf" } | Select-Object -First 1
    Check "portal item list has both files" ($r.Code -eq 0 -and $items.Count -eq 2) "$($r.Out) $($r.Err)"
    Check "item metadata: kind documents, ext .pdf, size, modified, created" ($pdf.kind -eq "documents" -and $pdf.ext -eq ".pdf" -and $pdf.size -gt 0 -and $pdf.modified -gt 0 -and $pdf.created -gt 0 -and $pdf.assignedBy -eq "portal") ($pdf | ConvertTo-Json -Compress)
    $r = Invoke-Cli @("item", "list", "--fence", $portalId, "--ext", "txt")
    Check "item list --ext filters client-side" ($r.Code -eq 0 -and @(Json $r.Out).Count -eq 1 -and @(Json $r.Out)[0].fileName -eq "notes.txt") "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("item", "list", "--fence", $portalId, "--kind", "images")
    Check "item list --kind with no match is an empty list" ($r.Code -eq 0 -and $r.Out -eq "[]") "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("item", "rename", (Join-Path $portalDir "Report.pdf"), "2026-09 electricity bill")
    $rn = Json $r.Out
    Check "item rename keeps the extension" ($r.Code -eq 0 -and $rn.changed -eq $true -and (Test-Path (Join-Path $portalDir "2026-09 electricity bill.pdf")) -and -not (Test-Path (Join-Path $portalDir "Report.pdf"))) "$($r.Out) $($r.Err)"
    Check "item rename returns the renamed item" ($rn.item.fileName -eq "2026-09 electricity bill.pdf") "$($r.Out)"
    $r = Invoke-Cli @("item", "rename", (Join-Path $portalDir "notes.txt"), "notes.md", "--keep-ext", "false")
    Check "item rename --keep-ext false renames verbatim" ($r.Code -eq 0 -and (Test-Path (Join-Path $portalDir "notes.md"))) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("item", "rename", (Join-Path $portalDir "notes.md"), "2026-09 electricity bill.pdf", "--keep-ext", "false")
    Check "item rename onto an existing name -> invalid_value" ($r.Code -eq 1 -and (Json $r.Err).error.code -eq "invalid_value") "code=$($r.Code) $($r.Err)"
    $r = Invoke-Cli @("item", "rename", (Join-Path $portalDir "notes.md"), "bad:name")
    Check "item rename with a forbidden character -> invalid_value" ($r.Code -eq 1 -and (Json $r.Err).error.code -eq "invalid_value") "code=$($r.Code) $($r.Err)"
    $r = Invoke-Cli @("rule", "apply", "--dry-run")
    $dr = Json $r.Out
    Check "rule apply --dry-run changes nothing and lists moves" ($r.Code -eq 0 -and $dr.dryRun -eq $true -and $dr.changed -eq $false -and $null -ne $dr.moves) "$($r.Out) $($r.Err)"

    # Events: a watcher waiting on the portal sees the file that appears in it.
    $wpsi = New-Object System.Diagnostics.ProcessStartInfo
    $wpsi.FileName = $cli
    $wpsi.Arguments = "--compact watch --fence $portalId --events item.added --once"
    $wpsi.UseShellExecute = $false
    $wpsi.RedirectStandardOutput = $true
    $wpsi.RedirectStandardError = $true
    $wpsi.StandardOutputEncoding = [Text.Encoding]::UTF8
    $wpsi.StandardErrorEncoding = [Text.Encoding]::UTF8
    $watch = [System.Diagnostics.Process]::Start($wpsi)
    $wOut = $watch.StandardOutput.ReadToEndAsync()
    $wErr = $watch.StandardError.ReadToEndAsync()
    Start-Sleep -Milliseconds 700
    Set-Content -LiteralPath (Join-Path $portalDir "arrived.png") -Value "smoke" -Encoding Ascii
    $watchDone = $watch.WaitForExit(8000)
    if (-not $watchDone) { $watch.Kill() }
    $watch.WaitForExit()
    $watchText = $wOut.Result.Trim()
    $ev = Json $watchText
    Check "watch --once exits on the first event" ($watchDone -and $watch.ExitCode -eq 0) "exit=$($watch.ExitCode) out=$watchText err=$($wErr.Result.Trim())"
    Check "watch event is item.added for arrived.png in the portal" ($ev.event -eq "item.added" -and $ev.item.fileName -eq "arrived.png" -and $ev.item.fence -eq $portalId -and $ev.seq -ge 1) $watchText
    $r = Invoke-Cli @("watch", "--events", "nope")
    Check "watch with an unknown event -> invalid_value" ($r.Code -eq 1 -and (Json $r.Err).error.code -eq "invalid_value") "code=$($r.Code) $($r.Err)"
    # 7a''. Dogfooding follow-ups: --fields/--ascii, fit + fence fit, adjusted, placement,
    # item move --dry-run and collision skips between two scratch portals, tab rect, snapshot counts.
    $r = Invoke-Cli @("fence", "list", "--fields", "id,title")
    $fl = @(Json $r.Out)
    $keys = @($fl | ForEach-Object { ($_.PSObject.Properties | ForEach-Object { $_.Name }) -join "," } | Sort-Object -Unique)
    Check "fence list --fields keeps only id,title" ($r.Code -eq 0 -and $fl.Count -ge 2 -and $keys.Count -eq 1 -and $keys[0] -eq "id,title") "$($r.Out) $($r.Err)"
    $cjk = "$([char]0x6E38)$([char]0x620F) $instance"
    $r = Invoke-Cli @("fence", "rename", $id, $cjk)
    Check "fence rename to a CJK title" ($r.Code -eq 0) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "get", $id, "--ascii", "--fields", "title")
    $isAscii = -not ($r.Out.ToCharArray() | Where-Object { [int]$_ -gt 127 })
    Check "--ascii output is pure ASCII and decodes to the same title" ($r.Code -eq 0 -and $isAscii -and $r.Out.Contains(([string][char]92) + 'u6e38') -and (Json $r.Out).title -eq $cjk) "$($r.Out) $($r.Err)"

    $r = Invoke-Cli @("fence", "get", $portalId)
    $pg = Json $r.Out
    Check "portal reports fit" ($r.Code -eq 0 -and $null -ne $pg.fit -and $pg.fit.columns -ge 1 -and $pg.fit.rows -ge 1 -and $pg.fit.fittingHeight -gt 0) "$($r.Out) $($r.Err)"
    Check "a portal at its own folder has no portal.current" ($null -eq $pg.portal.current -and $pg.portal.path) "$($r.Out)"
    $r = Invoke-Cli @("fence", "fit", $portalId)
    $pf = Json $r.Out
    Check "fence fit sets fittingHeight and leaves no overflow" ($r.Code -eq 0 -and $pf.fence.rect.h -eq $pg.fit.fittingHeight -and $pf.fence.fit.overflow -eq $false) "$($r.Out) $($r.Err)"
    $before96 = $pf.fence.rect
    $r = Invoke-Cli @("fence", "set", $portalId, "iconSize", "96")
    $s96 = Json $r.Out
    $moved = ($s96.fence.rect.w -ne $before96.w) -or ($s96.fence.rect.h -ne $before96.h) -or ($s96.fence.rect.x -ne $before96.x)
    Check "fence set iconSize reports adjusted exactly when the rect changed" ($r.Code -eq 0 -and ($moved -eq ($null -ne $s96.adjusted)) -and (-not $moved -or $s96.adjusted.reason -eq "cellSnap")) "$($r.Out) $($r.Err)"

    $pr = $s96.fence.rect
    $r = Invoke-Cli @("fence", "create", "--title", "Placed $instance", "--right-of", $portalId, "--size", "400,300")
    $pl = Json $r.Out
    Check "fence create --right-of aligns the top edge and keeps the gap" ($r.Code -eq 0 -and $pl.fence.rect.y -eq $pr.y -and $pl.fence.rect.x -ge ($pr.x + $pr.w) -and $pl.fence.rect.w -eq 400) "$($r.Out) $($r.Err)"
    $placedId = $pl.fence.id
    $r = Invoke-Cli @("fence", "create", "--title", "x", "--left-of", $portalId, "--size", "100000,300")
    Check "placement off the work area -> invalid_value" ($r.Code -eq 1 -and (Json $r.Err).error.code -eq "invalid_value") "code=$($r.Code) $($r.Err)"
    $r = Invoke-Cli @("fence", "create", "--title", "x", "--size", "300,200")
    Check "--size without a side -> usage" ($r.Code -eq 2) "code=$($r.Code) $($r.Err)"

    $r = Invoke-Cli @("fence", "merge", $placedId, "--into", $id)
    Check "merge placed fence into the test fence" ($r.Code -eq 0) "$($r.Out) $($r.Err)"
    $tab = Json (Invoke-Cli @("fence", "get", $placedId)).Out
    $hostDto = Json (Invoke-Cli @("fence", "get", $id)).Out
    $sameRect = ($tab.rect.x -eq $hostDto.rect.x) -and ($tab.rect.y -eq $hostDto.rect.y) -and ($tab.rect.w -eq $hostDto.rect.w) -and ($tab.rect.h -eq $hostDto.rect.h)
    Check "a tab reports its host's rect and no windowRect" ($sameRect -and $null -eq $tab.windowRect -and $tab.tabHost -eq $id) (($tab | ConvertTo-Json -Compress -Depth 3) + " / " + ($hostDto.rect | ConvertTo-Json -Compress))
    Check "only the shown tab has fit" ((($null -ne $tab.fit) -xor ($null -ne $hostDto.fit))) "tab.fit=$($tab.fit) host.fit=$($hostDto.fit)"
    # Tab bar position Left lists a host's tabs in a column while its title bar is at the top:
    # rolled up, the window shrinks to that column. A title bar at the bottom keeps its row.
    $r = Invoke-Cli @("settings", "set", "tabBar", "left")
    Check "settings set tabBar left" ($r.Code -eq 0) "$($r.Out) $($r.Err)"
    Invoke-Cli @("fence", "roll", $id) | Out-Null
    Start-Sleep -Milliseconds 800
    $hostDto = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "tabBar left: a rolled host at the top shrinks to its tab column" ($hostDto.rolledUp -eq $true -and $hostDto.titlePosition -eq "top" -and $hostDto.windowRect.w -lt $hostDto.rect.w -and $hostDto.windowRect.x -eq $hostDto.rect.x) "$($hostDto | ConvertTo-Json -Compress -Depth 4)"
    Invoke-Cli @("fence", "set", $id, "titlePosition", "bottom") | Out-Null
    $hostDto = Json (Invoke-Cli @("fence", "get", $id)).Out
    Check "tabBar left: a host with its title bar at the bottom keeps its title row" ($hostDto.rolledUp -eq $true -and $hostDto.windowRect.w -eq $hostDto.rect.w) "$($hostDto | ConvertTo-Json -Compress -Depth 4)"
    Invoke-Cli @("fence", "set", $id, "titlePosition", "top") | Out-Null
    Invoke-Cli @("settings", "set", "tabBar", "top") | Out-Null
    Invoke-Cli @("fence", "unroll", $id) | Out-Null
    Start-Sleep -Milliseconds 800
    $r = Invoke-Cli @("fence", "delete", $placedId)
    Check "delete the tab" ($r.Code -eq 0) "$($r.Out) $($r.Err)"

    $otherDir = Join-Path $stage "portal-b"
    New-Item -ItemType Directory -Path $otherDir | Out-Null
    Set-Content -LiteralPath (Join-Path $otherDir "notes.md") -Value "already here" -Encoding Ascii
    $r = Invoke-Cli @("fence", "create", "--portal", $otherDir, "--title", "SmokePortalB $instance")
    $otherId = (Json $r.Out).fence.id
    Check "second portal create" ($r.Code -eq 0 -and $otherId) "$($r.Out) $($r.Err)"
    Start-Sleep -Milliseconds 800
    $billPath = Join-Path $portalDir "2026-09 electricity bill.pdf"
    $notesPath = Join-Path $portalDir "notes.md"
    $r = Invoke-Cli @("item", "move", $billPath, $notesPath, "--to", $otherId, "--dry-run")
    $dm = Json $r.Out
    $bill = @($dm.moves) | Where-Object { $_.name -like "2026-09*" } | Select-Object -First 1
    $notes = @($dm.moves) | Where-Object { $_.name -like "notes*" } | Select-Object -First 1
    Check "item move --dry-run changes nothing" ($r.Code -eq 0 -and $dm.dryRun -eq $true -and $dm.changed -eq $false -and (Test-Path $billPath) -and (Test-Path $notesPath)) "$($r.Out) $($r.Err)"
    Check "dry run: fileMove with the destination" ($bill.action -eq "fileMove" -and [IO.Path]::GetFullPath([string]$bill.destination) -eq [IO.Path]::GetFullPath((Join-Path $otherDir "2026-09 electricity bill.pdf"))) "$($r.Out)"
    Check "dry run: a name that exists is skip/exists" ($notes.action -eq "skip" -and $notes.reason -eq "exists" -and $dm.moved -eq 1 -and $dm.fileMove -eq $true) "$($r.Out)"
    $r = Invoke-Cli @("item", "move", $billPath, $notesPath, "--to", $otherId)
    $mv = Json $r.Out
    Check "item move skips the collision with a warning" ($r.Code -eq 0 -and $mv.moved -eq 1 -and $mv.fileMove -eq $true -and @($mv.skipped).Count -eq 1 -and @($mv.skipped)[0].reason -eq "exists" -and $mv.warning -like "*already there*") "$($r.Out) $($r.Err)"
    $deadline = (Get-Date).AddSeconds(8)
    while (-not (Test-Path (Join-Path $otherDir "2026-09 electricity bill.pdf")) -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 200 }
    Check "the real file arrived and the colliding one stayed" ((Test-Path (Join-Path $otherDir "2026-09 electricity bill.pdf")) -and (Test-Path $notesPath) -and ((Get-Content -Raw (Join-Path $otherDir "notes.md")).Trim() -eq "already here")) ""
    $r = Invoke-Cli @("item", "move", (Join-Path $otherDir "notes.md"), "--to", $otherId, "--dry-run")
    Check "dry run into its own fence is none" ($r.Code -eq 0 -and @((Json $r.Out).moves)[0].action -eq "none") "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "delete", $otherId)
    Check "second portal delete" ($r.Code -eq 0) "$($r.Out) $($r.Err)"

    $r = Invoke-Cli @("snapshot", "save", "counts $instance")
    $sc = Json $r.Out
    $fenceCount = @(Json (Invoke-Cli @("fence", "list")).Out).Count
    Check "snapshot fenceCount is the current layout's" ($r.Code -eq 0 -and $sc.snapshot.fenceCount -eq $fenceCount) "$($r.Out) fences=$fenceCount"
    $null = Invoke-Cli @("snapshot", "delete", $sc.snapshot.id)
    $r = Invoke-Cli @("settings", "set", "snapping.guideLines", "false")
    Check "snapping.guideLines stored without a warning" ($r.Code -eq 0 -and (Json $r.Out).settings.snapping.guideLines -eq $false -and -not (Json $r.Out).warning) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("settings", "set", "snapping.gapPx", "12")
    Check "snapping.gapPx stored" ($r.Code -eq 0 -and (Json $r.Out).settings.snapping.gapPx -eq 12) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "delete", $portalId)
    Check "portal delete" ($r.Code -eq 0) "$($r.Out) $($r.Err)"

    # 7a'. Offline helpers: paths, log, config check.
    $r = Invoke-Cli @("paths")
    $pp = Json $r.Out
    Check "paths reports the running instance's config" ($r.Code -eq 0 -and $pp.running -eq $true -and $pp.source -eq "status" -and $pp.configExists -eq $true -and $pp.logExists -eq $true) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("log", "-n", "5")
    Check "log prints text" ($r.Code -eq 0 -and $r.Out.Length -gt 0 -and -not $r.Out.StartsWith("{")) "code=$($r.Code) $($r.Err)"
    $r = Invoke-Cli @("config", "check")
    $ck = Json $r.Out
    Check "config check of the live config passes" ($r.Code -eq 0 -and $ck.ok -eq $true -and $ck.schema -eq "https://pecofence.jiang.jp/schema/config.json") "$($r.Out) $($r.Err)"

    # 7b. Config export / import round trip and backups.
    $exportPath = Join-Path $stage "export.json"
    $r = Invoke-Cli @("config", "export", $exportPath)
    $ex = Json $r.Out
    Check "config export writes the file" ($r.Code -eq 0 -and (Test-Path $exportPath) -and $ex.bytes -gt 100) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("config", "export", "relative.json")
    Check "config export relative path -> invalid_value" ($r.Code -eq 1 -and (Json $r.Err).error.code -eq "invalid_value") "code=$($r.Code) $($r.Err)"
    $r = Invoke-Cli @("config", "import", $exportPath)
    $im = Json $r.Out
    Check "config import round trip" ($r.Code -eq 0 -and $im.changed -eq $true -and $im.imported) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "get", $id)
    Check "fence survives export/import" ($r.Code -eq 0 -and (Json $r.Out).id -eq $id) "$($r.Out) $($r.Err)"
    Set-Content -LiteralPath (Join-Path $stage "bad.json") -Value "{ not json" -Encoding Ascii
    $r = Invoke-Cli @("config", "import", (Join-Path $stage "bad.json"))
    Check "config import garbage -> validation_failed" ($r.Code -eq 1 -and (Json $r.Err).error.code -eq "validation_failed") "code=$($r.Code) $($r.Err)"
    $r = Invoke-Cli @("config", "check", (Join-Path $stage "bad.json"))
    Check "config check garbage -> validation_failed exit 1" ($r.Code -eq 1 -and (Json $r.Err).error.code -eq "validation_failed") "code=$($r.Code) $($r.Err)"
    $r = Invoke-Cli @("config", "check", $exportPath)
    Check "config check of the export passes" ($r.Code -eq 0 -and (Json $r.Out).ok -eq $true) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("backup", "list")
    Check "backup list ok" ($r.Code -eq 0) "$($r.Err)"
    $r = Invoke-Cli @("backup", "restore", $exportPath)
    Check "backup restore of a non-backup -> invalid_value" ($r.Code -eq 1 -and (Json $r.Err).error.code -eq "invalid_value") "code=$($r.Code) $($r.Err)"

    # 8. Delete; inbox protected.
    $r = Invoke-Cli @("fence", "delete", $id)
    Check "fence delete" ($r.Code -eq 0 -and (Json $r.Out).changed -eq $true) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("fence", "get", $id)
    Check "deleted fence gone -> fence_not_found" ($r.Code -eq 1 -and (Json $r.Err).error.code -eq "fence_not_found") "code=$($r.Code) $($r.Err)"
    $inbox = $before | Where-Object { $_.kind -eq "inbox" } | Select-Object -First 1
    if ($inbox) {
        $r = Invoke-Cli @("fence", "delete", $inbox.id)
        Check "inbox delete -> unsupported" ($r.Code -eq 1 -and (Json $r.Err).error.code -eq "unsupported") "code=$($r.Code) $($r.Err)"
    }
    $r = Invoke-Cli @("fence", "list")
    Check "fence count back to baseline" (@(Json $r.Out).Count -eq $before.Count) "before=$($before.Count) after=$(@(Json $r.Out).Count)"

    # 9. Config file reflects the last mutation right away (save before reply).
    # (Windows PowerShell's ConvertFrom-Json chokes on the full config, so match the text.)
    $cfgText = Get-Content -Raw (Join-Path $stage "config/config.json")
    $saved = $cfgText -match '"peek":\s*\{[^}]*"enabled":\s*false'
    Check "config.json saved with peek.enabled=false" $saved ""

    # A left title column with too little room on its right opens to the left
    # (as wide as it fits there, which other test fences may cut); after a restart it rolls back
    # into the same column (`rightAnchored` keeps that end).
    $r = Invoke-Cli @("fence", "create", "--title", "Column $instance", "--rect", "$($x + 500),$($work.y + 620),300,100")
    $cid = (Json $r.Out).fence.id
    Invoke-Cli @("fence", "set", $cid, "titlePosition", "left") | Out-Null
    Invoke-Cli @("fence", "roll", $cid) | Out-Null
    Start-Sleep -Milliseconds 800
    $ccol = (Json (Invoke-Cli @("fence", "get", $cid)).Out).windowRect
    $r = Invoke-Cli @("fence", "create", "--title", "Wall $instance", "--rect", "$($ccol.x + $ccol.w + 50),$($work.y + 620),200,100")
    $wall = (Json $r.Out).fence.id
    Start-Sleep -Milliseconds 400
    Invoke-Cli @("fence", "unroll", $cid) | Out-Null
    Start-Sleep -Milliseconds 800
    $c = Json (Invoke-Cli @("fence", "get", $cid)).Out
    Check "a left column with too little room on its right opens to the left" ($c.rolledUp -eq $false -and $c.rightAnchored -eq $true -and ($c.windowRect.x + $c.windowRect.w) -eq ($ccol.x + $ccol.w) -and $c.windowRect.w -gt ($ccol.w + 50) -and $c.windowRect.w -le 300) "col=$($ccol | ConvertTo-Json -Compress) window=$($c.windowRect | ConvertTo-Json -Compress)"

    # 10. Issue #125: a rolled fence with expandUpward comes back at the same title row after a
    # restart (the window is created with its row at the saved end of the saved rect).
    $r = Invoke-Cli @("fence", "create", "--title", "Upward $instance", "--rect", "$x,$($y + 200),400,300")
    $up = (Json $r.Out).fence.id
    Invoke-Cli @("fence", "set", $up, "expandUpward", "true") | Out-Null
    Invoke-Cli @("fence", "roll", $up) | Out-Null
    # A rolled bottom title row comes back at its row too.
    $r = Invoke-Cli @("fence", "create", "--title", "Footer $instance", "--rect", "$($x + 450),$($y + 200),300,300")
    $foot = (Json $r.Out).fence.id
    Invoke-Cli @("fence", "set", $foot, "titlePosition", "bottom") | Out-Null
    Invoke-Cli @("fence", "roll", $foot) | Out-Null
    Start-Sleep -Milliseconds 800
    $was = Json (Invoke-Cli @("fence", "get", $up)).Out
    $footWas = Json (Invoke-Cli @("fence", "get", $foot)).Out
    # The roll's landing records the rect standing on the row; its save runs 800 ms later.
    Start-Sleep -Milliseconds 1200
    Stop-Process -Id $app.Id -Force
    Start-Sleep -Milliseconds 500
    $app = Start-Process -FilePath (Join-Path $stage "pecofence.exe") -WorkingDirectory $stage `
        -ArgumentList @("--portable", "--no-hide-icons", "--exit-after", "$LifetimeMs") -PassThru
    $deadline = (Get-Date).AddSeconds(30)
    do {
        Start-Sleep -Milliseconds 250
        $r = Invoke-Cli @("status")
    } while ($r.Code -ne 0 -and (Get-Date) -lt $deadline)
    Start-Sleep -Milliseconds 500
    $now = Json (Invoke-Cli @("fence", "get", $up)).Out
    Check "expandUpward rolled fence keeps its spot across a restart" ($now.rolledUp -eq $true -and $now.rect.y -eq $was.rect.y -and $now.rect.h -eq $was.rect.h -and $now.windowRect.y -eq $was.windowRect.y -and ($now.windowRect.y + $now.windowRect.h) -eq ($now.rect.y + $now.rect.h)) "before=$($was.rect | ConvertTo-Json -Compress) $($was.windowRect | ConvertTo-Json -Compress) after=$($now.rect | ConvertTo-Json -Compress) $($now.windowRect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "delete", $up) | Out-Null
    $now = Json (Invoke-Cli @("fence", "get", $foot)).Out
    Check "a rolled bottom title row keeps its spot across a restart" ($now.rolledUp -eq $true -and $now.titlePosition -eq "bottom" -and $now.windowRect.y -eq $footWas.windowRect.y -and $now.windowRect.h -eq $footWas.windowRect.h -and $now.rect.y -eq $footWas.rect.y -and $now.bottomAnchored -eq $footWas.bottomAnchored) "before=$($footWas.rect | ConvertTo-Json -Compress) $($footWas.windowRect | ConvertTo-Json -Compress) after=$($now.rect | ConvertTo-Json -Compress) $($now.windowRect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "delete", $foot) | Out-Null
    $c = Json (Invoke-Cli @("fence", "get", $cid)).Out
    Check "after a restart the open column still keeps its right edge" ($c.rolledUp -eq $false -and $c.rightAnchored -eq $true -and ($c.rect.x + $c.rect.w) -eq ($ccol.x + $ccol.w)) "col=$($ccol | ConvertTo-Json -Compress) rect=$($c.rect | ConvertTo-Json -Compress) window=$($c.windowRect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "roll", $cid) | Out-Null
    Start-Sleep -Milliseconds 800
    $c = Json (Invoke-Cli @("fence", "get", $cid)).Out
    Check "after a restart the column rolls back into the same spot" ($c.rolledUp -eq $true -and $c.windowRect.x -eq $ccol.x -and $c.windowRect.w -eq $ccol.w) "col=$($ccol | ConvertTo-Json -Compress) now=$($c.windowRect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "delete", $cid) | Out-Null
    Invoke-Cli @("fence", "delete", $wall) | Out-Null

    # A title row at the top of the screen, switched on: the fence does not fit above it, so
    # the row stays and it opens downward.
    $r = Invoke-Cli @("fence", "create", "--title", "Top $instance", "--rect", "$x,$($work.y + 20),400,300")
    $top = (Json $r.Out).fence.id
    Start-Sleep -Milliseconds 500
    Invoke-Cli @("fence", "roll", $top) | Out-Null
    Start-Sleep -Milliseconds 800
    $row = (Json (Invoke-Cli @("fence", "get", $top)).Out).windowRect
    Invoke-Cli @("fence", "set", $top, "expandUpward", "true") | Out-Null
    $w = Json (Invoke-Cli @("fence", "get", $top)).Out
    Check "expandUpward at the top of the screen: the row stays, the rect hangs below it" ($w.windowRect.y -eq $row.y -and $w.rect.y -eq $row.y -and $w.rect.h -eq 300 -and $w.bottomAnchored -eq $false) "rect=$($w.rect | ConvertTo-Json -Compress) window=$($w.windowRect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "unroll", $top) | Out-Null
    Start-Sleep -Milliseconds 800
    $w = Json (Invoke-Cli @("fence", "get", $top)).Out
    Check "expandUpward at the top of the screen: opens downward" ($w.windowRect.y -eq $row.y -and $w.windowRect.h -eq 300) "window=$($w.windowRect | ConvertTo-Json -Compress)"
    # fence resize --h keeps the top edge of a fence that opened downward.
    $was = $w.rect
    Invoke-Cli @("fence", "resize", $top, "--h", "$($was.h - 60)") | Out-Null
    $w = Json (Invoke-Cli @("fence", "get", $top)).Out
    Check "expandUpward resize --h keeps the top edge of a fence that opened downward" ($w.rect.h -eq ($was.h - 60) -and $w.rect.y -eq $was.y) "before=$($was | ConvertTo-Json -Compress) after=$($w.rect | ConvertTo-Json -Compress)"
    # Dock to top: the title row at the top edge, opening downward; the switch stays on.
    Invoke-Cli @("fence", "dock-top", $top) | Out-Null
    $w = Json (Invoke-Cli @("fence", "get", $top)).Out
    Check "dock-top: the title row sits at the top, the switch stays on" ($w.expandUpward -eq $true -and $w.rolledUp -eq $true -and $w.windowRect.y -eq $work.y -and $w.rect.y -eq $work.y) "rect=$($w.rect | ConvertTo-Json -Compress) window=$($w.windowRect | ConvertTo-Json -Compress)"
    Start-Sleep -Milliseconds 800
    Invoke-Cli @("fence", "unroll", $top) | Out-Null
    Start-Sleep -Milliseconds 800
    $w = Json (Invoke-Cli @("fence", "get", $top)).Out
    Check "dock-top: opens downward from the top edge" ($w.rolledUp -eq $false -and $w.windowRect.y -eq $work.y) "window=$($w.windowRect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "delete", $top) | Out-Null

    # A bottom title row near the top of the screen prefers up but does not fit above its row:
    # the row stays and the fence opens downward.
    $r = Invoke-Cli @("fence", "create", "--title", "Bottom $instance", "--rect", "$x,$($work.y + 20),400,300")
    $bottom = (Json $r.Out).fence.id
    Start-Sleep -Milliseconds 500
    Invoke-Cli @("fence", "roll", $bottom) | Out-Null
    Start-Sleep -Milliseconds 800
    $row = (Json (Invoke-Cli @("fence", "get", $bottom)).Out).windowRect
    Invoke-Cli @("fence", "set", $bottom, "titlePosition", "bottom") | Out-Null
    $w = Json (Invoke-Cli @("fence", "get", $bottom)).Out
    Check "a bottom title row at the top of the screen: the row stays, the rect hangs below it" ($w.expandUpward -eq $true -and $w.windowRect.y -eq $row.y -and $w.rect.y -eq $row.y -and $w.rect.h -eq 300 -and $w.bottomAnchored -eq $false) "rect=$($w.rect | ConvertTo-Json -Compress) window=$($w.windowRect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "unroll", $bottom) | Out-Null
    Start-Sleep -Milliseconds 800
    $w = Json (Invoke-Cli @("fence", "get", $bottom)).Out
    Check "a bottom title row at the top of the screen opens downward" ($w.rolledUp -eq $false -and $w.windowRect.y -eq $row.y -and $w.windowRect.h -eq 300) "window=$($w.windowRect | ConvertTo-Json -Compress)"
    # Rolled after opening downward, `fence resize --h` keeps the row and the end it is at.
    Start-Sleep -Milliseconds 400
    Invoke-Cli @("fence", "roll", $bottom) | Out-Null
    Start-Sleep -Milliseconds 800
    Invoke-Cli @("fence", "resize", $bottom, "--h", "250") | Out-Null
    $w = Json (Invoke-Cli @("fence", "get", $bottom)).Out
    Check "resize --h on a rolled bottom title row that opened down keeps the row and its end" ($w.bottomAnchored -eq $false -and $w.windowRect.y -eq $row.y -and $w.rect.y -eq $row.y -and $w.rect.h -eq 250) "rect=$($w.rect | ConvertTo-Json -Compress) window=$($w.windowRect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "unroll", $bottom) | Out-Null
    Start-Sleep -Milliseconds 800
    # Dock to top moves a bottom or side title bar to the top edge.
    Invoke-Cli @("fence", "dock-top", $bottom) | Out-Null
    $w = Json (Invoke-Cli @("fence", "get", $bottom)).Out
    Check "dock-top moves a bottom title bar to the top" ($w.titlePosition -eq "top" -and $w.rolledUp -eq $true -and $w.windowRect.y -eq $work.y) "$($w | ConvertTo-Json -Compress -Depth 4)"
    Start-Sleep -Milliseconds 800
    Invoke-Cli @("fence", "set", $bottom, "titlePosition", "right") | Out-Null
    Invoke-Cli @("fence", "dock-top", $bottom) | Out-Null
    $w = Json (Invoke-Cli @("fence", "get", $bottom)).Out
    Check "dock-top moves a side title bar to the top" ($w.titlePosition -eq "top" -and $w.rolledUp -eq $true -and $w.windowRect.y -eq $work.y -and $w.windowRect.w -eq $w.rect.w) "$($w | ConvertTo-Json -Compress -Depth 4)"
    Invoke-Cli @("fence", "delete", $bottom) | Out-Null

    # A bottom title row with Expand upward off opens downward where the fence fits below its
    # row. Near the bottom of the screen it opens upward instead, the fence above sliding up out
    # of its way, and covers it at no point.
    $lx = $x + 450
    $r = Invoke-Cli @("fence", "create", "--title", "Low $instance", "--rect", "$lx,$($work.y + 500),300,200")
    $low = (Json $r.Out).fence.id
    Start-Sleep -Milliseconds 500
    Invoke-Cli @("fence", "set", $low, "titlePosition", "bottom") | Out-Null
    Invoke-Cli @("fence", "set", $low, "expandUpward", "false") | Out-Null
    Invoke-Cli @("fence", "roll", $low) | Out-Null
    Start-Sleep -Milliseconds 800
    $lrow = (Json (Invoke-Cli @("fence", "get", $low)).Out).windowRect
    Invoke-Cli @("fence", "unroll", $low) | Out-Null
    Start-Sleep -Milliseconds 800
    $w = Json (Invoke-Cli @("fence", "get", $low)).Out
    Check "a bottom row with Expand upward off opens downward where it fits" ($w.rolledUp -eq $false -and $w.windowRect.y -eq $lrow.y -and $w.windowRect.h -eq 200) "row=$($lrow | ConvertTo-Json -Compress) window=$($w.windowRect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "delete", $low) | Out-Null
    $fy = $work.y + $work.h - 260
    $r = Invoke-Cli @("fence", "create", "--title", "Floor $instance", "--rect", "$lx,$fy,300,200")
    $floor = (Json $r.Out).fence.id
    $r = Invoke-Cli @("fence", "create", "--title", "Over $instance", "--rect", "$lx,$($fy - 110),300,100")
    $over = (Json $r.Out).fence.id
    Start-Sleep -Milliseconds 500
    Invoke-Cli @("fence", "set", $floor, "titlePosition", "bottom") | Out-Null
    Invoke-Cli @("fence", "set", $floor, "expandUpward", "false") | Out-Null
    Invoke-Cli @("fence", "roll", $floor) | Out-Null
    Start-Sleep -Milliseconds 800
    $frow = (Json (Invoke-Cli @("fence", "get", $floor)).Out).windowRect
    # Taller than the room between its row and the fence above.
    Invoke-Cli @("fence", "resize", $floor, "--h", "300") | Out-Null
    $overWas = (Json (Invoke-Cli @("fence", "get", $over)).Out).windowRect
    Invoke-Cli @("fence", "unroll", $floor) | Out-Null
    Start-Sleep -Milliseconds 900
    $w = (Json (Invoke-Cli @("fence", "get", $floor)).Out).windowRect
    $o = (Json (Invoke-Cli @("fence", "get", $over)).Out).windowRect
    Check "near the bottom of the screen it opens upward from its row" (($w.y + $w.h) -eq ($frow.y + $frow.h) -and $w.h -eq 300) "row=$($frow | ConvertTo-Json -Compress) window=$($w | ConvertTo-Json -Compress)"
    Check "the fence above slides up out of its way" (($o.y + $o.h) -le $w.y -and $o.y -lt $overWas.y) "floor=$($w | ConvertTo-Json -Compress) over=$($o | ConvertTo-Json -Compress) was=$($overWas | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "roll", $floor) | Out-Null
    Start-Sleep -Milliseconds 900
    $o = (Json (Invoke-Cli @("fence", "get", $over)).Out).windowRect
    Check "rolled up again, the fence above comes back" ($o.y -eq $overWas.y) "was=$($overWas | ConvertTo-Json -Compress) now=$($o | ConvertTo-Json -Compress)"
    foreach ($f in @($floor, $over)) { Invoke-Cli @("fence", "delete", $f) | Out-Null }

    # A rolled column moved to the other side stays where it was; at the right edge of the screen
    # a left column then opens to the left, and `fence resize --w` keeps that end. Back at the top,
    # the column's end is gone and resize keeps the left edge, as for any title row.
    $workRight = $work.x + $work.w
    $r = Invoke-Cli @("fence", "create", "--title", "Edge $instance", "--rect", "$($workRight - 300),$($work.y + 300),300,200")
    $edge = (Json $r.Out).fence.id
    Invoke-Cli @("fence", "set", $edge, "titlePosition", "right") | Out-Null
    Invoke-Cli @("fence", "roll", $edge) | Out-Null
    Start-Sleep -Milliseconds 800
    $bar = (Json (Invoke-Cli @("fence", "get", $edge)).Out).windowRect
    Invoke-Cli @("fence", "set", $edge, "titlePosition", "left") | Out-Null
    $w = Json (Invoke-Cli @("fence", "get", $edge)).Out
    Check "a rolled right column moved to the left stays where it was and opens to the left" ($w.titlePosition -eq "left" -and $w.windowRect.x -eq $bar.x -and $w.windowRect.y -eq $bar.y -and $w.rightAnchored -eq $true -and ($w.rect.x + $w.rect.w) -eq $workRight) "bar=$($bar | ConvertTo-Json -Compress) now=$($w.windowRect | ConvertTo-Json -Compress) rect=$($w.rect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "resize", $edge, "--w", "260") | Out-Null
    $w = Json (Invoke-Cli @("fence", "get", $edge)).Out
    Check "resize --w on a rolled left column at its right end keeps that end" ($w.rightAnchored -eq $true -and $w.rect.w -eq 260 -and ($w.rect.x + $w.rect.w) -eq $workRight -and ($w.windowRect.x + $w.windowRect.w) -eq $workRight) "rect=$($w.rect | ConvertTo-Json -Compress) window=$($w.windowRect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "set", $edge, "titlePosition", "top") | Out-Null
    $w = Json (Invoke-Cli @("fence", "get", $edge)).Out
    $was = $w.rect
    Invoke-Cli @("fence", "resize", $edge, "--w", "240") | Out-Null
    $w = Json (Invoke-Cli @("fence", "get", $edge)).Out
    Check "back at the top the column's end is gone and resize --w keeps the left edge" ($w.rightAnchored -eq $false -and $w.rect.x -eq $was.x -and $w.rect.w -eq 240) "before=$($was | ConvertTo-Json -Compress) after=$($w.rect | ConvertTo-Json -Compress)"
    Invoke-Cli @("fence", "delete", $edge) | Out-Null
}
finally {
    if ($app -and -not $app.HasExited) { Stop-Process -Id $app.Id -Force }
    Start-Sleep -Milliseconds 300
    Get-Process pecofence-watchdog -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -like "$stage*" } | Stop-Process -Force -ErrorAction SilentlyContinue
    $logPath = Join-Path $env:LOCALAPPDATA "PecoFence\pecofence.$instance.log"
    foreach ($k in $savedEnv.Keys) {
        if ($null -eq $savedEnv[$k]) { Remove-Item -Path "Env:$k" -ErrorAction SilentlyContinue }
        else { Set-Item -Path "Env:$k" -Value $savedEnv[$k] }
    }
}

Write-Host ""
if ($script:failures -gt 0) {
    Write-Host "$($script:checks - $script:failures)/$($script:checks) checks passed; log: $logPath"
    exit 1
}
Write-Host "$($script:checks)/$($script:checks) checks passed"
Remove-Item -LiteralPath $stage -Recurse -Force -ErrorAction SilentlyContinue
