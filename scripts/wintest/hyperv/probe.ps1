# Probe of a Hyper-V guest before the kit's steps: screen size (the kit's steps expect
# 1600x900 at 100 %), Windows version and DPI.
Case 'screen'
$before = "$([G]::GetSystemMetrics(0))x$([G]::GetSystemMetrics(1))"
if ($before -ne '1600x900') { $r = [G]::SetRes(1600, 900); Log "SetRes 1600x900 -> $r"; Start-Sleep -Seconds 3 }
$after = "$([G]::GetSystemMetrics(0))x$([G]::GetSystemMetrics(1))"
Check 'the screen is 1600x900' ($after -eq '1600x900') "before=$before after=$after modes=$([G]::Modes())"
$os = Get-CimInstance Win32_OperatingSystem
Check 'Windows version' $true "$($os.Caption) $($os.Version)"
$dpi = (Get-ItemProperty 'HKCU:\Control Panel\Desktop\WindowMetrics' -ErrorAction SilentlyContinue).AppliedDPI
Check 'DPI' $true "AppliedDPI=$dpi"
Shot 'probe'
