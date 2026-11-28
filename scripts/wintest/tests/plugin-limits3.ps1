# Plugin interface 3, the limits the owner lifted (plan v2 of 2026-11-22, rows L8 to L11). The
# test plugin assets\limits-test runs the pf calls the steps write into its data folder
# (cmd.json). L8: `network` names 127.0.0.1 and localhost, and a request to them goes by the
# local rules without a `local` port; a request to the port of Remote Desktop or of the
# Kubernetes API is refused whatever the host. L9: an answer of 1.5 MB reaches main.js (0.5.0
# refused one over 1 MB). L10: `folders: "any"` writes a .zip as text and an .exe as bytes, each
# marked as coming from the internet (Zone.Identifier with ZoneId=3), a .txt unmarked, and trashes
# the .exe. L11: a batch file gets `&echo INJECTED` as one argument, a PowerShell script gets
# `a; echo INJECTED` as one, a .cmd with the Mark of the Web is refused, and so is a .cmd inside
# the picked folder. On 0.5.0 the manifest itself is refused (an IP host, folders any, .bat).
# Release build, English interface.
$Lr = 'C:\pf\run\lim'
$Notes = "$Lr\Notes"
$Feed = "$Lr\feed"
$Id = 'limits-test'

# Types $path into the folder picker's Folder: box and picks it (as tests/plugin-folders.ps1).
function PickIn($dlg, $path) {
  [G]::SetForegroundWindow($dlg) | Out-Null; Start-Sleep -Milliseconds 300
  Chord 0x11 0x41; Start-Sleep -Milliseconds 80
  [G]::Type($path); Start-Sleep -Milliseconds 200
  [G]::Press(0x0D); Start-Sleep -Milliseconds 1200
  $b = @(([W]::Buttons($dlg) -split "`n") | Where-Object { $_ -match "`tSelect Folder`t" }) | Select-Object -First 1
  Log "folder picker: $path, button $b"
  if ($b) { $r = ($b -split "`t")[3] -split ','; [G]::Click([int](([int]$r[0] + [int]$r[2]) / 2), [int](([int]$r[1] + [int]$r[3]) / 2)) }
  Start-Sleep -Milliseconds 800
}
function Rows() {
  $map = @{}
  foreach ($l in "$(FenceOut $script:Fid)" -split "`n") { $k, $v = $l -split ' ', 2; if ($k) { $map[$k] = $v } }
  return $map
}
function WaitRow($k, $sec = 20) {
  for ($i = 0; $i -lt $sec * 2; $i++) { $v = (Rows)[$k]; if ("$v" -match '^(ok|error)') { return $v }; Start-Sleep -Milliseconds 500 }
  return (Rows)[$k]
}
# One pf call of the test plugin and its row.
function Call($k, $call, [object[]]$callArgs = @()) {
  $o = [ordered]@{ k = $k; call = $call; args = @($callArgs) }
  [IO.File]::WriteAllText("$script:Config\plugins-data\$Id\cmd.json", '[' + ($o | ConvertTo-Json -Compress -Depth 4) + ']')
  return (WaitRow $k)
}
function Zone($path) { return "$(Get-Content -LiteralPath $path -Stream Zone.Identifier -ErrorAction SilentlyContinue)" }

Case 'setup: programs, a feed, the plugin, its fence and a picked folder'
CloseWindows
New-Item -ItemType Directory -Force $Notes, $Feed | Out-Null
Set-Content "$Lr\echo.bat" "@echo off`r`necho [%1]" -Encoding ASCII
Set-Content "$Lr\args.ps1" 'Write-Output "[$($args[0])]"' -Encoding ASCII
Set-Content "$Lr\web.cmd" "@echo off`r`necho came from the web" -Encoding ASCII
Set-Content -LiteralPath "$Lr\web.cmd" -Stream Zone.Identifier -Value "[ZoneTransfer]`r`nZoneId=3"
Set-Content "$Notes\drop.cmd" "@echo off`r`necho written by a plugin" -Encoding ASCII
Set-Content "$Feed\hello.txt" 'hello from the box' -Encoding ASCII -NoNewline
[IO.File]::WriteAllText("$Feed\big.txt", ('a' * 1536000))
$script:Config = "$PfApp\config"
New-Item -ItemType Directory -Force "$script:Config\plugins-data\$Id" | Out-Null
PfPlacePlugin "C:\pf\kit\assets\$Id" $Id
$feedProc = PfFeed 8744 $Feed
PfStart
Pf settings set tools.allowSystem true | Out-Null
$script:Fid = PfPluginFence 'LT' '60,120,900,560' $Id
$started = [bool](PfWaitLog 'plugin fence starts its main.js' 20)
Check 'the plugin with an IP host, folders any and scripts starts' $started (FenceOut $script:Fid)
if (-not $started) { Stop-Process -Id $feedProc.Id -Force -ErrorAction SilentlyContinue; return }
$o = [ordered]@{ k = 'pick'; call = 'folder.pick'; args = @() }
[IO.File]::WriteAllText("$script:Config\plugins-data\$Id\cmd.json", '[' + ($o | ConvertTo-Json -Compress) + ']')
$dlg = WaitWin '#32770' 10
if ($dlg -ne [IntPtr]::Zero) { PickIn $dlg $Notes }
$v = WaitRow 'pick'
Check 'a folder to write in is picked' ($v -eq ('ok ' + (ConvertTo-Json $Notes))) $v

Case 'L8: this computer by address and by name, without a local port; refused ports for any host'
$v = Call 'h1' 'http.get' @('http://127.0.0.1:8744/hello.txt')
Check 'http://127.0.0.1:8744 (network names 127.0.0.1): the answer comes' ($v -eq 'ok [200,18,"hello from the box"]') $v
$v = Call 'h2' 'http.get' @('http://localhost:8744/hello.txt')
Check 'http://localhost:8744 (network names localhost): the answer comes' ($v -eq 'ok [200,18,"hello from the box"]') $v
Check 'the log says it went by the loopback rules' ([bool](PfWaitLog 'plugin local request: the name resolved.*reach=Loopback' 3)) ''
$v = Call 'h3' 'http.get' @('http://127.0.0.1:3389/')
Check 'port 3389 (Remote Desktop): refused before any connection' ($v -match '^error http.get: port 3389 \(Remote Desktop\) may not be reached') $v
$v = Call 'h4' 'http.get' @('https://localhost:6443/')
Check 'port 6443 (Kubernetes API) over https: refused' ($v -match '^error http.get: port 6443 \(Kubernetes API\) may not be reached') $v
$v = Call 'h5' 'http.get' @('http://nas/')
Check 'nas (a name without a dot) goes by the lan rules: it does not resolve here, and says so' ($v -match '^error .*nas') $v

Case 'L9: an answer of 1.5 MB reaches main.js'
$v = Call 'b1' 'http.get' @('http://127.0.0.1:8744/big.txt')
Check 'the 1.5 MB answer comes whole' ($v -eq 'ok [200,1536000,"aaaaaaaaaaaaaaaaaaaa"]') "$v".Substring(0, [Math]::Min(120, "$v".Length))

Case 'L10: any file in the picked folder, marked when it is no data file'
$v = Call 'w1' 'folder.write' @("$Notes\a.zip", 'zipped text')
Check 'a .zip written as text' ($v -eq 'ok null' -and [IO.File]::ReadAllText("$Notes\a.zip") -eq 'zipped text') $v
Check 'and marked as coming from the internet' ((Zone "$Notes\a.zip") -match 'ZoneId=3') (Zone "$Notes\a.zip")
$v = Call 'w2' 'folder.writeBytes' @("$Notes\tool.exe", 'TVqQAA==')
$bytes = if (Test-Path "$Notes\tool.exe") { ([IO.File]::ReadAllBytes("$Notes\tool.exe") | ForEach-Object { '{0:x2}' -f $_ }) -join '' } else { '' }
Check 'an .exe written as bytes, byte for byte' ($v -eq 'ok null' -and $bytes -eq '4d5a9000') "$v bytes=$bytes"
Check 'and marked' ((Zone "$Notes\tool.exe") -match 'ZoneId=3') (Zone "$Notes\tool.exe")
$v = Call 'w3' 'folder.writeBytes' @("$Notes\b.txt", 'aGk=')
Check 'a .txt as bytes: written, not marked' ($v -eq 'ok null' -and [IO.File]::ReadAllText("$Notes\b.txt") -eq 'hi' -and (Zone "$Notes\b.txt") -eq '') "$v zone=$(Zone "$Notes\b.txt")"
$left = @(Get-ChildItem -Force $Notes | Where-Object { $_.Name -like '.*.tmp' }).Count
Check 'no temporary file is left' ($left -eq 0) "tmp=$left"
$v = Call 'w4' 'folder.trash' @("$Notes\tool.exe")
Check 'the .exe goes to the Recycle Bin (any kind with folders any)' ($v -eq 'ok true' -and -not (Test-Path "$Notes\tool.exe")) $v
$v = Call 'w5' 'folder.writeBytes' @("$Notes\c.bin", 'not base64!')
Check 'bytes that are not base64: refused, nothing written' ($v -match '^error folder.writeBytes: not base64' -and -not (Test-Path "$Notes\c.bin")) $v

Case 'L11: batch files and scripts get their arguments as text; no program from the web or the picked folder'
$v = Call 'r1' 'run' @("$Lr\echo.bat", @('&echo INJECTED'))
Check 'echo.bat gets "&echo INJECTED" as one quoted argument' ($v -eq 'ok [0,"[\"&echo INJECTED\"]"]') $v
$v = Call 'r2' 'run' @("$Lr\args.ps1", @('a; echo INJECTED'))
Check 'args.ps1 "a; echo INJECTED": one argument' ($v -eq 'ok [0,"[a; echo INJECTED]"]') $v
$v = Call 'r3' 'run' @("$Lr\web.cmd", @())
Check 'web.cmd with the Mark of the Web: refused' ($v -match '^error .*came from the internet') $v
$v = Call 'r4' 'run' @("$Notes\drop.cmd", @())
Check 'drop.cmd inside the picked folder: refused' ($v -match '^error .*in a folder the plugin may write') $v
Check 'the log names each refusal' ((LogCount 'pf.run refused: the program carries the Mark of the Web') -ge 1 -and (LogCount 'pf.run refused: the program is in a folder the plugin may write') -ge 1) ''
Shot 'rows'
Stop-Process -Id $feedProc.Id -Force -ErrorAction SilentlyContinue
