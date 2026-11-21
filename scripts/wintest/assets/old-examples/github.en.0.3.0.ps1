# PecoFence script fence example: the newest open issues and pull requests of a GitHub
# repository; click a row to open it in the browser.
# Change the repository below, save, then choose "Run now" in the fence menu. No sign-in is
# needed (GitHub answers up to 60 times an hour).
# Each line is "text | key=value": href= opens a URL, icon= goes before the text, tooltip= shows
# while the pointer rests on the row.
$repo = 'DayuanJiang/PecoFence'
$items = Invoke-RestMethod "https://api.github.com/repos/$repo/issues?state=open&per_page=6" `
  -Headers @{ 'User-Agent' = 'PecoFence' } -TimeoutSec 8
"$repo | href=https://github.com/$repo icon=🐙"
"---"
foreach ($i in $items) {
  $icon = if ($i.pull_request) { '🔀' } else { '🟢' }
  $title = $i.title -replace '"', "'"
  "#$($i.number)  $title | href=$($i.html_url) icon=$icon tooltip=""$($i.user.login): $title"""
}
"---"
"Refresh | refresh=true icon=🔄"
