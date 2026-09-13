param(
    [Parameter(Mandatory = $true)][string]$LogPath,
    [string]$Title = '文件夹',
    [int]$Width = 272,
    [int]$RolledHeight = 72,
    [int]$ExpandedHeight = 282,
    [double]$CornerRadius = 48
)
$ErrorActionPreference = 'Stop'
$frames = [Collections.Generic.List[object]]::new()
$pattern = '(window_w|window_h|client_w|client_h|inset_x|inset_y|chrome_w|chrome_h|clip_w|clip_h|radius_x|radius_y)=(-?\d+(?:\.\d+)?)'
foreach ($line in Get-Content -LiteralPath $LogPath) {
    if (-not $line.Contains("shape frame title=$Title ")) { continue }
    $fields = @{}
    foreach ($match in [regex]::Matches($line, $pattern)) {
        $fields[$match.Groups[1].Value] = [double]::Parse(
            $match.Groups[2].Value, [Globalization.CultureInfo]::InvariantCulture
        )
    }
    if ($fields.Count -ne 12) { throw "Incomplete frame: $line" }
    $fields.Opening = $line.Contains('peeking=true')
    $frames.Add([pscustomobject]$fields)
}
if ($frames.Count -eq 0) { throw 'No frame diagnostics found' }
$previousMode = $null
$previousHeight = 0
$maximumOpeningStep = 0
foreach ($frame in $frames) {
    $expectedRadius = [math]::Min($CornerRadius, [math]::Min($Width, $frame.window_h) / 2)
    if ($frame.window_w -ne $Width -or
        $frame.client_w -ne $Width -or $frame.chrome_w -ne $Width -or $frame.clip_w -ne $Width -or
        $frame.client_h -ne $frame.window_h -or $frame.chrome_h -ne $frame.window_h -or
        $frame.clip_h -ne $frame.window_h -or $frame.inset_x -ne 0 -or $frame.inset_y -ne 0 -or
        $frame.radius_x -ne $expectedRadius -or $frame.radius_y -ne $expectedRadius) {
        throw "Inconsistent frame geometry: $($frame | ConvertTo-Json -Compress)"
    }
    if ($previousMode -ne $frame.Opening) {
        $previousHeight = if ($frame.Opening) { $RolledHeight } else { $ExpandedHeight }
    }
    $step = $frame.window_h - $previousHeight
    if (($frame.Opening -and $step -lt 0) -or (-not $frame.Opening -and $step -gt 0)) {
        throw 'Height reversed during a transition'
    }
    if ($frame.Opening) { $maximumOpeningStep = [math]::Max($maximumOpeningStep, $step) }
    $previousHeight = $frame.window_h
    $previousMode = $frame.Opening
}
$opening = @($frames | Where-Object Opening)
$closing = @($frames | Where-Object { -not $_.Opening })
if ($opening.Count -eq 0 -or $closing.Count -eq 0 -or
    $opening[-1].window_h -ne $ExpandedHeight -or $closing[-1].window_h -ne $RolledHeight) {
    throw 'Both complete hover-open and hover-close transitions are required'
}
[pscustomobject]@{
    Frames = $frames.Count
    OpeningFrames = $opening.Count
    ClosingFrames = $closing.Count
    InvalidFrames = 0
    FirstOpeningHeight = $opening[0].window_h
    MaximumOpeningStep = $maximumOpeningStep
    WindowClientSurfaceAndClipMatch = $true
    ClientInsets = '0,0'
} | ConvertTo-Json
