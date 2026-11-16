# A stand-in HTTP service for the local and LAN plugin steps (tests/plugin-lan.ps1): answers like
# Ollama (-Kind ollama: GET /api/tags, POST /api/generate) or like Home Assistant (-Kind ha:
# GET /api/states, POST /api/services/homeassistant/toggle, both with "Authorization: Bearer
# <-Token>", else 401). One connection at a time; each request is a line in -Log:
#   <method> <path> host=<Host header> hosts=<how many Host headers> auth=<yes|no> from=<address> body=<body>
#   powershell -File standin-http.ps1 -Kind ollama -Address 127.0.0.1 -Port 11434 -Log C:\pf\run\lan\ollama.log
param([string]$Kind, [string]$Address, [int]$Port, [string]$Log, [string]$Token = 'ha-token-7d2e')
$ErrorActionPreference = 'Continue'
$lights = @{ 'light.kitchen' = 'on'; 'switch.fan' = 'off' }
function States() {
  $s = @(
    @{ entity_id = 'sensor.living_room_temperature'; state = '21.5'; attributes = @{ friendly_name = 'Living room temperature'; unit_of_measurement = [string][char]0xB0 + 'C' } },
    @{ entity_id = 'sensor.living_room_humidity'; state = '48'; attributes = @{ friendly_name = 'Living room humidity'; unit_of_measurement = '%' } },
    @{ entity_id = 'light.kitchen'; state = $lights['light.kitchen']; attributes = @{ friendly_name = 'Kitchen light' } },
    @{ entity_id = 'switch.fan'; state = $lights['switch.fan']; attributes = @{ friendly_name = 'Fan' } },
    @{ entity_id = 'sun.sun'; state = 'above_horizon'; attributes = @{ friendly_name = 'Sun' } }
  )
  return ConvertTo-Json -InputObject $s -Depth 5 -Compress
}
$l = New-Object System.Net.Sockets.TcpListener([System.Net.IPAddress]::Parse($Address), $Port)
try { $l.Start() } catch { Add-Content $Log "cannot listen on ${Address}:${Port}: $($_.Exception.Message)"; exit 1 }
Add-Content $Log "listening $Kind on ${Address}:$Port"
while ($true) {
  $c = $l.AcceptTcpClient()
  try {
    $c.ReceiveTimeout = 5000
    $s = $c.GetStream(); $r = New-Object IO.StreamReader($s, [Text.Encoding]::UTF8)
    $line = $r.ReadLine(); $hosts = @(); $len = 0; $auth = ''
    while ($true) { $h = $r.ReadLine(); if (-not $h) { break }
      if ($h -match '^Host:\s*(.*)$') { $hosts += $Matches[1] }
      if ($h -match '^Content-Length:\s*(\d+)') { $len = [int]$Matches[1] }
      if ($h -match '^Authorization:\s*(.*)$') { $auth = $Matches[1] } }
    $body = ''
    if ($len -gt 0) { $buf = New-Object char[] $len; $got = 0; while ($got -lt $len) { $n = $r.Read($buf, $got, $len - $got); if ($n -le 0) { break }; $got += $n }; $body = -join $buf[0..($got - 1)] }
    $method, $path = ($line -split ' ')[0, 1]
    $from = $c.Client.RemoteEndPoint.Address.ToString()
    Add-Content $Log "$method $path host=$($hosts -join ',') hosts=$($hosts.Count) auth=$(if ($auth) { 'yes' } else { 'no' }) from=$from body=$body" -Encoding UTF8
    $status = '200 OK'; $out = ''
    if ($Kind -eq 'ollama') {
      if ($path -eq '/api/tags') {
        $out = '{"models":[{"name":"llama3.2:3b","size":2019393189,"details":{"parameter_size":"3.2B"}},{"name":"qwen2.5:7b","size":4683087332,"details":{"parameter_size":"7.6B"}}]}'
      } elseif ($method -eq 'POST' -and $path -eq '/api/generate') {
        $q = try { ($body | ConvertFrom-Json) } catch { $null }
        $out = ConvertTo-Json -Compress @{ model = "$($q.model)"; response = "Stand-in answer to: $($q.prompt)"; done = $true }
      } elseif ($path -eq '/') { $out = 'Ollama is running' } else { $status = '404 Not Found'; $out = '404 page not found' }
    } else {
      if ($auth -ne "Bearer $Token") { $status = '401 Unauthorized'; $out = '401: Unauthorized' }
      elseif ($path -eq '/api/states') { $out = States }
      elseif ($method -eq 'POST' -and $path -eq '/api/services/homeassistant/toggle') {
        $id = "$(try { ($body | ConvertFrom-Json).entity_id } catch { '' })"
        if ($lights.ContainsKey($id)) { $lights[$id] = if ($lights[$id] -eq 'on') { 'off' } else { 'on' } }
        $out = '[]'
      } else { $status = '404 Not Found'; $out = '404: Not Found' }
    }
    $b = [Text.Encoding]::UTF8.GetBytes($out)
    $hb = [Text.Encoding]::ASCII.GetBytes("HTTP/1.1 $status`r`nContent-Type: application/json; charset=utf-8`r`nContent-Length: $($b.Length)`r`nConnection: close`r`n`r`n")
    $s.Write($hb, 0, $hb.Length); $s.Write($b, 0, $b.Length); $s.Flush()
  } catch { Add-Content $Log "error $($_.Exception.Message)" } finally { $c.Close() }
}
