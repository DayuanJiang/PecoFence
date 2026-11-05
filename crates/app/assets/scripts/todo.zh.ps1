# PecoFence 脚本栅栏示例：一个 todo.txt 里的待办，点方框打勾，结果写回文件。
# 想用自己的文件，把 $file 改成它的路径，例如 "$env:USERPROFILE\Documents\todo.txt"。
# 行尾的 action= 让 PecoFence 在点击时再运行一次这个脚本，并把它放在 PECOFENCE_ACTION 里。
$file = Join-Path $env:PECOFENCE_DATA 'todo.txt'
if (-not (Test-Path $file)) {
  Set-Content $file -Encoding UTF8 -Value @('[ ] 交房租', '[ ] 回复邮件', '[x] 买牛奶')
}
$lines = @(Get-Content $file -Encoding UTF8)
if ($env:PECOFENCE_ACTION -match '^toggle:(\d+)$') {
  $i = [int]$Matches[1]
  if ($i -lt $lines.Count -and $lines[$i] -match '^\[[ x]\]') {
    $mark = if ($lines[$i].StartsWith('[x]')) { '[ ]' } else { '[x]' }
    $lines[$i] = $mark + $lines[$i].Substring(3)
    Set-Content $file $lines -Encoding UTF8
  }
}
# 只有待办行能点，其他行原样显示。
for ($i = 0; $i -lt $lines.Count; $i++) {
  if ($lines[$i] -match '^\[[ x]\]') { "$($lines[$i]) | action=toggle:$i" } else { $lines[$i] }
}
"---"
"打开 todo.txt | open=""$file"" icon=📄"
