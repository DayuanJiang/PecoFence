# GitHub #269, item 6: the generic invitation is not drawn in a fully transparent
# empty fence. Folder status and the invitation at visible opacity still show.
trap { PfTrap $_; continue }

PfStart -Script (DumpScript)
Pf fence create --title EmptyHint | Out-Null

function CheckEmptyText($name, $expected, $description) {
  $actual = DumpVal (DumpOf $name) 'empty_text_visible'
  Check $description ($actual -eq $expected) "empty_text_visible=$actual"
}

Pf fence set EmptyHint opacity clear | Out-Null
CheckEmptyText EmptyHint 'true' 'A translucent empty fence still shows its hint'
Pf fence set EmptyHint opacity transparent | Out-Null
foreach ($layout in @('icons', 'list', 'details')) {
  Pf fence set EmptyHint layout $layout | Out-Null
  CheckEmptyText EmptyHint 'false' "A transparent empty fence hides its hint in $layout"
}
Pf fence set EmptyHint opacity default | Out-Null
CheckEmptyText EmptyHint 'true' 'Restoring opacity restores the hint'

$folder = (New-Item -ItemType Directory -Path (Join-Path $PfRun 'empty-hint-portal')).FullName
Pf fence create --title EmptyPortal --portal $folder | Out-Null
Pf fence set EmptyPortal opacity transparent | Out-Null
CheckEmptyText EmptyPortal 'true' 'A transparent portal still shows its empty-folder status'
