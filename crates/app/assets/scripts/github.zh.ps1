# PecoFence 脚本栅栏示例：一个 GitHub 仓库最新的 issue 和 PR，点一行在浏览器里打开。
# 把下面的仓库名改成你关心的仓库，保存后在栅栏菜单里选“立即运行”。不需要登录（GitHub 每小时最多回答 60 次）。
# 每一行写成“文字 | 参数=值”：href= 点了打开网址，icon= 行首的图标，tooltip= 鼠标停留时的提示。
# 标题和用户名来自网络，谁都可以写。Clean 在打印前去掉会被当成这一行参数的字符（| " ' 和换行）。
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
  # 网址用编号拼出来（PR 的编号 GitHub 会转到它自己的页面）。
  "#$number  $title | href=https://github.com/$repo/issues/$number icon=$icon tooltip=""$(Clean $i.user.login): $title"""
}
"---"
"刷新 | refresh=true icon=🔄"
