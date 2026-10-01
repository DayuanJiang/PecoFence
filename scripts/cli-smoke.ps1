# End-to-end smoke test for pecofence-cli against an isolated PecoFence instance.
#
#   powershell -File scripts/cli-smoke.ps1                 # uses target/debug binaries
#   powershell -File scripts/cli-smoke.ps1 -Profile release
#   powershell -File scripts/cli-smoke.ps1 -BinDir target/package/release
#
# Copies pecofence.exe, pecofence-watchdog.exe, pecofence-cli.exe and WebView2Loader.dll into a
# scratch folder under .cache/, points LOCALAPPDATA/APPDATA there, and starts the app as
# `PECOFENCE_INSTANCE=clitest --portable --no-hide-icons`, so the user's configuration and
# desktop icons are never touched. Then it drives the instance with the CLI and asserts the JSON
# replies and exit codes. Exit code 0 = all checks passed.
param(
    [ValidateSet("debug", "release")] [string]$Profile = "debug",
    [int]$LifetimeMs = 120000,
    [string]$BinDir = ""
)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$bin = if ($BinDir) { (Resolve-Path -LiteralPath $BinDir).Path } else { Join-Path $root "target/$Profile" }
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
    -ArgumentList @("--portable", "--no-hide-icons", "--exit-after", "$LifetimeMs") -WindowStyle Hidden -PassThru
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
    Check "settings set negative number" ($r.Code -eq 0 -and (Json $r.Out).settings.snapping.gapPx -eq -4) "$($r.Out) $($r.Err)"
    $r = Invoke-Cli @("rule", "add", "--name", "Empty", "--to", "inbox", "--json", "[]")
    Check "rule add without conditions rejected" ($r.Code -ne 0) "$($r.Out)"

    # 4. Settings.
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
}
finally {
    if ($app -and -not $app.HasExited) { Stop-Process -Id $app.Id -Force }
    Start-Sleep -Milliseconds 300
    Get-Process pecofence-watchdog -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -like "$stage*" } | Stop-Process -Force -ErrorAction SilentlyContinue
    $logPath = Join-Path $stage "data\logs\pecofence.$instance.log"
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
$stagePath = [IO.Path]::GetFullPath($stage)
$cachePath = [IO.Path]::GetFullPath((Join-Path $root '.cache')) + [IO.Path]::DirectorySeparatorChar
if (-not $stagePath.StartsWith($cachePath, [StringComparison]::OrdinalIgnoreCase)) { throw "Unsafe test cleanup path: $stagePath" }
Remove-Item -LiteralPath $stagePath -Recurse -Force -ErrorAction SilentlyContinue
