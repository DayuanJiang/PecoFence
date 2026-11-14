# Issue #219, Windows 10 1809 / Server 2019 (build 17763): Segoe MDL2 Assets there is version
# 1.68 and lacks glyphs the app used, which showed as empty boxes: the icon of 退出 PecoFence in
# the tray menu, and five icons of the settings page (颜色模式, 自动隐藏滚动条, 为空时自动隐藏,
# 自由摆放, the 压缩包 chip). The unit test `the_icon_font_has_every_menu_glyph` covers the menus;
# this file covers the settings page: no icon on it is drawn as the missing-glyph box.
# Needs a debug build (the page over CDP). It passes on Windows 11 too; the gaps only show on a
# Windows 10 box, most on build 17763: a `manual` line in suite.txt.
Case 'setup'
CloseWindows
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
PfStart
$opened = PfSettings
Check 'the settings page opens over CDP (debug build)' ($opened -and $PfDebugBuild) "opened=$opened debug=$PfDebugBuild"

Case 'settings page icons'
# Every element with the icon class, drawn the way the page's own check draws it and compared
# with a code point no icon font has: the list of icons that would show as an empty box.
$missing = Js @'
(() => {
  const ctx = document.createElement('canvas').getContext('2d', { willReadFrequently: true });
  const drawn = ch => { ctx.clearRect(0, 0, 24, 24); ctx.font = '16px "Segoe Fluent Icons", "Segoe MDL2 Assets"'; ctx.textBaseline = 'top'; ctx.fillText(ch, 2, 2); return ctx.getImageData(0, 0, 24, 24).data.join(); };
  const box = drawn(String.fromCharCode(0xF8FF));
  const out = new Set();
  for (const el of document.querySelectorAll('.ic, .iconbtn')) {
    const t = el.textContent;
    if (t.length === 1 && t.charCodeAt(0) >= 0xE000 && drawn(t) === box) out.add('U+' + t.charCodeAt(0).toString(16).toUpperCase());
  }
  return { count: document.querySelectorAll('.ic, .iconbtn').length, missing: [...out].sort().join(' ') };
})()
'@
Check 'no icon of the settings page is an empty box' ($missing.count -gt 40 -and $missing.missing -eq '') "icons on the page: $($missing.count); not in the icon font: '$($missing.missing)'"
$alts = Js "[...document.querySelectorAll('.ic[data-alt]')].map(e => 'U+' + e.textContent.charCodeAt(0).toString(16).toUpperCase() + (e.textContent === e.dataset.alt ? ' (stand-in)' : '')).join(', ')"
Log "icons with a stand-in: $alts"
$font = @(PfLogAll | Where-Object { $_ -match 'font not installed; using fallback' }) -join ' | '
Log "icon font: $(if ($font) { $font } else { 'Segoe Fluent Icons is installed' })"
Shot 'icons-general-page'
CloseWindows
