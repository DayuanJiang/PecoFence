# 插件 interface 2, the rest of rows 5, 10 and 11 of plan v6 (assets\local-demo): a text and a link
# dragged from a test program reach the plugin (copied, never moved); a file from File Explorer is
# read as text and as base64 after the drop handler, and saved into the data folder; a file over
# 4 MB is refused to the plugin and still saved; pf.apps.front and the apps event follow Notepad
# starting, coming to the front and ending; a secret setting is typed masked with 显示, the plugin
# reads it, the configuration keeps it encrypted, the log and an export never have it.
# Debug build (CDP on the settings page), Chinese interface.
$Kit = 'C:\pf\kit\assets\local-demo'
$Lr = 'C:\pf\run\local'
$Token = 'tok-5f3a9c'

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
# The point of an item of an Explorer window (by its name with or without the extension), selected.
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
function DragDrop($x1, $y1, $x2, $y2, $shot = '') {
  [G]::MoveTo($x1, $y1); Start-Sleep -Milliseconds 150
  [G]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 200
  Glide $x1 $y1 ($x1 + 14) ($y1 + 14) 3 30
  Glide ($x1 + 14) ($y1 + 14) $x2 $y2 16 35
  Start-Sleep -Milliseconds 500
  [G]::MoveTo($x2 + 1, $y2 + 1); Start-Sleep -Milliseconds 500
  if ($shot) { Shot $shot }
  [G]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 1500
}
function FenceMid() { $r = (FenceGet $script:Fid).windowRect; return @([int]($r.x + $r.w / 2), [int]($r.y + $r.h / 2)) }
function WaitOut($regex, $sec = 10) { for ($i = 0; $i -lt $sec * 2; $i++) { if ((FenceOut $script:Fid) -match $regex) { return $true }; Start-Sleep -Milliseconds 500 }; return $false }
# The test program that starts an OLE drag of a text or a link (and its text) on a mouse press,
# and writes the effect the drop took into effect.txt.
function DragSource([string]$mode, [string]$value) {
  Remove-Item "$Lr\effect.txt" -ErrorAction SilentlyContinue
  $p = Start-Process "$Lr\dragsrc.exe" -ArgumentList $mode, 700, 160, "`"$value`"", "$Lr\effect.txt" -PassThru
  for ($i = 0; $i -lt 40; $i++) { $p.Refresh(); if ($p.MainWindowHandle -ne [IntPtr]::Zero) { break }; Start-Sleep -Milliseconds 250 }
  Start-Sleep -Milliseconds 500
  return $p
}
function Effect() { for ($i = 0; $i -lt 20 -and -not (Test-Path "$Lr\effect.txt"); $i++) { Start-Sleep -Milliseconds 250 }; return "$(Get-Content "$Lr\effect.txt" -ErrorAction SilentlyContinue)" }

Case 'setup: the test plugin, its fence, the test files'
CloseWindows
New-Item -ItemType Directory -Force $Lr | Out-Null
[IO.File]::WriteAllText("$Lr\notes-a.txt", 'a note')
$big = New-Object byte[] (5MB); for ($i = 0; $i -lt $big.Length; $i += 4096) { $big[$i] = 7 }
[IO.File]::WriteAllBytes("$Lr\big.bin", $big)
Add-Type -OutputAssembly "$Lr\dragsrc.exe" -OutputType WindowsApplication -ReferencedAssemblies System.Windows.Forms, System.Drawing -TypeDefinition @'
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
$script:Config = "$PfApp\config"
PfPlacePlugin $Kit 'local-demo'
PfStart -Lang zh-CN
$script:Fid = PfPluginFence '本机示例' '80,140,520,360' 'local-demo'
Check 'the plugin starts: no secret yet, Notepad does not run' ([bool](PfWaitLog 'plugin fence starts its main.js' 20) -and (WaitOut 'token=none') -and (WaitOut 'running=none front=none')) (FenceOut $script:Fid)

Case 'a text and a link dragged from another program: the plugin gets them, the drag copies'
$to = FenceMid
$src = DragSource 'text' 'hello from the test'
DragDrop 760 220 $to[0] $to[1] 'drop-text'
Check 'the drop event has the text, no files and no link' (WaitOut 'drop files=none text=hello from the test url=none') (FenceOut $script:Fid)
Check 'the drag ended as a copy (the source keeps its text)' ((Effect) -eq 'Copy') "effect=$(Effect)"
$src = DragSource 'link' 'https://example.com/page?q=1'
DragDrop 760 220 $to[0] $to[1]
Check 'a link: the drop event has the address (and its text)' (WaitOut 'url=https://example.com/page\?q=1') (FenceOut $script:Fid)
$src = DragSource 'link' 'file:///C:/Windows/win.ini'
DragDrop 760 220 $to[0] $to[1]
Check 'a file: link is no link for the plugin (its text still comes)' ((WaitOut 'text=file:///C:/Windows/win.ini url=none')) (FenceOut $script:Fid)

Case 'a file from File Explorer: its text, its bytes, a copy in the data folder'
$ex = ExOpen $Lr 760 60 760 600
$p = ExPick $ex 'notes-a.txt'
DragDrop $p[0] $p[1] $to[0] $to[1] 'drop-file'
Check 'the drop event names the file and its size' (WaitOut 'drop files=notes-a.txt:6 ') (FenceOut $script:Fid)
Check 'pf.drop.text after the handler: the file''s text' (WaitOut 'text0: a note') (FenceOut $script:Fid)
Check 'pf.drop.read: the bytes in base64' (WaitOut 'b64: YSBub3Rl') (FenceOut $script:Fid)
$saved = "$script:Config\plugins-data\local-demo\saved.bin"
Check 'pf.drop.save: the file in the data folder, the same bytes' ((WaitOut 'saved: ok') -and (Test-Path $saved) -and [IO.File]::ReadAllText($saved) -eq 'a note') "$(FenceOut $script:Fid)"

Case 'a file over 4 MB: not into the plugin, saved by PecoFence'
$p = ExPick $ex 'big.bin'
DragDrop $p[0] $p[1] $to[0] $to[1]
Check 'pf.drop.read and pf.drop.text are refused with the size' ((WaitOut 'b64 refused: .*5242880 bytes') -and (FenceOut $script:Fid) -match 'text0 refused') (FenceOut $script:Fid)
Check 'pf.drop.save copies all 5 MB' ((WaitOut 'saved: ok') -and (Get-Item $saved).Length -eq 5MB) "$((Get-Item $saved).Length)"
CloseWindows

Case 'apps: Notepad starts and comes to the front, goes behind, ends'
$np = Start-Process notepad.exe -PassThru
Start-Sleep -Seconds 1
Check 'the apps event: Notepad runs and is in front' (WaitOut 'event running=notepad.exe front=notepad.exe' 15) (FenceOut $script:Fid)
Check 'pf.apps.front names it as the manifest does' (WaitOut 'running=notepad.exe front=notepad.exe events=1') (FenceOut $script:Fid)
DeskClick
Check 'another window in front: the event says none is' (WaitOut 'event running=notepad.exe front=none' 15) (FenceOut $script:Fid)
Stop-Process -Id $np.Id -Force
Check 'Notepad ended: the event says it runs no more' (WaitOut 'event running=none front=none' 15) (FenceOut $script:Fid)
$e = [int]([regex]::Match((FenceOut $script:Fid), 'events=(\d+)').Groups[1].Value)
Start-Sleep -Seconds 11
$e2 = [int]([regex]::Match((FenceOut $script:Fid), 'events=(\d+)').Groups[1].Value)
Check 'no event while nothing changes' ($e -ge 3 -and $e2 -eq $e) "events $e then $e2"
Check 'the programs are read only while such a plugin shows: started once' ((LogCount 'they are read every 5 s from now on') -ge 1) ''

Case 'a secret setting: masked with 显示, the plugin reads it, kept encrypted'
$ok = PfSettings
$sw = WaitWin 'PecoFence.Settings' 5
[L]::MoveWindow($sw, 760, 60, 900, 900, $true) | Out-Null
PageClick 'nav a[data-page=tools]'; Start-Sleep -Seconds 3
$i = [int](Js "[...document.querySelectorAll('#pluginList .pl-row')].findIndex(r => r.textContent.startsWith('本机示例'))")
PageClick '#pluginList .pl-row' $i; Start-Sleep -Seconds 1
$d = Js "(() => { const d = document.querySelector('#pluginList .pl-detail'); return d ? d.innerText : ''; })()"
Check 'the details say what it does with drops and programs' ($d -match '你拖到它上面的文件和文字，它能读取' -and $d -match '知道这些程序是否在运行、是否在最前面：notepad.exe') $d
$kind = Js "(() => { const i = document.querySelector('#pluginList .pl-secret input'); return i ? i.type : ''; })()"
Check 'the secret is typed in a hidden box, with 显示 and the note' ($kind -eq 'password' -and $d -match '插件能读到这个值' -and $d -match '换到另一台电脑后要重新填写') "type=$kind"
PageClick '#pluginList .pl-secret input'; Start-Sleep -Milliseconds 300
[G]::Type($Token); [G]::Press(0x09); Start-Sleep -Seconds 1
Js "document.querySelector('#pluginList .pl-secret input').scrollIntoView({ block: 'center' })" | Out-Null
Shot 'secret-hidden'
PageClick '#pluginList .pl-secret button'; Start-Sleep -Milliseconds 500
$shown = Js "(() => { const i = document.querySelector('#pluginList .pl-secret input'); return i.type + ' ' + i.value + ' ' + document.querySelector('#pluginList .pl-secret button').textContent; })()"
Shot 'secret-shown'
Check '显示 shows what was typed, the button says 隐藏' ($shown -eq "text $Token 隐藏") $shown
Check 'the plugin reads it in pf.settings' (WaitOut "token=$Token") (FenceOut $script:Fid)
Start-Sleep -Seconds 3
$cfg = Get-Content "$script:Config\config.json" -Raw -Encoding UTF8
$sealed = ($cfg | ConvertFrom-Json).scripts."$script:Fid".settings.__secrets.token
Check 'the configuration has it encrypted only' ($sealed -match '^[0-9a-f]{100,}$' -and $cfg -notmatch $Token) "sealed=$("$sealed".Length) chars"
$file = "$Lr\export.json"
Pf config export $file | Out-Null
$ex2 = Get-Content $file -Raw -Encoding UTF8
Check 'an export has no secret' ((Test-Path $file) -and $ex2 -notmatch '__secrets' -and $ex2 -notmatch $Token) ''
Check 'the log has no secret' (@(PfLogAll | Where-Object { $_ -match $Token -or $_ -match $sealed.Substring(0, 24) }).Count -eq 0) ''
Js "(() => { const i = document.querySelector('#pluginList .pl-secret input'); i.value = ''; i.dispatchEvent(new Event('change')); return 1; })()" | Out-Null
function Sealed() { return ((Get-Content "$script:Config\config.json" -Raw -Encoding UTF8 | ConvertFrom-Json).scripts."$script:Fid".settings.__secrets).token }
$gone = WaitOut 'token=none'
for ($i = 0; $i -lt 20 -and (Sealed); $i++) { Start-Sleep -Milliseconds 500 }
Check 'an empty box removes it' ($gone -and -not (Sealed)) (FenceOut $script:Fid)
