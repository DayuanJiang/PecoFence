# Texts of the third owner round (package ux3-peek), on a box without a running Everything.
#   E  a smart fence that waits for Everything: its sentence does not tell the user to right-click
#      the fence, the row under it starts or installs Everything
trap { PfTrap $_; continue }
# The words Windows' OCR reads in a fence (English interface: the box has no Chinese OCR).
function TxWords($title, $name) { $r = (FenceGet $title).windowRect; return (OcrText $r.x $r.y $r.w $r.h $name) }
function TxSource($title) { try { return "$((FenceGet $title).smart.source)" } catch { return '' } }

Case 'E: a smart fence that waits for Everything'
CloseWindows
EvStop | Out-Null
PfStart -Lang en
Pf settings set theme light | Out-Null
ClearFences
Pf fence create --title 'Whole PC' --everything '会议纪要' --rect '60,60,440,220' | Out-Null
for ($i = 0; $i -lt 20 -and (TxSource 'Whole PC') -ne 'needsEverything'; $i++) { Start-Sleep -Milliseconds 500 }
Check 'E the fence waits for Everything' ((TxSource 'Whole PC') -eq 'needsEverything') "source=$(TxSource 'Whole PC')"
ParkMouse
Start-Sleep -Milliseconds 800
$words = (TxWords 'Whole PC' 'e-en') -replace '\s+', ' '
Check 'E the sentence says that the search needs Everything' ($words -match 'needs Everything') "words='$words'"
Check 'E the sentence does not tell the user to right-click the fence' ($words -notmatch 'right-click') "words='$words'"
Check 'E the row that starts or installs Everything is under it' ($words -match '(Start|Install) Everything') "words='$words'"
$r = (FenceGet 'Whole PC').windowRect
Crop 'e-fence-en' ($r.x - 10) ($r.y - 10) ($r.w + 20) ($r.h + 20) | Out-Null
# The same fence in Chinese, for the picture.
PfStart -Lang zh-CN
for ($i = 0; $i -lt 20 -and (TxSource 'Whole PC') -ne 'needsEverything'; $i++) { Start-Sleep -Milliseconds 500 }
ParkMouse
Start-Sleep -Milliseconds 800
$r = (FenceGet 'Whole PC').windowRect
Crop 'e-fence-zh' ($r.x - 10) ($r.y - 10) ($r.w + 20) ($r.h + 20) | Out-Null
Check 'E the fence still waits for Everything in Chinese' ((TxSource 'Whole PC') -eq 'needsEverything') "source=$(TxSource 'Whole PC')"
ClearFences
