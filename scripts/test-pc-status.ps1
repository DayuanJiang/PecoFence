# Execute the real bundled scripts with deterministic network/CIM data in Windows PowerShell.
# Stop after one frame; no network changes, clipboard writes or external dependencies.
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot

function New-Configuration {
    param([string]$Status, [string[]]$Addresses, [bool]$Gateway)
    [pscustomobject]@{
        NetAdapter = [pscustomobject]@{ Status = $Status }
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
    param([string]$ClassName)
    # A slow query after output starts leaves a visibly incomplete streaming frame.
    if ($lines.Count -gt 0) { throw "Partial frame visible during $ClassName query: $($lines.Count) rows" }
    switch ($ClassName) {
        'Win32_Processor' { [pscustomobject]@{ LoadPercentage = 10 } }
        'Win32_OperatingSystem' {
            [pscustomobject]@{ FreePhysicalMemory = 50; TotalVisibleMemorySize = 100 }
        }
        'Win32_LogicalDisk' { $script:disks }
        default { throw "Unexpected CIM class: $ClassName" }
    }
}
function Get-PSDrive {
    param([string]$Name)
    [pscustomobject]@{ Free = 50GB; Used = 50GB }
}
function Start-Sleep { param([int]$Seconds) throw 'pc-status-frame-complete' }

$cases = @(
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
$expectedDisks = @(
    @('C:', 50, 50, 'green'),
    @('D:', 5, 5, 'red'),
    @('E:', 2, 20, 'green'),
    @('G:', 0, 0, 'red'),
    @('R:', 1, 50, 'green')
)

$failures = 0
foreach ($language in 'en', 'zh') {
    $path = Join-Path $root "crates/app/assets/scripts/status.$language.ps1"
    $source = [IO.File]::ReadAllText($path, [Text.Encoding]::UTF8)
    foreach ($case in $cases) {
        $script:configurations = $case.Configurations
        $lines = New-Object 'System.Collections.Generic.List[string]'
        try {
            & ([scriptblock]::Create($source)) | ForEach-Object { $lines.Add([string]$_) }
        } catch {
            if ($_.Exception.Message -ne 'pc-status-frame-complete') { throw }
        }
        $ipRows = @($lines | Where-Object { $_ -match ' \| copy=' })
        $expected = $case.Expected
        if ($ipRows.Count -ne 1 -or -not $ipRows[0].Contains(" $expected | copy=$expected ")) {
            Write-Host "FAIL $language / $($case.Name): $($ipRows -join '; ')"
            $failures++
        } else {
            Write-Host "PASS $language / $($case.Name)"
        }
        $diskRows = @($lines | Where-Object { $_ -match ' \| open=' })
        if ($diskRows.Count -ne $expectedDisks.Count) {
            Write-Host "FAIL $language / local disks: expected 5 rows, got $($diskRows.Count)"
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
    }
}
if ($failures) { throw "$failures PC status cases failed" }
