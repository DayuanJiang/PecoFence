# Unit tests of the workspace (built by unit.sh) in the desktop session: one check per test
# executable, with the failed test names in the detail.
Case 'sources'
$repo = (Get-Content "$PfExtra\repo-path.txt" -Raw).Trim()
$win = 'C:' + ($repo -replace '/', '\')
New-Item -ItemType Directory -Force $win | Out-Null
& tar.exe -xf "$PfExtra\src.tar" -C $win 2>&1 | Out-Null
Check 'sources unpacked for CARGO_MANIFEST_DIR' (Test-Path "$win\crates") $win

Case 'unit tests'
Set-Location "$PfExtra\exe"
foreach ($t in Get-ChildItem "$PfExtra\exe\*.exe") {
  $out = & $t.FullName 2>&1 | Out-String
  $code = $LASTEXITCODE
  $out | Set-Content "$PfRun\$($t.BaseName).txt" -Encoding UTF8
  $sum = ($out -split "`n" | Where-Object { $_ -match '^test result:' }) -join ' | '
  $failed = ($out -split "`n" | Where-Object { $_ -match '^test .* FAILED$|^    \S+$' } | Select-Object -First 8) -join ' '
  Check $t.BaseName ($code -eq 0) "exit $code; $sum $failed"
}
Remove-Item -Recurse -Force 'C:\home' -ErrorAction SilentlyContinue
$PfAllowLogErrors = $true
