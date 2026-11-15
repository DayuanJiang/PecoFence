# PecoFence script fence example: today's weather (data from wttr.in).
# Change Tokyo below to your city, save, then choose "Run now" in the fence menu.
# Every line the script prints shows in the fence.
$city = 'Tokyo'
$w = Invoke-RestMethod "https://wttr.in/$($city)?format=j1" -TimeoutSec 8
$now = $w.current_condition[0]
$today = $w.weather[0]
$tomorrow = $w.weather[1]
$rain = ($tomorrow.hourly | Measure-Object -Property chanceofrain -Maximum).Maximum
"$city  $($now.temp_C)°C  $($now.weatherDesc[0].value)"
"Feels like $($now.FeelsLikeC)°C, humidity $($now.humidity)%, wind $($now.windspeedKmph) km/h"
"Today $($today.mintempC) to $($today.maxtempC)°C, tomorrow $($tomorrow.mintempC) to $($tomorrow.maxtempC)°C, rain $rain%"
