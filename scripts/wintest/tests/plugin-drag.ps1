# 插件 interface 2, the rest of row 5 of plan v6: drags out of a plugin's rows (assets\drag-demo)
# and drops into and drags out of a web plugin's page (assets\webdrag).
# - Rows: a data file of the plugin's folder (a copy into File Explorer, the file stays), a text
#   and a link (to a test drop target), a file of a folder the user picked; refused: a link to a
#   host not in links or network, a file of another type (.hta), a file outside the picked folders.
#   Each drag ends in `dragend` {row, dropped}; a click without moving is no drag.
# - The page: a text, a file and a link dropped on it reach it as the `drop` event (with the
#   point, so the zone under the pointer is found) and pf.drop.text reads the file; a folder is
#   refused; the browser's own drop events never fire. Out of the page: a link to example.com and
#   a text go, a link to other.org does not, nor do they on a WebView2 without DragStarting
#   (debug builds: PECOFENCE_TEST_OLD_WEBVIEW stands in for one).
# Debug build (CDP on the page), English interface. On the build before, the new checks fail.
#   --var Part=M   two monitors at 100 % and 150 % (a Hyper-V guest with the virtual display
#                  driver: --extra with prepare-guest.ps1, mon.cs and the driver): the drop point
#                  and the row under it, the drag pictures, on the 150 % monitor
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'A' }
$Dr = 'C:\pf\run\drag'
$script:Config = "$PfApp\config"

function ExOpen($path, $x, $y, $w, $h) {
  Start-Process explorer.exe "`"$path`""
  for ($i = 0; $i -lt 60; $i++) {
    foreach ($win in @((New-Object -ComObject Shell.Application).Windows())) {
      $p = try { $win.Document.Folder.Self.Path } catch { '' }
      if ($p -eq $path) { $hw = [IntPtr]$win.HWND; [L]::MoveWindow($hw, $x, $y, $w, $h, $true) | Out-Null; Start-Sleep -Milliseconds 1200; return $hw }
    }
    Start-Sleep -Milliseconds 250
  }
  return [IntPtr]::Zero
}
function ExPick($hw, [string]$leaf) {
  $root = [Windows.Automation.AutomationElement]::FromHandle($hw)
  $isItem = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::ListItem)
  $el = $null
  for ($k = 0; $k -lt 20 -and -not $el; $k++) {
    foreach ($n in @($leaf, [IO.Path]::GetFileNameWithoutExtension($leaf))) {
      $by = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::NameProperty, $n)
      if (-not $el) { $el = $root.FindFirst([Windows.Automation.TreeScope]::Descendants, (New-Object Windows.Automation.AndCondition($isItem, $by))) }
    }
    if (-not $el) { Start-Sleep -Milliseconds 250 }
  }
  if (-not $el) { Log "ExPick: '$leaf' not in the window"; return $null }
  $el.GetCurrentPattern([Windows.Automation.SelectionItemPattern]::Pattern).Select(); Start-Sleep -Milliseconds 150
  $r = $el.Current.BoundingRectangle
  return @([int]($r.Left + 40), [int](($r.Top + $r.Bottom) / 2))
}
# A press at (x1, y1), a glide to (x2, y2), a rest there (a shot when named), the release.
function Drag($x1, $y1, $x2, $y2, $shot = '') {
  if ($null -eq $x1 -or $null -eq $x2) { Log "Drag: no point ($x1,$y1 to $x2,$y2)"; return }
  [G]::MoveTo($x1, $y1); Start-Sleep -Milliseconds 200
  MDown; Start-Sleep -Milliseconds 200
  Glide $x1 $y1 ($x1 + 12) ($y1 + 12) 4 30
  Glide ($x1 + 12) ($y1 + 12) $x2 $y2 16 35
  Start-Sleep -Milliseconds 500
  [G]::MoveTo($x2 + 1, $y2 + 1); Start-Sleep -Milliseconds 500
  if ($shot) { Shot $shot }
  MUp; Start-Sleep -Milliseconds 1500
}
function Out() { return (FenceOut $script:Fid) }
function WaitOut($regex, $sec = 8) { for ($i = 0; $i -lt $sec * 2; $i++) { if ((Out) -match $regex) { return $true }; Start-Sleep -Milliseconds 500 }; return $false }
function CellRects($w) {
  $v = DumpVal $w 'script_cells'
  if (-not $v -or $v -eq '-') { return @() }
  return @($v -split ';' | ForEach-Object {
    $pair = $_ -split ':'; $n = $pair[1] -split ','
    [pscustomobject]@{ i = [int]$pair[0]; l = [double]$n[0]; t = [double]$n[1]; r = [double]$n[2]; b = [double]$n[3] }
  })
}
# The middle of row `row` of the fence titled `title` on screen (null: the row takes no press).
function RowPt($row, $title = 'Drag') {
  $w = DumpOf $title 5
  $r = @(CellRects $w | Where-Object { $_.i -eq $row * 100 }) | Select-Object -First 1
  if (-not $r) { Log "no hit area of row $row"; return $null }
  $sc = [double](DumpVal $w 'content_y') / 36.0
  return @([int]($w.l + [double](DumpVal $w 'content_x') + ($r.l + 40) * $sc), [int]($w.t + [double](DumpVal $w 'content_y') + ($r.t + $r.b) / 2 * $sc))
}
# The test drop target: a window that takes a copy or a link of anything and writes what it got.
function DropTarget($x, $y) {
  Remove-Item "$Dr\dst.txt" -ErrorAction SilentlyContinue
  $p = Start-Process "$Dr\dragdst.exe" -ArgumentList $x, $y, "$Dr\dst.txt" -PassThru
  for ($i = 0; $i -lt 40; $i++) { $p.Refresh(); if ($p.MainWindowHandle -ne [IntPtr]::Zero) { break }; Start-Sleep -Milliseconds 250 }
  Start-Sleep -Milliseconds 500
  return $p
}
function Got() { for ($i = 0; $i -lt 8 -and -not (Test-Path "$Dr\dst.txt"); $i++) { Start-Sleep -Milliseconds 250 }; return "$(Get-Content "$Dr\dst.txt" -Raw -ErrorAction SilentlyContinue)" }
function Forget() { Remove-Item "$Dr\dst.txt" -ErrorAction SilentlyContinue }
# The page (CDP): its state lines, an element's middle on screen (the page fills the fence to its
# bottom right).
function PageOut() { return "$(Js "document.getElementById('out').textContent")" }
function WaitPage($regex, $sec = 8) { for ($i = 0; $i -lt $sec * 2; $i++) { if ((PageOut) -match $regex) { return $true }; Start-Sleep -Milliseconds 500 }; return $false }
function ElPt([string]$sel) {
  $q = $sel | ConvertTo-Json
  $e = Js "(() => { const b = document.querySelector($q).getBoundingClientRect(), d = devicePixelRatio; return { x: (b.left + b.width / 2) * d, y: (b.top + b.height / 2) * d, pw: innerWidth * d, ph: innerHeight * d }; })()"
  $r = (FenceGet $script:Web).windowRect
  return @([int]($r.x + $r.w - $e.pw + $e.x), [int]($r.y + $r.h - $e.ph + $e.y))
}

Case 'setup: the two test plugins, their fences, the files, the drop target'
CloseWindows
New-Item -ItemType Directory -Force $Dr, "$Dr\target", "$Dr\folder" | Out-Null
[IO.File]::WriteAllText("$Dr\picked.txt", 'picked by the user')
[IO.File]::WriteAllText('C:\pf\run\outside.txt', 'not picked')
[IO.File]::WriteAllText("$Dr\dropped.txt", 'dropped on the page')
$data = "$script:Config\plugins-data\drag-demo"
New-Item -ItemType Directory -Force $data | Out-Null
$b = New-Object Drawing.Bitmap 32, 32; $g = [Drawing.Graphics]::FromImage($b); $g.Clear([Drawing.Color]::FromArgb(40, 140, 220)); $g.Dispose()
$b.Save("$data\pic.png", [Drawing.Imaging.ImageFormat]::Png)
[IO.File]::WriteAllText("$data\run.hta", '<script>close()</script>')
Add-Type -OutputAssembly "$Dr\dragdst.exe" -OutputType WindowsApplication -ReferencedAssemblies System.Windows.Forms, System.Drawing -TypeDefinition @'
using System; using System.Drawing; using System.IO; using System.Text; using System.Windows.Forms;
public static class DragDst {
  [STAThread] public static void Main(string[] a) {
    var f = new Form(); f.Text = "PF drop target"; f.StartPosition = FormStartPosition.Manual;
    f.Location = new Point(int.Parse(a[0]), int.Parse(a[1])); f.Size = new Size(320, 220); f.TopMost = true; f.AllowDrop = true;
    DragEventHandler over = (s, e) => { e.Effect = (e.AllowedEffect & DragDropEffects.Copy) != 0 ? DragDropEffects.Copy : (e.AllowedEffect & DragDropEffects.Link); };
    f.DragEnter += over; f.DragOver += over;
    f.DragDrop += (s, e) => {
      var sb = new StringBuilder();
      sb.AppendLine("formats=" + string.Join(",", e.Data.GetFormats(false)));
      var t = e.Data.GetData(DataFormats.UnicodeText) as string; sb.AppendLine("text=" + (t ?? "none"));
      var u = e.Data.GetData("UniformResourceLocatorW") as MemoryStream;
      sb.AppendLine("url=" + (u == null ? "none" : Encoding.Unicode.GetString(u.ToArray()).TrimEnd('\0')));
      var files = e.Data.GetData(DataFormats.FileDrop) as string[]; sb.AppendLine("files=" + (files == null ? "none" : string.Join("|", files)));
      var g = e.Data.GetData("FileGroupDescriptorW") as MemoryStream; var names = "none";
      if (g != null) { var b = g.ToArray(); var n = BitConverter.ToInt32(b, 0); var l = new string[n];
        for (int i = 0; i < n; i++) l[i] = Encoding.Unicode.GetString(b, 4 + i * 592 + 72, 520).Split('\0')[0];
        names = string.Join("|", l); }
      sb.AppendLine("virtual=" + names);
      File.WriteAllText(a[2], sb.ToString());
    };
    Application.Run(f);
  }
}
'@
# The test drag source (as plugin-local.ps1's): a window that starts an OLE drag of a text or a
# link on a press and writes the effect the drop took.
Add-Type -OutputAssembly "$Dr\dragsrc.exe" -OutputType WindowsApplication -ReferencedAssemblies System.Windows.Forms, System.Drawing -TypeDefinition @'
using System; using System.Drawing; using System.IO; using System.Text; using System.Windows.Forms;
public static class DragSrc {
  [STAThread] public static void Main(string[] a) {
    var f = new Form(); f.Text = "PF drag source"; f.StartPosition = FormStartPosition.Manual;
    f.Location = new Point(int.Parse(a[1]), int.Parse(a[2])); f.Size = new Size(320, 200); f.TopMost = true;
    f.MouseDown += (s, e) => {
      var d = new DataObject();
      if (a[0] == "link") d.SetData("UniformResourceLocatorW", new MemoryStream(Encoding.Unicode.GetBytes(a[3] + "\0")));
      d.SetText(a[3]);
      var r = f.DoDragDrop(d, DragDropEffects.Copy | DragDropEffects.Move | DragDropEffects.Link);
      File.WriteAllText(a[4], r.ToString());
      f.Close();
    };
    Application.Run(f);
  }
}
'@
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
PfPlacePlugin 'C:\pf\kit\assets\drag-demo' 'drag-demo'
PfPlacePlugin 'C:\pf\kit\assets\webdrag' 'webdrag'
$b.Save("$script:Config\plugins\webdrag\1.0.0\pic.png", [Drawing.Imaging.ImageFormat]::Png); $b.Dispose()
PfStart -Lang en -Script (DumpScript 400 30 3000)
$script:Fid = PfPluginFence 'Drag' '80,60,380,420' 'drag-demo'
Check 'the plugin shows its rows' (WaitOut 'dragend none' 20) (Out)
$script:Web = PfPluginFence 'Web' '80,480,460,380' 'webdrag'
Check 'the page loads' ([bool](PfWaitLog 'web plugin: page loading' 30)) ''
# The folder the user picked, as pf.folder.pick keeps it in the fence's settings.
PfQuit
$j = Get-Content "$script:Config\config.json" -Raw -Encoding UTF8 | ConvertFrom-Json
$spec = $j.scripts."$script:Fid"
if ($spec.settings) { $spec.settings | Add-Member __folders @($Dr) -Force } else { $spec | Add-Member settings ([pscustomobject]@{ __folders = @($Dr) }) -Force }
[IO.File]::WriteAllText("$script:Config\config.json", ($j | ConvertTo-Json -Depth 100), (New-Object Text.UTF8Encoding($false)))
PfStart -Lang en -Script (DumpScript 400 30 3000)
Check 'the plugin starts again with the picked folder' ((WaitOut 'dragend none' 20) -and (Get-Content "$script:Config\config.json" -Raw) -match '__folders') (Out)
Start-Sleep -Seconds 2

Case 'rows: the refused drags are logged, their rows take no drag'
foreach ($row in 'other', 'hta', 'outside') {
  Check "the drag of row $row is refused, with why" ([bool](PfWaitLog "plugin row: its drag is refused.*$row" 5)) (PfWaitLog "its drag is refused.*$row" 1)
}
function Ends() { return [int]([regex]::Match((Out), 'ends=(\d+)').Groups[1].Value) }
$n = Ends
$p = RowPt 4
Drag $p[0] $p[1] 1100 360
Check 'a refused row drags nothing: no dragend' ((Ends) -eq $n) (Out)

Case 'rows: a data file dragged into File Explorer is a copy, the file stays'
$ex = ExOpen "$Dr\target" 760 60 760 560
$p = RowPt 0
Drag $p[0] $p[1] 1100 360 'row-file-over-explorer'
Check 'the picture is copied into the folder' (Test-Path "$Dr\target\pic.png") "$((Get-ChildItem "$Dr\target" -Name) -join ',')"
Check 'it stays in the data folder (a copy, never a move)' (Test-Path "$data\pic.png") ''
Check 'the plugin hears dragend: its row, dropped' (WaitOut 'dragend row=pic dropped=true') (Out)
$p = RowPt 3
Drag $p[0] $p[1] 1100 360
Check 'a file of the picked folder goes too' ((Test-Path "$Dr\target\picked.txt") -and (Get-Content "$Dr\target\picked.txt") -eq 'picked by the user') "$((Get-ChildItem "$Dr\target" -Name) -join ',')"
CloseWindows

Case 'rows: a text and a link to another program'
$dst = DropTarget 760 160
$p = RowPt 1
Drag $p[0] $p[1] 900 260 'row-text-over-target'
$got = Got
Check 'the target gets the text' ($got -match 'text=text from the plugin') $got
Check 'dragend: the text row, dropped' (WaitOut 'dragend row=text dropped=true') (Out)
Forget
$p = RowPt 2
Drag $p[0] $p[1] 900 260
$got = Got
Check 'a link: the target gets it as a link and as its text' ($got -match 'url=https://example.com/page' -and $got -match 'text=https://example.com/page') $got
Stop-Process -Id $dst.Id -Force -ErrorAction SilentlyContinue
$p = RowPt 1
Drag $p[0] $p[1] (FreeSpot 1200 700)[0] (FreeSpot 1200 700)[1]
Check 'let go where nothing takes it: dragend says not dropped' (WaitOut 'dragend row=text dropped=false') (Out)
$n = Ends
[G]::Click($p[0], $p[1]); Start-Sleep -Seconds 1
Check 'a click without moving is no drag' ((Ends) -eq $n) (Out)

Case 'the page: a text, a file and a link dropped on it'
$ok = CdpOpen -Url 'pecofence-plugin://webdrag/*' -sec 30
Check 'the page has no refusal of drags out on this WebView2' (-not (PfWaitLog 'cannot check a drag out of the page' 1)) ''
function FromSource([string]$value, [string]$mode, $to, $shot = '') {
  trap { PfTrap $_; continue }
  Remove-Item "$Dr\src-effect.txt" -ErrorAction SilentlyContinue
  $sp = Start-Process "$Dr\dragsrc.exe" -ArgumentList $mode, 1200, 120, "`"$value`"", "$Dr\src-effect.txt" -PassThru
  for ($i = 0; $i -lt 40; $i++) { $sp.Refresh(); if ($sp.MainWindowHandle -ne [IntPtr]::Zero) { break }; Start-Sleep -Milliseconds 250 }
  Start-Sleep -Milliseconds 500
  Drag 1300 200 $to[0] $to[1] $shot
  for ($i = 0; $i -lt 20 -and -not (Test-Path "$Dr\src-effect.txt"); $i++) { Start-Sleep -Milliseconds 250 }
  Stop-Process -Id $sp.Id -Force -ErrorAction SilentlyContinue
  return "$(Get-Content "$Dr\src-effect.txt" -ErrorAction SilentlyContinue)"
}
$z2 = ElPt '#z2'
$eff = FromSource 'hello page' 'text' $z2 'page-text-over'
Check 'a text: the page hears drop with it, at the zone under the pointer' (WaitPage 'drop=files=none text=hello page url=none zone=z2 row=null') (PageOut)
Check 'the drag copies (the source keeps its text)' ($eff -eq 'Copy') "effect=$eff"
$z1 = ElPt '#z1'
$eff = FromSource 'https://example.com/x' 'link' $z1
Check 'a link: the page has it, in zone z1' (WaitPage 'url=https://example.com/x zone=z1') (PageOut)
$ex = ExOpen $Dr 960 60 600 560
$p = ExPick $ex 'dropped.txt'
Drag $p[0] $p[1] $z2[0] $z2[1] 'page-file-over'
Check 'a file: the page has its name and size, and reads it with pf.drop.text' ((WaitPage 'files=dropped.txt:19 ') -and (WaitPage 'text0=dropped on the page')) (PageOut)
$n = [int]([regex]::Match((PageOut), 'drops=(\d+)').Groups[1].Value)
$p = ExPick $ex 'folder'
Drag $p[0] $p[1] $z2[0] $z2[1] 'page-folder-over'
Check 'a folder: refused, the page hears nothing' ((PageOut) -match "drops=$n\b") (PageOut)
Check 'the browser''s own drop events never fired' ((PageOut) -match 'html=0') (PageOut)
CloseWindows

Case 'out of the page: a listed link and a text go, another link does not'
$dst = DropTarget 760 160
$p = ElPt '#ok'
Drag $p[0] $p[1] 900 260 'page-link-out'
$got = Got
Check 'the example.com link reaches the target' ($got -match 'url=https://example.com/page') $got
Check 'the page hears dragend with the effect' (WaitPage 'dragend=ok:(copy|link)') (PageOut)
Check 'the log names what the drag carried' ([bool](PfWaitLog 'web plugin: a drag out of the page' 3)) (PfWaitLog 'a drag out of the page' 1)
Forget
$p = ElPt '#txt'
Drag $p[0] $p[1] 900 260
$got = Got
Check 'the text reaches the target' ($got -match 'text=text from the page') $got
Forget
$p = ElPt '#no'
Drag $p[0] $p[1] 900 260 'page-other-out'
Check 'the other.org link is refused: nothing reaches the target' (-not (Test-Path "$Dr\dst.txt")) (Got)
Check 'the log says why' ([bool](PfWaitLog 'the drag out of the page is refused.*other.org' 3)) (PfWaitLog 'out of the page is refused' 1)
Forget
$p = ElPt '#pic'
Drag $p[0] $p[1] 900 260 'page-pic-out'
$got = Got
Log "picture drag: $got"
Check 'the picture of the page goes as a png' ($got -match 'virtual=pic\.png' -or $got -match 'pic\.png') $got
Stop-Process -Id $dst.Id -Force -ErrorAction SilentlyContinue

Case 'inside the page: its own drag and drop works'
$c = ElPt '#card'
$z1 = ElPt '#z1'
Drag $c[0] $c[1] $z1[0] $z1[1] 'page-card-inside'
Check 'a card dragged onto zone z1 lands there (the page gets its own drag)' (WaitPage 'inpage=c1 on z1') (PageOut)
Check 'still no drop from outside reached the browser' ((PageOut) -match 'html=0') (PageOut)

Case 'a WebView2 without DragStarting: nothing leaves the page, drops still come'
$env:PECOFENCE_TEST_OLD_WEBVIEW = '1'
PfQuit
PfStart -Lang en -Script (DumpScript 400 30 3000)
Check 'the log says the page drags nothing out' ([bool](PfWaitLog 'cannot check a drag out of the page' 30)) ''
$ok = CdpOpen -Url 'pecofence-plugin://webdrag/*' -sec 30
JsWait "document.getElementById('out').textContent.includes('ready') || true" 5 | Out-Null
Start-Sleep -Seconds 2
$dst = DropTarget 760 160
$p = ElPt '#ok'
Drag $p[0] $p[1] 900 260
Check 'the listed link does not leave the page either' (-not (Test-Path "$Dr\dst.txt")) (Got)
Stop-Process -Id $dst.Id -Force -ErrorAction SilentlyContinue
$z2 = ElPt '#z2'
$eff = FromSource 'still dropped' 'text' $z2
Check 'a drop on the page still comes' (WaitPage 'text=still dropped') (PageOut)
Remove-Item Env:\PECOFENCE_TEST_OLD_WEBVIEW
Get-Process dragsrc, dragdst -ErrorAction SilentlyContinue | Stop-Process -Force
