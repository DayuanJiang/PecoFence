# Compatibility entry point; both formats share make-windows.ps1.
param(
  [string]$Version = "",
  [string]$TargetDir = "target/package",
  [string]$Python = "python",
  [string]$Repository = "",
  [switch]$SkipBuild
)
$ErrorActionPreference = "Stop"
& "$PSScriptRoot/make-windows.ps1" -Format Portable @PSBoundParameters
