# PecoFence 脚本栅栏示例：显示天气（数据来自 wttr.in）。
# 把下面的 Tokyo 改成你所在的城市（英文或拼音），保存后在栅栏菜单里选「立即运行」。
# 脚本打印出来的每一行文字都会显示在栅栏里。
$city = 'Tokyo'
$w = Invoke-RestMethod "https://wttr.in/$($city)?format=j1&lang=zh" -TimeoutSec 8
$now = $w.current_condition[0]
$today = $w.weather[0]
$tomorrow = $w.weather[1]
$desc = $now.lang_zh[0].value
if (-not $desc) { $desc = $now.weatherDesc[0].value }
$rain = ($tomorrow.hourly | Measure-Object -Property chanceofrain -Maximum).Maximum
"$city  $($now.temp_C)°C  $desc"
"体感 $($now.FeelsLikeC)°C，湿度 $($now.humidity)%，风 $($now.windspeedKmph) km/h"
"今天 $($today.mintempC) 到 $($today.maxtempC)°C，明天 $($tomorrow.mintempC) 到 $($tomorrow.maxtempC)°C，降雨概率 $rain%"
