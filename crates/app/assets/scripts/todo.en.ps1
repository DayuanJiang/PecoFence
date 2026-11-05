# PecoFence script fence example: the to-dos in a todo.txt; click a box to tick it in the file.
# To use your own file, set $file to its path, for example "$env:USERPROFILE\Documents\todo.txt".
# action= at the end of a line makes PecoFence run this script again on a click, with the
# action in PECOFENCE_ACTION.
$file = Join-Path $env:PECOFENCE_DATA 'todo.txt'
if (-not (Test-Path $file)) {
  Set-Content $file -Encoding UTF8 -Value @('[ ] Pay rent', '[ ] Answer emails', '[x] Buy milk')
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
# Only the to-do lines can be clicked; other lines show as they are.
for ($i = 0; $i -lt $lines.Count; $i++) {
  if ($lines[$i] -match '^\[[ x]\]') { "$($lines[$i]) | action=toggle:$i" } else { $lines[$i] }
}
"---"
"Open todo.txt | open=""$file"" icon=📄"
