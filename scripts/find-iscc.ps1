# Shared by packaging and installer tests. Never downloads tools on a developer's PC.
function Find-Iscc([string]$Path = "") {
  if (-not $Path) { $Path = $env:ISCC }
  if (-not $Path) {
    $command = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    if ($command) { $Path = $command.Source }
  }
  if (-not $Path) {
    foreach ($base in @($env:ProgramFiles, ${env:ProgramFiles(x86)}, (Join-Path $env:LOCALAPPDATA "Programs"))) {
      if ($base) {
        $candidate = Join-Path $base "Inno Setup 7/ISCC.exe"
        if (Test-Path -LiteralPath $candidate -PathType Leaf) { $Path = $candidate; break }
      }
    }
  }
  if (-not $Path) { throw "Inno Setup 7 is required. Install it or pass -Iscc <path-to-ISCC.exe>." }
  $resolved = (Get-Command $Path -ErrorAction Stop).Source
  $version = & $resolved --version
  if ($LASTEXITCODE -ne 0 -or "$version" -notmatch '^7\.') { throw "Expected Inno Setup 7, got: $version" }
  return $resolved
}
