# PecoFence 脚本栅栏示例：一个 GitHub 仓库最新的 issue 和 PR，点一行在浏览器里打开。
# 把下面的仓库名改成你关心的仓库，保存后在栅栏菜单里选“立即运行”。不需要登录（GitHub 每小时最多回答 60 次）。
# 每一行写成“文字 | 参数=值”：href= 点了打开网址，icon= 行首的图标，tooltip= 鼠标停留时的提示。
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
"刷新 | refresh=true icon=🔄"
