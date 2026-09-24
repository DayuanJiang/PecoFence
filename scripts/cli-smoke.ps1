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

    $r = Invoke-Cli @("fence", "rename", $id, "$title renamed")
    Check "fence rename" ($r.Code -eq 0 -and (Json $r.Out).fence.title -eq "$title renamed") "$($r.Out) $($r.Err)"

    # 3b. Review follow-ups: colour values, --string, --all, inbox alias, rect validation.
    $r = Invoke-Cli @("fence", "set", $id, "tint", "#FF8800")
    Check "fence set tint #FF8800" ($r.Code -eq 0 -and (Json $r.Out).fence.tint -eq "#FF8800") "$($r.Out) $($r.Err)"
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
