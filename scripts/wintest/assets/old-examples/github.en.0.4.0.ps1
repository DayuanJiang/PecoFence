# PecoFence script fence example: the newest open issues and pull requests of a GitHub
# repository; click a row to open it in the browser.
# Change the repository below, save, then choose "Run now" in the fence menu. No sign-in is
# needed (GitHub answers up to 60 times an hour).
# Each line is "text | key=value": href= opens a URL, icon= goes before the text, tooltip= shows
# while the pointer rests on the row.
# Titles and names come from the network and anyone can write them. Clean takes out what a row
# would read as its parameters (| " ' and line breaks) before they are printed.
function Clean($text) { "$text" -replace '\|', '¦' -replace '[\r\n]+', ' ' -replace '["'']', [char]0x2019 }
$repo = 'DayuanJiang/PecoFence'
$items = Invoke-RestMethod "https://api.github.com/repos/$repo/issues?state=open&per_page=6" `
  -Headers @{ 'User-Agent' = 'PecoFence' } -TimeoutSec 8
"$repo | href=https://github.com/$repo icon=🐙"
"---"
foreach ($i in $items) {
  $icon = if ($i.pull_request) { '🔀' } else { '🟢' }
  $number = [int]$i.number
  $title = Clean $i.title
  # The link is built from the number (GitHub sends a pull request's number on to its page).
  "#$number  $title | href=https://github.com/$repo/issues/$number icon=$icon tooltip=""$(Clean $i.user.login): $title"""
}
"---"
"Refresh | refresh=true icon=🔄"
