# The description beside the usage statistics switch (Settings › 关于) in the build under test:
# in each of the 11 languages the small text of the row is the new sentence (two sentences, the
# second says what is never sent), it takes at most two lines at the window's first size, the row
# is not cut by its card, and the switch still turns the statistics off and on. Shots: about-<lang>
# (the settings window), row-<lang> (the three cards around the switch). Debug build (CDP).
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
$Langs = 'zh-CN', 'en', 'ja', 'zh-TW', 'ko', 'de', 'fr', 'es', 'pt-BR', 'ru', 'it'
$Row = "document.querySelector('[data-bind=usageStats]').closest('.card')"

Case 'the row in every language'
PfStart -Lang 'zh-CN'
$ok = PfSettings
Check 'the settings page opens over CDP' $ok "debug=$PfDebugBuild"
if ($ok) {
  Js "(document.querySelector('a[data-page=about]').click(), true)" | Out-Null
  Start-Sleep -Seconds 2
  $s = [G]::FindWindowW('PecoFence.Settings', [NullString]::Value)
  $r = Rect $s
  foreach ($lang in $Langs) {
    Pf settings set language $lang | Out-Null
    Start-Sleep -Milliseconds 1500
    $m = Js "(() => { const c = $Row, t = c.querySelector('.s'), b = t.getBoundingClientRect(), cb = c.getBoundingClientRect(); return { text: t.textContent, lines: Math.round(b.height / parseFloat(getComputedStyle(t).lineHeight)), inside: b.bottom <= cb.bottom && b.right <= c.querySelector('.toggle').getBoundingClientRect().left, title: c.querySelector('.t').textContent }; })()"
    $sentences = @($m.text -split '[。.]' | Where-Object { $_.Trim() }).Count
    Check "${lang}: two sentences, at most two lines, inside the card" ($sentences -eq 2 -and $m.lines -ge 1 -and $m.lines -le 2 -and $m.inside) "lines=$($m.lines) inside=$($m.inside) title='$($m.title)' text='$($m.text)'"
    ParkMouse
    Crop "about-$lang" $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) | Out-Null
    $a = PageRect '#updateAutoCard'
    $b = PageRect '#openManual'
    if ($a -and $b) { Crop "row-$lang" ($a.x - 8) ($a.y - 8) ($a.w + 16) ($b.cy + 34 - $a.y + 8) | Out-Null }
  }
  Pf settings set language zh-CN | Out-Null
  Start-Sleep -Milliseconds 1200

  Case 'the switch still works'
  PageClick '[data-bind=usageStats]'
  Start-Sleep -Milliseconds 800
  $off = Pf settings get usageStats
  PageClick '[data-bind=usageStats]'
  Start-Sleep -Milliseconds 800
  $on = Pf settings get usageStats
  Check 'a click turns it off, another turns it on' ($off -match 'false' -and $on -match 'true') "off=$off on=$on"
}
PfQuit
