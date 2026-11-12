# Hardening round, 2026-11-12. Parts (run.sh --var Part=A ...; without it every part runs):
#   A  a configuration imported from someone else: 「在资源管理器中打开文件夹」 of a folder fence
#      and of a smart fence opens folders only (the file can name a program there), and the
#      import holds the file's scripts off whatever the file's own 小工具 switch says.
#   B  icons: a shortcut whose target is gone, or whose file is not on this disk, keeps a real
#      icon, and every icon that comes back empty or generic says why in the log.
#   C  the filter box of a fence (Ctrl+F) is drawn in the fence's own text colour.
# Payloads are harmless: Notepad, and a .cmd that writes a marker file into C:\pf\run.
if (-not $Part) { $Part = 'ABC' }
function Tools() { return (PfJson settings get).tools }
function LogText() { return ((PfLogAll) -join "`n") }

if ($Part -match 'A') {
  Case 'HD-1 setup'
  CloseWindows
  $Mark = "$PfRun\hd1-ran.txt"
  $Probe = "$PfRun\hd1-probe.cmd"
  Set-Content $Probe -Value "@echo ran>> `"$Mark`"" -Encoding ASCII
  $DirA = "$PfRun\共享 资料"; $DirB = "$PfRun\下载 的 文件"
  New-Item -ItemType Directory -Force $DirA, $DirB | Out-Null
  Set-Content -LiteralPath "$DirA\说明.txt" -Value 'x'; Set-Content -LiteralPath "$DirB\季度 报告.txt" -Value 'x'
  PfStart -Lang en
  Start-Sleep -Seconds 2
  Pf fence create --portal $DirA --title 'Shared' --rect '420,140,360,240' | Out-Null
  Pf fence create --title 'Found' --smart-folder $DirB --rect '820,140,360,240' | Out-Null
  Start-Sleep -Seconds 2

  Case 'HD-1 a real folder still opens'
  $m = FenceMenuOf 'Shared'; $picked = MenuPick '^Open folder in File Explorer' $m
  $seen = $false
  for ($i = 0; $i -lt 20 -and -not $seen; $i++) { Start-Sleep -Milliseconds 500; $seen = ((Explorers) -join "`n").Contains($DirA) }
  Check 'folder fence: the menu opens its folder in Explorer' $seen "picked=$picked $((Explorers) -join ' / ')"
  CloseWindows
  $m = FenceMenuOf 'Found'; $picked = MenuPick '^Open folder in File Explorer' $m
  $seen = $false
  for ($i = 0; $i -lt 20 -and -not $seen; $i++) { Start-Sleep -Milliseconds 500; $seen = ((Explorers) -join "`n").Contains($DirB) }
  Check 'smart fence: the menu opens its folder in Explorer' $seen "picked=$picked $((Explorers) -join ' / ')"
  CloseWindows

  Case 'HD-1 an imported file names programs as the folders'
  $exp = "$PfRun\hd1-export.json"
  Pf config export $exp | Out-Null
  $json = [IO.File]::ReadAllText($exp)
  $bad = $json.Replace(($DirA -replace '\\', '\\'), ($Probe -replace '\\', '\\')).Replace(($DirB -replace '\\', '\\'), 'C:\\Windows\\System32\\notepad.exe')
  [IO.File]::WriteAllText("$PfRun\hd1-bad.json", $bad)
  Check 'the file to import names a .cmd and Notepad as folders' ($bad -match 'hd1-probe\.cmd' -and $bad -match 'notepad\.exe') ''
  Pf config import "$PfRun\hd1-bad.json" | Out-Null; Start-Sleep -Seconds 3
  Remove-Item $Mark -ErrorAction SilentlyContinue
  $m = FenceMenuOf 'Shared'; $texts = MenuTexts $m; $picked = MenuPick '^Open folder in File Explorer' $m
  Start-Sleep -Seconds 3
  Check 'folder fence: the menu line starts no program' ((Test-Path $Mark) -eq $false) "picked=$picked marker=$(Test-Path $Mark) menu=$texts"
  CloseMenus
  $m = FenceMenuOf 'Found'; $texts = MenuTexts $m; $picked = MenuPick '^Open folder in File Explorer' $m
  Start-Sleep -Seconds 3
  $np = @(Get-Process notepad -ErrorAction SilentlyContinue).Count
  Shot 'hd1-after-open-folder'
  Check 'smart fence: the menu line starts no program' ($np -eq 0) "picked=$picked notepads=$np menu=$texts"
  Check 'log: both were refused as no folder' (([regex]::Matches((LogText), 'open folder refused: not a folder')).Count -ge 2) ([regex]::Match((LogText), '.*open folder refused[^\n]*').Value)
  CloseMenus; CloseWindows

  Case 'HD-1 import and scripts'
  $S = "$PfRun\hd1-script.cmd"; Set-Content $S -Value '@echo hello' -Encoding ASCII
  $base = "$PfRun\hd1-base.json"; Pf config export $base | Out-Null
  Pf fence create --script $S --title 'Runner' --rect '420,440,300,160' | Out-Null; Start-Sleep -Seconds 2
  Pf config import $base | Out-Null; Start-Sleep -Seconds 3
  Check 'a file without scripts leaves the 小工具 switch on' ((Tools).enabled -eq $true) ((Tools) | ConvertTo-Json -Compress)
  # The undo snapshot of that import keeps the Runner fence and its script: an export holds both.
  Pf settings set tools.enabled true | Out-Null
  $withSnap = "$PfRun\hd1-snap.json"; Pf config export $withSnap | Out-Null
  $j = [IO.File]::ReadAllText($withSnap)
  $off = $j -replace '("tools":\s*\{\s*"enabled":\s*)true', '${1}false'
  [IO.File]::WriteAllText("$PfRun\hd1-off.json", $off)
  Check 'the file to import has a script, a snapshot with a script, and the switch off' ($off -match 'hd1-script' -and $off -match '"tools":\s*\{\s*"enabled":\s*false') ''
  $rep = PfJson config import "$PfRun\hd1-off.json"; Start-Sleep -Seconds 3
  Check 'the CLI reply says how many scripts wait' ($rep.scriptsHeld -eq 1) ($rep | ConvertTo-Json -Compress)
  $line = [regex]::Match((LogText), '.*imported script fences held off[^\n]*switch_was_on=false[^\n]*').Value
  Check 'the switch off in the file: its scripts are held and the import says so' ($line -ne '' -and (Tools).enabled -eq $false) $line
  Check 'a snapshot of the file brings no script' ($line -match 'snapshot_scripts=[1-9]') $line
  PfQuit
}

if ($Part -match 'B') {
  Case 'HD-2 setup'
  CloseWindows
  # Shortcuts in a folder of their own, shown by a folder fence in name order.
  # WScript.Shell saves a shortcut only under a path in the system code page: each is made
  # under an ASCII name and moved to its Chinese name. Attribute letters as `attrib` has them:
  # O offline (0x1000), P pinned (0x80000), U unpinned (0x100000), what a OneDrive folder gives.
  $Work = "$PfRun\hd2"; $Dir = "$Work\图标 探针"; $Gone = "$Work\gone"
  New-Item -ItemType Directory -Force $Dir, $Gone | Out-Null
  Copy-Item C:\Windows\regedit.exe "$Gone\app.exe"
  Copy-Item C:\Windows\regedit.exe "$Work\offline-target.exe"
  $ws = New-Object -ComObject WScript.Shell
  function Lnk($name, $target, $icon, $attr) {
    $tmp = "$Work\tmp.lnk"
    $s = $ws.CreateShortcut($tmp); $s.TargetPath = $target; if ($icon) { $s.IconLocation = $icon }; $s.Save()
    if ($attr) { & attrib.exe $attr $tmp | Out-Null }
    Move-Item -LiteralPath $tmp -Destination "$Dir\$name.lnk"
  }
  Lnk 'a 正常' 'C:\Windows\regedit.exe' $null $null
  Lnk 'b 目标没了' "$Gone\app.exe" $null $null
  Lnk 'c 目标没了 自带图标' "$Gone\app.exe" 'C:\Windows\System32\shell32.dll,43' $null
  Lnk 'd 快捷方式离线' 'C:\Windows\regedit.exe' $null '+O'
  Lnk 'e 快捷方式固定' 'C:\Windows\regedit.exe' $null '+P'
  Lnk 'f 快捷方式未固定' 'C:\Windows\regedit.exe' $null '+U'
  Lnk 'g 目标离线' "$Work\offline-target.exe" $null $null
  Remove-Item -Recurse -Force $Gone
  & attrib.exe +O "$Work\offline-target.exe" | Out-Null
  Log ("attributes: " + ((Get-ChildItem -LiteralPath $Dir | ForEach-Object { "$($_.Name.Substring(0, 1))=0x{0:x}" -f [int]$_.Attributes }) -join ' ') + " target=0x{0:x}" -f [int](Get-Item "$Work\offline-target.exe").Attributes)
  PfStart -Lang en
  Start-Sleep -Seconds 2
  Pf fence create --portal $Dir --title 'Probe' --rect '420,140,640,240' | Out-Null
  Start-Sleep -Seconds 8
  Shot 'hd2-probe'

  Case 'HD-2 a shortcut file with the offline attribute keeps its icon'
  $pa = ItemAt 'Probe' 'a 正常'; $pd = ItemAt 'Probe' 'd 快捷方式离线'
  $ca = Crop 'hd2-icon-normal' ($pa[0] - 24) ($pa[1] - 24) 48 48
  $cd = Crop 'hd2-icon-offline' ($pd[0] - 24) ($pd[1] - 24) 48 48
  $diff = [Px]::Diff($ca, $cd, 0, 47, 0, 47)
  Check 'it shows the same picture as a shortcut to the same program without the attribute' ($diff -lt 3) ("mean brightness difference {0:n1} of 255 (the white page against the program icon is about 60)" -f $diff)
  Crop 'hd2-fence' 420 140 640 150 | Out-Null

  Case 'HD-2 the log says why an icon is not the file''s own'
  $log = (Get-Content (PfLogPath) -Encoding UTF8) -join "`n"
  @([regex]::Matches($log, '[^\n]*icon came back empty, blank or generic[^\n]*') | ForEach-Object { $_.Value }) | ForEach-Object { Log "per file: $_" }
  $info = @([regex]::Matches($log, '[^\n]*icon is not the file''s own picture[^\n]*') | ForEach-Object { $_.Value })
  Check 'the offline shortcut: one info line with the type, the reason, its attributes and that the shortcut was read' (@($info | Where-Object { $_ -match 'file_ext="lnk" reason="generic" from_shortcut=true' -and $_ -match 'attrs=4128' -and $_ -match 'target_there=true' }).Count -eq 1) ($info -join ' // ')
  Check 'the shortcut whose target is gone: one info line that says the target is not there' (@($info | Where-Object { $_ -match 'file_ext="lnk" reason="generic_program" from_shortcut=false' -and $_ -match 'target_ext="exe"' -and $_ -match 'target_there=false' }).Count -eq 1) ($info -join ' // ')
  Check 'one line per type and reason, however many files' ($info.Count -eq 2) "$($info.Count) lines"
  $share = if (Test-Path (ShareLog)) { (Get-Content (ShareLog) -Encoding UTF8) -join "`n" } else { '' }
  $sl = @([regex]::Matches($share, '[^\n]*icon is not the file''s own picture[^\n]*') | ForEach-Object { $_.Value }) -join ' // '
  Check 'the shareable log keeps the type, the reason and the place, and no path' ($sl -match 'file_ext=lnk reason=generic from_shortcut=true place=fixed' -and $sl -match 'target_ext=exe' -and $sl -notmatch '探针|pf\\run|hd2') $sl

  # Other sizes, as other display scales ask for: Windows hands back its 256 px picture there
  # and PecoFence scales it down, so the white page must be known in that form too.
  Case 'HD-2 the white page is known at other icon sizes'
  foreach ($size in 72, 96) {
    Pf fence set Probe iconSize $size | Out-Null; Start-Sleep -Seconds 5
    $log = (Get-Content (PfLogPath) -Encoding UTF8) -join "`n"
    $seen = @([regex]::Matches($log, '[^\n]*icon came back empty, blank or generic[^\n]*d [^\n]*\.lnk reason="generic" from_shortcut=true[^\n]*')).Count
    Check "icons of $size px: the offline shortcut is found and read again" ($seen -eq $(if ($size -eq 72) { 2 } else { 3 })) "per-file lines for it so far: $seen"
  }
  Shot 'hd2-probe-96'
  PfQuit
}

if ($Part -match 'C') {
  Add-Type -ReferencedAssemblies System.Drawing @'
using System; using System.Collections.Generic; using System.Drawing; using System.Drawing.Imaging; using System.Runtime.InteropServices;
public static class HdLook {
  static int[] Px(Bitmap b) {
    var d = b.LockBits(new Rectangle(0, 0, b.Width, b.Height), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
    var px = new int[b.Width * b.Height]; Marshal.Copy(d.Scan0, px, 0, px.Length); b.UnlockBits(d);
    return px;
  }
  static int Lum(int c) { return (((c >> 16) & 255) * 299 + ((c >> 8) & 255) * 587 + (c & 255) * 114) / 1000; }
  static string Rgb(int c) { return ((c >> 16) & 255) + "," + ((c >> 8) & 255) + "," + (c & 255); }
  // Text on a plate: the plate is the pixel of median brightness, the text the pixel farthest
  // from it. "plate rgb|text rgb|plate brightness|text brightness".
  public static string Text(Bitmap b) {
    var px = Px(b); var lum = new int[px.Length];
    for (int i = 0; i < px.Length; i++) lum[i] = Lum(px[i]);
    var sorted = (int[])lum.Clone(); Array.Sort(sorted);
    int med = sorted[sorted.Length / 2], plate = 0, ink = 0, near = 999, far = -1;
    for (int i = 0; i < px.Length; i++) {
      int diff = Math.Abs(lum[i] - med);
      if (diff < near) { near = diff; plate = i; }
      if (diff > far) { far = diff; ink = i; }
    }
    return Rgb(px[plate]) + "|" + Rgb(px[ink]) + "|" + lum[plate] + "|" + lum[ink];
  }
  // A text box: its field is the most frequent colour, its text the pixel farthest from the
  // field. The caret is the field's colour inverted and counts as neither. Same format.
  public static string Box(Bitmap b) {
    var px = Px(b); var count = new Dictionary<int, int>();
    int field = 0, most = 0;
    foreach (int p in px) { int c = p & 0xFFFFFF, n; count.TryGetValue(c, out n); count[c] = ++n; if (n > most) { most = n; field = c; } }
    int fl = Lum(field), far = -1, text = field, caret = ~field & 0xFFFFFF;
    foreach (int p in px) { int c = p & 0xFFFFFF; if (c == caret) continue; int d = Math.Abs(Lum(c) - fl); if (d > far) { far = d; text = c; } }
    return Rgb(field) + "|" + Rgb(text) + "|" + fl + "|" + Lum(text);
  }
}
'@
  function HdWall($name) {
    $rgb = if ($name -eq 'light') { 236, 239, 243 } else { 18, 20, 26 }
    $bm = New-Object Drawing.Bitmap 32, 32; $gr = [Drawing.Graphics]::FromImage($bm); $gr.Clear([Drawing.Color]::FromArgb($rgb[0], $rgb[1], $rgb[2])); $gr.Dispose()
    $p = "$PfRun\wall-$name.bmp"; $bm.Save($p, [Drawing.Imaging.ImageFormat]::Bmp); $bm.Dispose()
    [G]::SystemParametersInfoW(0x14, 0, $p, 3) | Out-Null
    # PecoFence looks at the wallpaper every 5 seconds.
    Start-Sleep -Seconds 8
  }
  function HdRead($kind, $x, $y, $w, $h) {
    $b = [Px]::Grab([int]$x, [int]$y, [int]$w, [int]$h)
    $r = if ($kind -eq 'box') { [HdLook]::Box($b) } else { [HdLook]::Text($b) }
    $b.Dispose()
    $p = $r -split '\|'
    return [pscustomobject]@{ Back = $p[0]; Ink = $p[1]; BackLum = [int]$p[2]; InkLum = [int]$p[3]; DarkText = ([int]$p[3] -lt [int]$p[2]); Text = "text $($p[1]) on $($p[0])" }
  }
  function HdApart($a, $b) {
    $x = @($a -split ',' | ForEach-Object { [int]$_ }); $y = @($b -split ',' | ForEach-Object { [int]$_ })
    return [math]::Max([math]::Max([math]::Abs($x[0] - $y[0]), [math]::Abs($x[1] - $y[1])), [math]::Abs($x[2] - $y[2]))
  }

  Case 'HD-3 setup'
  CloseWindows
  $FDir = "$PfRun\hd3\合同 与 报告"
  New-Item -ItemType Directory -Force $FDir | Out-Null
  foreach ($n in 'a1 季度报告.txt', 'b2 合同.txt', 'c3 notes.txt') { Set-Content -LiteralPath "$FDir\$n" -Value 'x' -Encoding UTF8 }
  HdWall 'light'
  PfStart -Lang en
  Start-Sleep -Seconds 2
  Pf settings set featureTips false | Out-Null
  Pf settings set themeStyle fluent | Out-Null
  Pf fence create --portal $FDir --title 'Papers' --rect '300,160,420,220' | Out-Null
  Start-Sleep -Seconds 2

  # The filter box against the title it covers.
  function HdFilterBox($mode, $wall) {
    trap { PfTrap $_; continue }
    Case "HD-3 filter box: $mode mode, $wall wallpaper"
    Pf settings set theme $mode | Out-Null
    HdWall $wall
    Start-Sleep -Milliseconds 1500
    $w = (FenceGet 'Papers').windowRect
    # The fence gets the keyboard: a click on its empty lower part.
    [G]::Click(($w.x + $w.w - 40), ($w.y + $w.h - 30)); ParkMouse 1540 780; Start-Sleep -Milliseconds 500
    $title = HdRead 'text' ($w.x + 12) ($w.y + 8) 120 22
    Chord 0x11 0x46; Start-Sleep -Milliseconds 700
    $box = [G]::FindWindowW('PecoFence.Filter', [NullString]::Value)
    [G]::Type('abc'); Start-Sleep -Milliseconds 700
    $q = $null
    if ($box -ne [IntPtr]::Zero) { $r = Rect $box; $q = HdRead 'box' ($r.L + 34) ($r.T + 6) 100 15 }
    Crop "hd3-$mode-$wall" $w.x $w.y $w.w 64 | Out-Null
    $side = if ($title.DarkText) { 'dark' } else { 'light' }
    Check "the filter box has $side text, like the title it covers" ($q -and $q.DarkText -eq $title.DarkText) "title: $($title.Text); box: $(if ($q) { $q.Text } else { 'did not open' })"
    Check "the filter box's text has the title's own colour" ($q -and (HdApart $q.Ink $title.Ink) -le 32) "title text $($title.Ink), box text $(if ($q) { $q.Ink } else { 'did not open' })"
    Check "the filter box has a field that $side text reads on" ($q -and (($q.BackLum -ge 128) -eq $title.DarkText) -and [math]::Abs($q.BackLum - $q.InkLum) -ge 120) "box: $(if ($q) { "$($q.Text), brightness $($q.InkLum) on $($q.BackLum)" } else { 'did not open' })"
    [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  }
  HdFilterBox dark light
  HdFilterBox light dark
  HdFilterBox dark dark
  HdFilterBox light light
  PfQuit
}
