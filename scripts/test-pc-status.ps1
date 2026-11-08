# Execute the real bundled scripts with deterministic network/CIM data in Windows PowerShell.
# Stop after one frame; no network changes, clipboard writes or external dependencies.
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot

# $Physical: Wi-Fi or Ethernet (ConnectorPresent); a VPN or a virtual switch is not.
function New-Configuration {
    param([string]$Status, [string[]]$Addresses, [bool]$Gateway, [bool]$Physical = $true, [int]$Metric = 35)
    [pscustomobject]@{
        NetAdapter = [pscustomobject]@{ Status = $Status; ConnectorPresent = $Physical }
        NetIPv4Interface = [pscustomobject]@{ InterfaceMetric = $Metric }
        IPv4DefaultGateway = $(if ($Gateway) { [pscustomobject]@{ NextHop = '192.0.2.1' } })
        IPv4Address = @($Addresses | ForEach-Object { [pscustomobject]@{ IPAddress = $_ } })
    }
}

function Get-NetIPConfiguration { $script:configurations }
# The old implementation queries all addresses, including disconnected and virtual adapters.
function Get-NetIPAddress {
    param([string]$AddressFamily)
    if ($AddressFamily -ne 'IPv4') { throw 'Expected an IPv4 query' }
    $script:configurations | ForEach-Object { $_.IPv4Address }
}
function Get-CimInstance {
    param([string]$ClassName, [string]$Filter)
    # A slow query after output starts leaves a visibly incomplete streaming frame.
    if ($lines.Count -gt 0) { throw "Partial frame visible during $ClassName query: $($lines.Count) rows" }
    switch ($ClassName) {
        'Win32_Processor' { [pscustomobject]@{ LoadPercentage = 10 } }
        'Win32_OperatingSystem' {
            [pscustomobject]@{ FreePhysicalMemory = 50; TotalVisibleMemorySize = 100 }
        }
        'Win32_LogicalDisk' {
            # Windows filters in the query: network drives (a lost one can hang) and CD/DVD
            # drives are never asked.
            $types = @([regex]::Matches($Filter, 'DriveType = (\d)') | ForEach-Object { [int]$_.Groups[1].Value })
            if ($types -contains 4 -or $types -contains 5 -or -not $types) { throw "Disk query asks network or CD drives: '$Filter'" }
            $script:disks | Where-Object { $types -contains $_.DriveType }
        }
        default { throw "Unexpected CIM class: $ClassName" }
    }
}
function Get-PSDrive {
    param([string]$Name)
    [pscustomobject]@{ Free = 50GB; Used = 50GB }
}
# Ends the run after $script:stopAfter frames; $lines keeps the last frame. The address comes
# with frame 2 (frame 1 shows before the first lookup), then every 15 frames (30 s): a case with
# Later changes the network after frame 15, and frame 17 shows it.
function Start-Sleep {
    param([int]$Seconds)
    $script:frames++
    if ($script:frames -ge $script:stopAfter) { throw 'pc-status-frame-complete' }
    $lines.Clear()
    if ($script:frames -eq 15 -and $null -ne $script:later) { $script:configurations = $script:later }
}

$cases = @(
    @{
        Name = 'the first screen comes before the first lookup and copies nothing'
        Configurations = @((New-Configuration 'Up' @('192.168.40.162') $true))
        Frames = 1
        Expected = ''
    }
    @{
        Name = 'disconnected stale IP and virtual adapters before Wi-Fi'
        Configurations = @(
            (New-Configuration 'Disconnected' @('192.168.8.117') $true)
            (New-Configuration 'Up' @('172.21.208.1') $false)
            (New-Configuration 'Up' @('172.20.192.1') $false)
            (New-Configuration 'Up' @('192.168.40.162') $true)
        )
        Expected = '192.168.40.162'
    }
    @{
        Name = 'connected Ethernet with multiple IPv4 addresses'
        Configurations = @((New-Configuration 'Up' @('10.0.0.2', '10.0.0.3') $true))
        Expected = '10.0.0.2'
    }
    @{
        Name = 'exclude loopback and link-local addresses'
        Configurations = @((New-Configuration 'Up' @('127.0.0.1', '169.254.1.2', '10.0.0.2') $true))
        Expected = '10.0.0.2'
    }
    @{
        Name = 'no connected gateway interface'
        Configurations = @(
            (New-Configuration 'Disconnected' @('192.168.8.117') $true)
            (New-Configuration 'Up' @('172.21.208.1') $false)
        )
        Expected = ''
    }
    @{
        Name = 'no IPv4 address'
        Configurations = @((New-Configuration 'Up' @() $false))
        Expected = ''
    }
    @{
        Name = 'a VPN with its own gateway and a lower metric, and Wi-Fi'
        Configurations = @(
            (New-Configuration 'Up' @('198.18.0.1') $true $false 5)
            (New-Configuration 'Up' @('192.168.40.162') $true $true 35)
        )
        Expected = '192.168.40.162'
    }
    @{
        Name = 'Wi-Fi listed before Ethernet: Ethernet has the lower metric'
        Configurations = @(
            (New-Configuration 'Up' @('192.168.40.162') $true $true 35)
            (New-Configuration 'Up' @('192.168.40.20') $true $true 25)
        )
        Expected = '192.168.40.20'
    }
    @{
        Name = 'only a virtual switch adapter has the gateway (Hyper-V external switch)'
        Configurations = @(
            (New-Configuration 'Up' @() $false $true 25)
            (New-Configuration 'Up' @('192.168.1.50') $true $false 15)
        )
        Expected = '192.168.1.50'
    }
    @{
        Name = 'no network at all'
        Configurations = @()
        Expected = ''
    }
    @{
        Name = 'not connected at sign-in, Wi-Fi connects a little later'
        Configurations = @()
        Later = @((New-Configuration 'Up' @('192.168.40.162') $true))
        Frames = 17
        Expected = '192.168.40.162'
    }
    @{
        Name = 'moved to another Wi-Fi'
        Configurations = @((New-Configuration 'Up' @('192.168.40.162') $true))
        Later = @((New-Configuration 'Up' @('10.1.2.3') $true))
        Frames = 17
        Expected = '10.1.2.3'
    }
)

$script:disks = @(
    [pscustomobject]@{ DeviceID = 'D:'; DriveType = 3; Size = 100GB; FreeSpace = 5GB }
    [pscustomobject]@{ DeviceID = 'C:'; DriveType = 3; Size = 100GB; FreeSpace = 50GB }
    [pscustomobject]@{ DeviceID = 'E:'; DriveType = 2; Size = 10GB; FreeSpace = 2GB }
    [pscustomobject]@{ DeviceID = 'N:'; DriveType = 4; Size = 100GB; FreeSpace = 50GB }
    [pscustomobject]@{ DeviceID = 'F:'; DriveType = 5; Size = $null; FreeSpace = $null }
    [pscustomobject]@{ DeviceID = 'G:'; DriveType = 5; Size = 4GB; FreeSpace = 0 }
    [pscustomobject]@{ DeviceID = 'R:'; DriveType = 6; Size = 2GB; FreeSpace = 1GB }
)
# Not shown: N: (network), F: and G: (CD/DVD, empty and with a disc that is always full).
$expectedDisks = @(
    @('C:', 50, 50, 'green'),
    @('D:', 5, 5, 'red'),
    @('E:', 2, 20, 'green'),
    @('R:', 1, 50, 'green')
)

$failures = 0
foreach ($language in 'en', 'zh') {
    $path = Join-Path $root "crates/app/assets/scripts/status.$language.ps1"
    $source = [IO.File]::ReadAllText($path, [Text.Encoding]::UTF8)
    foreach ($case in $cases) {
        $script:configurations = $case.Configurations
        $script:later = $case.Later
        $script:stopAfter = if ($case.Frames) { $case.Frames } else { 2 }
        $script:frames = 0
        $lines = New-Object 'System.Collections.Generic.List[string]'
        try {
            & ([scriptblock]::Create($source)) | ForEach-Object { $lines.Add([string]$_) }
        } catch {
            if ($_.Exception.Message -ne 'pc-status-frame-complete') { throw }
        }
        $ipRows = @($lines | Where-Object { $_ -match ' \| copy=' })
        $expected = $case.Expected
        # Without an address the row says so and copies nothing.
        $noIpRows = @($lines | Where-Object { $_ -match ' \| color=gray icon=' })
        $ipOk = if ($expected) {
            $ipRows.Count -eq 1 -and $ipRows[0].Contains(" $expected | copy=$expected ") -and $noIpRows.Count -eq 0
        } else {
            $ipRows.Count -eq 0 -and $noIpRows.Count -eq 1
        }
        if (-not $ipOk) {
            Write-Host "FAIL $language / $($case.Name): $(($ipRows + $noIpRows) -join '; ')"
            $failures++
        } else {
            Write-Host "PASS $language / $($case.Name)"
        }
        $diskRows = @($lines | Where-Object { $_ -match ' \| open=' })
        # The drives come last, so a short fence cuts drives, not the IP or Task Manager rows.
        $firstDisk = [array]::FindIndex([string[]]$lines, [Predicate[string]] { param($l) $l -match ' \| open=' })
        $taskRow = [array]::FindIndex([string[]]$lines, [Predicate[string]] { param($l) $l -match 'run=taskmgr' })
        if ($firstDisk -lt $taskRow) {
            Write-Host "FAIL $language / $($case.Name): a drive row comes before the Task Manager row"
            $failures++
        }
        if ($diskRows.Count -ne $expectedDisks.Count) {
            Write-Host "FAIL $language / local disks: expected $($expectedDisks.Count) rows, got $($diskRows.Count)"
            $failures++
        } else {
            for ($i = 0; $i -lt $expectedDisks.Count; $i++) {
                $drive, $free, $pct, $color = $expectedDisks[$i]
                # Check capacities, low-space color and the exact root opened on click.
                $pattern = '^' + [regex]::Escape($drive.TrimEnd(':')) + '.*' +
                    [regex]::Escape("$free GB") + '.*' + [regex]::Escape("$pct%") +
                    '.*' + [regex]::Escape(" | open=$drive\ color=$color icon=")
                if ($diskRows[$i] -notmatch $pattern) { throw "Incorrect disk row: $($diskRows[$i])" }
            }
            Write-Host "PASS $language / local disks, removable media, colors and open paths"
        }
        if ($lines[$lines.Count - 1] -ne '~~~') { throw 'Incomplete status frame' }
        # The app reads parameters after a ' | ' only when all of them are key=value pairs (a
        # value with spaces needs quotes); otherwise the row is plain text and does nothing.
        foreach ($row in @($lines | Where-Object { $_ -match ' \| ' })) {
            $params = $row.Substring($row.IndexOf(' | ') + 3)
            if ($params -notmatch '^\s*(\w+=("[^"]*"|''[^'']*''|\S+)\s*)+$') {
                Write-Host "FAIL $language / $($case.Name): parameters the app would show as text: $row"
                $failures++
            }
        }
    }
}
if ($failures) { throw "$failures PC status cases failed" }
