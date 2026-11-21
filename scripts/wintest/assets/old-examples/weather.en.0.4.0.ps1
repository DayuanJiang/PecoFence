# PecoFence script fence example: today's weather (data from wttr.in).
# Change Tokyo below to your city, save, then choose "Run now" in the fence menu.
# Every line the script prints shows in the fence.
# The values come from the network. Clean takes out what a row would read as its parameters
# (| " ' and line breaks) before a line is printed.
function Clean($text) { "$text" -replace '\|', '¦' -replace '[\r\n]+', ' ' -replace '["'']', [char]0x2019 }
$city = 'Tokyo'
$w = Invoke-RestMethod "https://wttr.in/$($city)?format=j1" -TimeoutSec 8
$now = $w.current_condition[0]
$today = $w.weather[0]
$tomorrow = $w.weather[1]
$rain = ($tomorrow.hourly | Measure-Object -Property chanceofrain -Maximum).Maximum
Clean "$city  $($now.temp_C)°C  $($now.weatherDesc[0].value)"
Clean "Feels like $($now.FeelsLikeC)°C, humidity $($now.humidity)%, wind $($now.windspeedKmph) km/h"
Clean "Today $($today.mintempC) to $($today.maxtempC)°C, tomorrow $($tomorrow.mintempC) to $($tomorrow.maxtempC)°C, rain $rain%"
