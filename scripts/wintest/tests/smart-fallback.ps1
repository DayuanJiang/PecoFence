# 智能栅栏 without Everything (installed on the box but stopped, see smart.ps1): a folder search
# reads the folder on a worker (the CLI keeps answering meanwhile, a big tree is cut at 20000
# entries), new files still show; a whole-computer search says it needs Everything, its menu
# starts Everything and the fence fills; an imported configuration whose template folder is not
# on this computer finds the local one.
Case 'setup'
CloseWindows
DownloadsScene
Check 'Everything is installed and stopped' ((EvInstalled) -and (EvStop)) "installed=$(EvInstalled) running=$(EvRunning)"
AgedFile $PfDocs 'cover art.psd' 3 | Out-Null
# A big tree for the worker: 21000 files in 21 folders.
$bulk = Join-Path $PfDocs 'smart-bulk'
for ($d = 0; $d -lt 21; $d++) {
  $dir = Join-Path $bulk "part$d"; New-Item -ItemType Directory -Force $dir | Out-Null
  for ($i = 0; $i -lt 1000; $i++) { [IO.File]::WriteAllText((Join-Path $dir "file$i.txt"), 'x') }
}
Check 'the big tree is there' (@(Get-ChildItem -LiteralPath $bulk -Recurse -File).Count -eq 21000) ''
PfStart -Lang en -Script (DumpScript 500 15)
Start-Sleep -Seconds 2
$Dl = 'Downloads of the last 7 days'

Case 'a folder search without Everything'
Pf fence create --smart downloads --rect '420,140,560,330' | Out-Null
Check 'the six files of the last week, read from the folder' (WaitSmart $Dl { $s.source -eq 'folder' -and $s.shown -eq 6 } 20) (SmartLine $Dl)
$log = (PfLogAll) -join "`n"
Check 'log: read on a worker, then the result' ($log -match 'smart fence: reading the folder without Everything' -and $log -match 'smart fence: folder read without Everything.*scanned=\d+.*ms=\d+') ([regex]::Match($log, '.*folder read without Everything.*').Value)
$w = DumpOf $Dl
Check 'the count after the name: · 6' ($w.raw -match 'title_count=Some\(" · 6"\)') ([regex]::Match($w.raw, 'title_count=\S+ \S*').Value)
AgedFile $PfDl 'offline invoice.pdf' 0 | Out-Null
$t0 = Get-Date
Check 'a new file shows without Everything too' (WaitSmart $Dl { $n -match 'offline invoice' } 15) "$([int]((Get-Date) - $t0).TotalSeconds) s; $(SmartLine $Dl)"
Shot 'f01-folder-read'

Case 'a big tree: the worker reads, the CLI keeps answering'
Pf fence create --title 'Bulk' --smart-folder $bulk --rect '1000,140,400,330' | Out-Null
$sw = [Diagnostics.Stopwatch]::StartNew()
Pf status | Out-Null
$statusMs = $sw.ElapsedMilliseconds
Check 'Bulk shows the newest 200 of the files' (WaitSmart 'Bulk' { $s.source -eq 'folder' -and $s.shown -eq 200 } 60) (SmartLine 'Bulk')
$line = [regex]::Match(((PfLogAll) -join "`n"), '.*folder read without Everything[^\n]*scanned=2000\d[^\n]*').Value
Check 'the read stops at 20000 entries' ($line -ne '') $line
$readMs = if ($line -match 'ms=(\d+)') { [int]$Matches[1] } else { -1 }
Check "the CLI answered at once while the folder was read ($statusMs ms, the read took $readMs ms)" ($statusMs -lt 1500) ''
$w = DumpOf 'Bulk'
Check 'the count says more than shown: · 200+' ($w.raw -match 'title_count=Some\(" · 200\+"\)') ([regex]::Match($w.raw, 'title_count=\S+ \S*').Value)
Pf fence delete 'Bulk' | Out-Null

Case 'the whole computer needs Everything'
Pf fence create --smart psd --rect '420,500,560,250' | Out-Null
Check 'All .psd files: empty, waiting for Everything' (WaitSmart 'All .psd files' { $s.source -eq 'needsEverything' -and $s.shown -eq 0 } 10) (SmartLine 'All .psd files')
$w = DumpOf 'All .psd files'
Check 'no count while it cannot search' ($w.raw -match 'title_count=None') ([regex]::Match($w.raw, 'title_count=\S+').Value)
Shot 'f02-needs-everything'
$m = FenceMenuOf 'All .psd files'
$t = MenuTexts $m
Check 'its menu offers Start Everything' ($t -match 'Start Everything') $t
MenuPick '^Start Everything' $m | Out-Null
for ($i = 0; $i -lt 30 -and -not (EvRunning); $i++) { Start-Sleep -Milliseconds 500 }
Check 'Start Everything starts it' (EvRunning) ''
Check 'the fence fills once Everything runs' (WaitSmart 'All .psd files' { $s.source -eq 'everything' -and ($n -match 'cover art') } 40) (SmartLine 'All .psd files')
$log = (PfLogAll) -join "`n"
Check 'log: the waiting fences searched again' ($log -match 'smart fences waiting for Everything search again') ''
Check 'the downloads fence moves to Everything as well' (WaitSmart $Dl { $s.source -eq 'everything' } 80) (SmartLine $Dl)

Case 'an imported configuration finds this computer''s Downloads'
$exp = Join-Path $PfRun 'smart-export.json'
Pf config export $exp | Out-Null
$json = [IO.File]::ReadAllText($exp)
$moved = $json.Replace(($PfDl -replace '\\', '\\'), 'C:\\Users\\someone-else\\Downloads')
[IO.File]::WriteAllText($exp, $moved)
Check 'the export names another Downloads folder now' ($moved -match 'someone-else') ''
Pf config import $exp | Out-Null
Start-Sleep -Seconds 3
Check 'the template fence searches pf''s Downloads again' (WaitSmart $Dl { @($s.folders) -join '|' -eq $PfDl -and $s.shown -ge 6 } 20) "$(@((SmartOf $Dl).folders) -join '|') $(SmartLine $Dl)"
$log = (PfLogAll) -join "`n"
Check 'log: the template folder was found on this computer' ($log -match "smart fence: the template's folder found on this computer") ''
PfQuit
EvStop | Out-Null
Remove-Item -LiteralPath (Join-Path $PfDocs 'cover art.psd') -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $bulk -Recurse -Force -ErrorAction SilentlyContinue
ClearDownloads
