# <pecofence.type>streamable</pecofence.type>
# <pecofence.title>Clipboard History</pecofence.title>
# PecoFence script fence: the history of the Windows clipboard (texts, pictures and files).
#
# The first comment keeps the script running (no 10 second limit); the second one is the title a
# fence made from this script gets. The script watches the clipboard (GetClipboardSequenceNumber,
# every $PollMs milliseconds), keeps what was copied and shows one line per entry, newest first. A
# click on a row reaches the script as a line on its standard input (the row's action=).
#
# At the top of the fence:
#   Show entries       ticked: the entries are readable. Not ticked: they are all blurred (text
#                      and pictures alike), which protects passwords and the like while you stream.
#   Reveal on click    only available while "Show entries" is ticked. Ticked: the entries are
#                      blurred anyway and a click on one shows it for $RevealSeconds seconds.
#                      PecoFence cannot tell the script which row the mouse is over, so this stands
#                      in for "show only on mouse over".
#   Delete all         deletes every entry.
#   Purge entries in: [<] 10 min [>]   (on the line of Delete all, at its right)
#                      when ticked, every entry is deleted that long after the first one
#                      came (the countdown starts again with the next entry). A click on the number
#                      asks for a value; < and > change it.
# Each entry takes the width of the fence, and at its right edge has a trash button that deletes it
# with a pin button under it (a pinned entry goes to the top of the list and is kept by Delete all,
# by the auto-delete and when the list is full, and also when PecoFence starts again). A click on
# the entry copies it to the clipboard again.
#
# Texts, lists of files and pictures come in. What a program marks as not to be kept (the password
# managers do: the clipboard formats "ExcludeClipboardContentFromMonitorProcessing" and
# "CanIncludeInClipboardHistory" = 0) is left out, as Windows' own history leaves it out.
#
# Nothing leaves this computer. The entries are kept in memory only, except the pinned ones, which
# are written to pinned.json in the fence's own folder ($env:PECOFENCE_DATA) so that they are still
# there at the next start. A picture is stored as files in its images folder for as long as its
# entry lives; what is not pinned is removed when the script starts. Settings (the boxes and the
# purge delay) are kept in settings.json next to them. Errors go to clipboard-history.log there.

# ---- What you may want to change ----------------------------------------------------------
$Language = 'auto'                   # 'auto' (PecoFence's language), or 'en', 'fr', 'de', 'es', 'it',
                                     # 'pt-BR', 'ru', 'ja', 'ko', 'zh-CN', 'zh-TW'
$MaxEntries = 30                     # entries kept, the pinned ones not counted (the oldest go first)
$MaxTextChars = 200000               # characters kept of one text
$TextWidth = 250                     # width of the blurred picture of a hidden text, in DIPs (a shown text takes the fence's width)
$ImageHeight = 110                   # height of a picture, in DIPs
$RevealSeconds = 5                   # how long a click shows a blurred entry
$PollMs = 250                        # how often the clipboard and the clicks are looked at
$DefaultSeconds = 600                # the purge delay at the first start, in seconds

# ---- Texts --------------------------------------------------------------------------------
$Strings = @{
  'en' = @{
    show = 'Show entries'; reveal = 'Reveal on click'
    revealTip = 'Entries stay blurred; a click shows one for {0} seconds (PecoFence cannot tell the script which row the mouse is over)'
    showTip = 'Untick to blur every entry, for instance while you stream'
    deleteAll = 'Delete all'; auto = 'Purge entries in:'; minutes = 'min'; seconds = 's'
    autoTip = 'Delete every entry this long after the first one came'
    valueTip = 'Click to type minutes and seconds'; less = 'Less'; more = 'More'
    pin = 'Pin'; unpin = 'Unpin'; revealNeeds = 'Tick "Show entries" first'
    empty = 'Nothing copied yet. What you copy shows here.'
    delTip = 'Delete this entry'; copyTip = 'Click to copy it again'; revealTipRow = 'Click to show it for {0} seconds'
    image = 'Image'; files = 'files'; file = 'file'; chars = 'characters'
    dlgTitle = 'Auto-delete'; dlgPrompt = 'Delete all entries after how long?'; cancel = 'Cancel'
  }
  'fr' = @{
    show = 'Afficher les entrées'; reveal = 'Révéler au clic'
    revealTip = 'Les entrées restent floutées ; un clic en montre une pendant {0} secondes (PecoFence ne dit pas au script sur quelle ligne est la souris)'
    showTip = 'Décochez pour flouter toutes les entrées, par exemple pendant un direct'
    deleteAll = 'Tout supprimer'; auto = 'Purger les entrées dans :'; minutes = 'min'; seconds = 's'
    autoTip = 'Supprime toutes les entrées après ce délai, compté depuis l''arrivée de la première'
    valueTip = 'Cliquez pour saisir minutes et secondes'; less = 'Moins'; more = 'Plus'
    pin = 'Épingler'; unpin = 'Désépingler'; revealNeeds = 'Cochez d''abord « Afficher les entrées »'
    empty = 'Rien n''a encore été copié. Ce que vous copiez apparaît ici.'
    delTip = 'Supprimer cette entrée'; copyTip = 'Cliquez pour la copier à nouveau'; revealTipRow = 'Cliquez pour l''afficher pendant {0} secondes'
    image = 'Image'; files = 'fichiers'; file = 'fichier'; chars = 'caractères'
    dlgTitle = 'Suppression automatique'; dlgPrompt = 'Supprimer toutes les entrées après combien de temps ?'; cancel = 'Annuler'
  }
  'de' = @{
    show = 'Einträge anzeigen'; reveal = 'Per Klick zeigen'
    revealTip = 'Einträge bleiben unscharf; ein Klick zeigt einen für {0} Sekunden (PecoFence meldet dem Skript nicht, über welcher Zeile die Maus steht)'
    showTip = 'Abwählen, um alle Einträge unscharf zu machen, z. B. bei einem Livestream'
    deleteAll = 'Alle löschen'; auto = 'Einträge bereinigen in:'; minutes = 'Min.'; seconds = 'Sek.'
    autoTip = 'Löscht alle Einträge nach dieser Zeit, ab dem ersten'
    valueTip = 'Klicken, um Minuten und Sekunden einzutippen'; less = 'Weniger'; more = 'Mehr'
    pin = 'Anheften'; unpin = 'Lösen'; revealNeeds = 'Zuerst „Einträge anzeigen“ aktivieren'
    empty = 'Noch nichts kopiert. Was Sie kopieren, erscheint hier.'
    delTip = 'Diesen Eintrag löschen'; copyTip = 'Klicken, um ihn erneut zu kopieren'; revealTipRow = 'Klicken, um ihn {0} Sekunden zu zeigen'
    image = 'Bild'; files = 'Dateien'; file = 'Datei'; chars = 'Zeichen'
    dlgTitle = 'Automatisch löschen'; dlgPrompt = 'Alle Einträge nach welcher Zeit löschen?'; cancel = 'Abbrechen'
  }
  'es' = @{
    show = 'Mostrar entradas'; reveal = 'Mostrar al hacer clic'
    revealTip = 'Las entradas siguen desenfocadas; un clic muestra una durante {0} segundos (PecoFence no indica al script sobre qué fila está el ratón)'
    showTip = 'Desmarque para desenfocar todas las entradas, por ejemplo durante una emisión en directo'
    deleteAll = 'Borrar todo'; auto = 'Purgar entradas en:'; minutes = 'min'; seconds = 's'
    autoTip = 'Borra todas las entradas pasado este tiempo desde la primera'
    valueTip = 'Haga clic para escribir minutos y segundos'; less = 'Menos'; more = 'Más'
    pin = 'Anclar'; unpin = 'Desanclar'; revealNeeds = 'Marque primero «Mostrar entradas»'
    empty = 'Aún no se ha copiado nada. Lo que copie aparecerá aquí.'
    delTip = 'Borrar esta entrada'; copyTip = 'Haga clic para copiarla de nuevo'; revealTipRow = 'Haga clic para mostrarla {0} segundos'
    image = 'Imagen'; files = 'archivos'; file = 'archivo'; chars = 'caracteres'
    dlgTitle = 'Borrado automático'; dlgPrompt = '¿Borrar todas las entradas pasado cuánto tiempo?'; cancel = 'Cancelar'
  }
  'it' = @{
    show = 'Mostra voci'; reveal = 'Mostra al clic'
    revealTip = 'Le voci restano sfocate; un clic ne mostra una per {0} secondi (PecoFence non dice allo script su quale riga si trova il mouse)'
    showTip = 'Deseleziona per sfocare tutte le voci, ad esempio durante una diretta'
    deleteAll = 'Elimina tutto'; auto = 'Svuota le voci tra:'; minutes = 'min'; seconds = 's'
    autoTip = 'Elimina tutte le voci dopo questo tempo dalla prima'
    valueTip = 'Clicca per digitare minuti e secondi'; less = 'Meno'; more = 'Più'
    pin = 'Fissa'; unpin = 'Rimuovi'; revealNeeds = 'Seleziona prima «Mostra voci»'
    empty = 'Ancora niente di copiato. Ciò che copi compare qui.'
    delTip = 'Elimina questa voce'; copyTip = 'Clicca per copiarla di nuovo'; revealTipRow = 'Clicca per mostrarla per {0} secondi'
    image = 'Immagine'; files = 'file'; file = 'file'; chars = 'caratteri'
    dlgTitle = 'Eliminazione automatica'; dlgPrompt = 'Eliminare tutte le voci dopo quanto tempo?'; cancel = 'Annulla'
  }
  'pt-BR' = @{
    show = 'Mostrar entradas'; reveal = 'Mostrar ao clicar'
    revealTip = 'As entradas continuam desfocadas; um clique mostra uma por {0} segundos (o PecoFence não informa ao script em qual linha está o mouse)'
    showTip = 'Desmarque para desfocar todas as entradas, por exemplo durante uma transmissão ao vivo'
    deleteAll = 'Excluir tudo'; auto = 'Limpar entradas em:'; minutes = 'min'; seconds = 's'
    autoTip = 'Exclui todas as entradas depois deste tempo, contado da primeira'
    valueTip = 'Clique para digitar minutos e segundos'; less = 'Menos'; more = 'Mais'
    pin = 'Fixar'; unpin = 'Desafixar'; revealNeeds = 'Marque primeiro «Mostrar entradas»'
    empty = 'Nada copiado ainda. O que você copiar aparece aqui.'
    delTip = 'Excluir esta entrada'; copyTip = 'Clique para copiá-la de novo'; revealTipRow = 'Clique para mostrá-la por {0} segundos'
    image = 'Imagem'; files = 'arquivos'; file = 'arquivo'; chars = 'caracteres'
    dlgTitle = 'Exclusão automática'; dlgPrompt = 'Excluir todas as entradas depois de quanto tempo?'; cancel = 'Cancelar'
  }
  'ru' = @{
    show = 'Показывать записи'; reveal = 'Показать по щелчку'
    revealTip = 'Записи остаются размытыми; щелчок показывает одну на {0} с (PecoFence не сообщает скрипту, над какой строкой мышь)'
    showTip = 'Снимите флажок, чтобы размыть все записи, например во время трансляции'
    deleteAll = 'Удалить все'; auto = 'Очистка записей через:'; minutes = 'мин'; seconds = 'сек'
    autoTip = 'Удаляет все записи через это время после появления первой'
    valueTip = 'Щёлкните, чтобы ввести минуты и секунды'; less = 'Меньше'; more = 'Больше'
    pin = 'Закрепить'; unpin = 'Открепить'; revealNeeds = 'Сначала включите «Показывать записи»'
    empty = 'Пока ничего не скопировано. Скопированное появится здесь.'
    delTip = 'Удалить эту запись'; copyTip = 'Щёлкните, чтобы скопировать снова'; revealTipRow = 'Щёлкните, чтобы показать на {0} с'
    image = 'Изображение'; files = 'файлов'; file = 'файл'; chars = 'символов'
    dlgTitle = 'Автоудаление'; dlgPrompt = 'Через какое время удалить все записи?'; cancel = 'Отмена'
  }
  'ja' = @{
    show = '項目を表示'; reveal = 'クリックで表示'
    revealTip = '項目はぼかしたままで、クリックすると{0}秒間だけ表示します（マウスが行の上にあることはスクリプトに伝わらないため）'
    showTip = 'チェックを外すとすべての項目をぼかします（配信中など）'
    deleteAll = 'すべて削除'; auto = '次の時間後に項目を消去:'; minutes = '分'; seconds = '秒'
    autoTip = '最初の項目が入ってから、この時間後にすべての項目を削除します'
    valueTip = 'クリックして分と秒を入力'; less = '減らす'; more = '増やす'
    pin = 'ピン留め'; unpin = 'ピン留めを外す'; revealNeeds = '先に「項目を表示」にチェックを入れてください'
    empty = 'まだ何もコピーされていません。コピーしたものがここに表示されます。'
    delTip = 'この項目を削除'; copyTip = 'クリックでもう一度コピー'; revealTipRow = 'クリックで{0}秒間表示'
    image = '画像'; files = '個のファイル'; file = '個のファイル'; chars = '文字'
    dlgTitle = '自動削除'; dlgPrompt = 'どのくらい後にすべての項目を削除しますか？'; cancel = 'キャンセル'
  }
  'ko' = @{
    show = '항목 표시'; reveal = '클릭하여 표시'
    revealTip = '항목은 흐리게 유지되며, 클릭하면 {0}초 동안 하나를 보여 줍니다(PecoFence가 마우스가 있는 줄을 스크립트에 알려 주지 않기 때문입니다)'
    showTip = '선택을 해제하면 모든 항목을 흐리게 합니다(예: 방송 중)'
    deleteAll = '모두 삭제'; auto = '다음 시간 후 항목 비우기:'; minutes = '분'; seconds = '초'
    autoTip = '첫 항목이 들어온 뒤 이 시간이 지나면 모든 항목을 삭제합니다'
    valueTip = '클릭하여 분과 초 입력'; less = '줄이기'; more = '늘리기'
    pin = '고정'; unpin = '고정 해제'; revealNeeds = '먼저 ''항목 표시''를 선택하세요'
    empty = '아직 복사한 것이 없습니다. 복사한 내용이 여기에 표시됩니다.'
    delTip = '이 항목 삭제'; copyTip = '클릭하여 다시 복사'; revealTipRow = '클릭하여 {0}초 동안 표시'
    image = '이미지'; files = '개 파일'; file = '개 파일'; chars = '자'
    dlgTitle = '자동 삭제'; dlgPrompt = '얼마 후에 모든 항목을 삭제할까요?'; cancel = '취소'
  }
  'zh-CN' = @{
    show = '显示条目'; reveal = '点击时显示'
    revealTip = '条目保持模糊；点一下显示 {0} 秒（PecoFence 不会告诉脚本鼠标在哪一行上）'
    showTip = '取消勾选会模糊所有条目，例如直播时'
    deleteAll = '全部删除'; auto = '清除条目，时间：'; minutes = '分钟'; seconds = '秒'
    autoTip = '第一条出现后经过这段时间，删除所有条目'
    valueTip = '点击输入分钟和秒数'; less = '减少'; more = '增加'
    pin = '固定'; unpin = '取消固定'; revealNeeds = '请先勾选「显示条目」'
    empty = '还没有复制任何内容。复制的内容会显示在这里。'
    delTip = '删除此条目'; copyTip = '点击再次复制'; revealTipRow = '点击显示 {0} 秒'
    image = '图片'; files = '个文件'; file = '个文件'; chars = '个字符'
    dlgTitle = '自动删除'; dlgPrompt = '多长时间后删除所有条目？'; cancel = '取消'
  }
  'zh-TW' = @{
    show = '顯示項目'; reveal = '點擊時顯示'
    revealTip = '項目保持模糊；點一下顯示 {0} 秒（PecoFence 不會告訴指令碼滑鼠在哪一列上）'
    showTip = '取消勾選會模糊所有項目，例如直播時'
    deleteAll = '全部刪除'; auto = '清除項目，時間：'; minutes = '分鐘'; seconds = '秒'
    autoTip = '第一筆出現後經過這段時間，刪除所有項目'
    valueTip = '點擊輸入分鐘和秒數'; less = '減少'; more = '增加'
    pin = '固定'; unpin = '取消固定'; revealNeeds = '請先勾選「顯示項目」'
    empty = '尚未複製任何內容。複製的內容會顯示在這裡。'
    delTip = '刪除此項目'; copyTip = '點擊再次複製'; revealTipRow = '點擊顯示 {0} 秒'
    image = '圖片'; files = '個檔案'; file = '個檔案'; chars = '個字元'
    dlgTitle = '自動刪除'; dlgPrompt = '多久後刪除所有項目？'; cancel = '取消'
  }
}

# PecoFence's display language (it tells the script in PECOFENCE_LANGUAGE), else Windows', else English.
function Resolve-Language([string]$wanted) {
  if ($Strings.ContainsKey($wanted)) { return $wanted }
  $culture = (Get-UICulture).Name
  if ($Strings.ContainsKey($culture)) { return $culture }
  switch -Regex ($culture) {
    '^zh-(TW|HK|MO|Hant)' { return 'zh-TW' }
    '^zh' { return 'zh-CN' }
    '^pt' { return 'pt-BR' }
  }
  $short = (Get-UICulture).TwoLetterISOLanguageName
  if ($Strings.ContainsKey($short)) { return $short }
  'en'
}
if ($Language -eq 'auto') { $Language = $env:PECOFENCE_LANGUAGE }
$T = $Strings[(Resolve-Language $Language)]

# ---- Files and settings -------------------------------------------------------------------
$DataDir = if ($env:PECOFENCE_DATA) { $env:PECOFENCE_DATA } else { Join-Path $env:TEMP 'pecofence-clipboard' }
$ImageDir = Join-Path $DataDir 'images'
$SettingsFile = Join-Path $DataDir 'settings.json'
$LogFile = Join-Path $DataDir 'clipboard-history.log'
$Dark = $env:PECOFENCE_THEME -eq 'dark'
[void](New-Item -ItemType Directory -Force -Path $DataDir)

function Write-Log([string]$message) {
  try { Add-Content -LiteralPath $LogFile -Value ('{0:s} {1}' -f (Get-Date), $message) -Encoding UTF8 } catch { }
}

$PinnedFile = Join-Path $DataDir 'pinned.json'
try { [void](New-Item -ItemType Directory -Force -Path $ImageDir) } catch { Write-Log "Images folder: $($_.Exception.Message)" }

$cfg = @{ show = $true; reveal = $false; auto = $false; seconds = $DefaultSeconds }
try {
  if (Test-Path -LiteralPath $SettingsFile) {
    $saved = Get-Content -LiteralPath $SettingsFile -Raw -Encoding UTF8 | ConvertFrom-Json
    foreach ($name in 'show', 'reveal', 'auto') { if ($null -ne $saved.$name) { $cfg[$name] = [bool]$saved.$name } }
    if ($saved.seconds -gt 0) { $cfg.seconds = [Math]::Min(86400, [int]$saved.seconds) }
    elseif ($saved.minutes -gt 0) { $cfg.seconds = [Math]::Min(86400, 60 * [int]$saved.minutes) }
  }
} catch { Write-Log "Settings: $($_.Exception.Message)" }

function Save-Settings {
  try { $cfg | ConvertTo-Json | Set-Content -LiteralPath $SettingsFile -Encoding UTF8 } catch { Write-Log "Save: $($_.Exception.Message)" }
}

# ---- State --------------------------------------------------------------------------------
$State = @{
  Entries = New-Object System.Collections.Generic.List[object]   # newest first
  NextId = 1
  LastSeq = 0                                                     # the clipboard's sequence number when it was last looked at
  ReadAt = $null                                                  # when to read after a change
  Deadline = $null                                                # when the purge empties the list
  Revealed = @{}                                                  # id -> when it is blurred again
  LastScreen = ''
  Dialog = $null
  Ready = $false                                                  # the pinned entries are loaded
}

# ---- Rows ---------------------------------------------------------------------------------
function Row([string]$text, [string]$params = '') {
  $clean = ($text -replace '[\r\n]+', ' ' -replace '\|', [string][char]0x00A6).Trim()
  if ($clean.Length -gt 200) { $clean = $clean.Substring(0, 199) + [string][char]0x2026 }
  if ($params) { "$clean | $params" } else { $clean }
}

# A cell of a row of cells: `text | parameters`. A text that starts with [ ] or [x] would be a
# to-do, so a zero width space goes before the bracket.
function Cell([string]$text, [string]$params = '') {
  $clean = ($text -replace '[\r\n]+', ' ' -replace '\|', [string][char]0x00A6 -replace '::', ': :').Trim()
  if ($clean.StartsWith('[')) { $clean = [string][char]0x200B + $clean }
  if ($params) { "$clean | $params" } else { $clean }
}

function Tip([string]$text) { ($text -replace '"', "'" -replace '\|', [string][char]0x00A6 -replace '::', ': :' -replace "`r?`n", '\n') }

function Format-Flag([bool]$value) { if ($value) { 'true' } else { 'false' } }

# A whole screen in one write, after a ~~~ line; nothing is written when it did not change.
function Show-Screen([string[]]$rows) {
  $screen = $rows -join "`n"
  if ($screen -ceq $State.LastScreen) { return }
  $State.LastScreen = $screen
  Write-Output "~~~`n$screen"
}

# ---- Pictures -----------------------------------------------------------------------------
function New-Canvas([int]$w, [int]$h) {
  New-Object System.Drawing.Bitmap $w, $h, ([System.Drawing.Imaging.PixelFormat]::Format32bppPArgb)
}

# `source` blurred: shrunk to about 1/factor of its size and smoothed back up.
function New-Blur([System.Drawing.Bitmap]$source, [double]$factor) {
  $w = $source.Width; $h = $source.Height
  $sw = [Math]::Max(2, [int][Math]::Round($w / $factor)); $sh = [Math]::Max(2, [int][Math]::Round($h / $factor))
  $small = New-Canvas $sw $sh
  $attrs = New-Object System.Drawing.Imaging.ImageAttributes
  $attrs.SetWrapMode([System.Drawing.Drawing2D.WrapMode]::TileFlipXY)
  $g = [System.Drawing.Graphics]::FromImage($small)
  $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBilinear
  $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
  $g.DrawImage($source, (New-Object System.Drawing.Rectangle 0, 0, $sw, $sh), 0, 0, $w, $h, [System.Drawing.GraphicsUnit]::Pixel, $attrs)
  $g.Dispose()
  $out = New-Canvas $w $h
  $g = [System.Drawing.Graphics]::FromImage($out)
  $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
  $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
  $g.DrawImage($small, (New-Object System.Drawing.Rectangle 0, 0, $w, $h), 0, 0, $sw, $sh, [System.Drawing.GraphicsUnit]::Pixel, $attrs)
  $g.Dispose(); $small.Dispose(); $attrs.Dispose()
  $out
}

# The picture of an entry as files: the original (to copy it again), a small one to show and its
# blurred twin.
function Save-Picture([System.Drawing.Image]$image, [byte[]]$png, [int]$id) {
  $w = $image.Width; $h = $image.Height
  $scale = [Math]::Min(1.0, [Math]::Min(640.0 / $w, 260.0 / $h))
  $tw = [Math]::Max(1, [int][Math]::Round($w * $scale)); $th = [Math]::Max(1, [int][Math]::Round($h * $scale))
  $thumb = New-Canvas $tw $th
  $g = [System.Drawing.Graphics]::FromImage($thumb)
  $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
  $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
  $g.DrawImage($image, 0, 0, $tw, $th)
  $g.Dispose()
  $blur = New-Blur $thumb ([Math]::Max(4.0, [Math]::Max($tw, $th) / 14.0))
  $files = @{
    Full = Join-Path $ImageDir "$id.png"
    Thumb = Join-Path $ImageDir "$id.t.png"
    Blur = Join-Path $ImageDir "$id.b.png"
  }
  [System.IO.File]::WriteAllBytes($files.Full, $png)
  $thumb.Save($files.Thumb, [System.Drawing.Imaging.ImageFormat]::Png)
  $blur.Save($files.Blur, [System.Drawing.Imaging.ImageFormat]::Png)
  $thumb.Dispose(); $blur.Dispose()
  # As tall as it is drawn: the height asked for, or the picture's own when it is smaller.
  $files.Height = [Math]::Max(8.0, [Math]::Min([double]$ImageHeight, [double]$th))
  $files
}

# One line of text, as the fence would show it, drawn and blurred: what a hidden text shows. The
# height of the picture is returned.
function Save-TextBlur([string]$text, [string]$path) {
  $size = [System.Windows.Forms.TextRenderer]::MeasureText($text, $BodyFont, (New-Object System.Drawing.Size 4000, 40), [System.Windows.Forms.TextFormatFlags]::NoPadding)
  $w = [Math]::Max(24, [Math]::Min([int]$TextWidth, $size.Width) + 16); $h = 24
  $bitmap = New-Canvas $w $h
  $g = [System.Drawing.Graphics]::FromImage($bitmap)
  $g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAlias
  $brush = New-Object System.Drawing.SolidBrush $TextColor
  $g.DrawString($text, $BodyFont, $brush, 8.0, 3.0)
  $g.Dispose(); $brush.Dispose()
  $blur = New-Blur $bitmap 6.0
  $blur.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
  $bitmap.Dispose(); $blur.Dispose()
  $h
}

# The text as one line that fits $TextWidth DIPs, with "…" where it was cut.
function Get-OneLine([string]$text) {
  $line = ($text -replace '\s+', ' ').Trim()
  if ($line.Length -gt 300) { $line = $line.Substring(0, 300) }
  $flags = [System.Windows.Forms.TextFormatFlags]::NoPadding
  $box = New-Object System.Drawing.Size 4000, 40
  if ([System.Windows.Forms.TextRenderer]::MeasureText($line, $BodyFont, $box, $flags).Width -le $TextWidth) { return $line }
  $lo = 1; $hi = $line.Length
  while ($lo -lt $hi) {
    $mid = [int][Math]::Ceiling(($lo + $hi) / 2)
    $width = [System.Windows.Forms.TextRenderer]::MeasureText($line.Substring(0, $mid) + [char]0x2026, $BodyFont, $box, $flags).Width
    if ($width -le $TextWidth) { $lo = $mid } else { $hi = $mid - 1 }
  }
  $line.Substring(0, $lo).TrimEnd() + [char]0x2026
}

# ---- The clipboard ------------------------------------------------------------------------
# GetClipboardSequenceNumber (it changes each time the clipboard does) through a dynamic assembly:
# much quicker to start than compiling C# with Add-Type.
function New-NativeType {
  $assembly = [AppDomain]::CurrentDomain.DefineDynamicAssembly((New-Object System.Reflection.AssemblyName 'PfNative'), [System.Reflection.Emit.AssemblyBuilderAccess]::Run)
  $type = $assembly.DefineDynamicModule('PfNative').DefineType('Native', [System.Reflection.TypeAttributes]'Public, Class')
  $method = $type.DefinePInvokeMethod('GetClipboardSequenceNumber', 'user32.dll', [System.Reflection.MethodAttributes]'Public, Static, PinvokeImpl',
    [System.Reflection.CallingConventions]::Standard, [uint32], [Type[]]@(), [System.Runtime.InteropServices.CallingConvention]::Winapi, [System.Runtime.InteropServices.CharSet]::Auto)
  $method.SetImplementationFlags($method.GetMethodImplementationFlags() -bor [System.Reflection.MethodImplAttributes]::PreserveSig)
  $type.CreateType()
}

function Get-ClipboardSequence { $NativeType::GetClipboardSequenceNumber() }

function Get-Hash([byte[]]$bytes) {
  $sha = [System.Security.Cryptography.SHA1]::Create()
  try { ([BitConverter]::ToString($sha.ComputeHash($bytes))) } finally { $sha.Dispose() }
}

# What the clipboard holds now as an entry (without an id), or $null when there is nothing to keep.
function Read-Clipboard {
  $data = $null
  for ($try = 0; $try -lt 5 -and $null -eq $data; $try++) {
    try { $data = [System.Windows.Forms.Clipboard]::GetDataObject() } catch { Start-Sleep -Milliseconds 40 }
  }
  if ($null -eq $data) { return $null }
  # What a program asks not to be kept (password managers).
  if ($data.GetDataPresent('ExcludeClipboardContentFromMonitorProcessing')) { return $null }
  if ($data.GetDataPresent('CanIncludeInClipboardHistory')) {
    try {
      $flag = $data.GetData('CanIncludeInClipboardHistory')
      if ($flag -is [System.IO.MemoryStream]) {
        $bytes = $flag.ToArray()
        if ($bytes.Length -ge 4 -and [BitConverter]::ToUInt32($bytes, 0) -eq 0) { return $null }
      }
    } catch { }
  }
  if ($data.GetDataPresent([System.Windows.Forms.DataFormats]::UnicodeText)) {
    $text = [string]$data.GetData([System.Windows.Forms.DataFormats]::UnicodeText)
    if ($text.Trim()) {
      if ($text.Length -gt $MaxTextChars) { $text = $text.Substring(0, $MaxTextChars) }
      return @{ Kind = 'text'; Text = $text; Hash = 't' + (Get-Hash ([Text.Encoding]::UTF8.GetBytes($text))) }
    }
  }
  if ($data.GetDataPresent([System.Windows.Forms.DataFormats]::FileDrop)) {
    $paths = @($data.GetData([System.Windows.Forms.DataFormats]::FileDrop) | ForEach-Object { [string]$_ })
    if ($paths.Count) {
      return @{ Kind = 'files'; Paths = $paths; Text = ($paths -join "`n"); Hash = 'f' + (Get-Hash ([Text.Encoding]::UTF8.GetBytes($paths -join '|'))) }
    }
  }
  if ($data.GetDataPresent([System.Windows.Forms.DataFormats]::Bitmap)) {
    $image = $null
    try { $image = [System.Windows.Forms.Clipboard]::GetImage() } catch { }
    if ($image) {
      # Encoded once here: its hash tells that the same picture copied twice is the same entry,
      # and the bytes are what is stored when it is a new one.
      $ms = New-Object System.IO.MemoryStream
      try {
        $image.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
        $png = $ms.ToArray()
      } finally { $ms.Dispose() }
      return @{ Kind = 'image'; Image = $image; Png = $png; Hash = 'i' + (Get-Hash $png) }
    }
  }
  $null
}

function Remove-EntryFiles($entry) {
  foreach ($name in 'Full', 'Thumb', 'Blur') {
    if ($entry.$name) { try { Remove-Item -LiteralPath $entry.$name -Force -ErrorAction SilentlyContinue } catch { } }
  }
}

# The entries that are not pinned, newest first.
function Get-Unpinned { @($State.Entries | Where-Object { -not $_.Pinned }) }

function Remove-Entry($entry) {
  [void]$State.Entries.Remove($entry)
  $State.Revealed.Remove($entry.Id)
  Remove-EntryFiles $entry
  if ($entry.Pinned) { Save-Pinned }
  if ((Get-Unpinned).Count -eq 0) { $State.Deadline = $null }
}

# Everything but the pinned entries.
function Clear-Entries {
  foreach ($entry in $State.Entries.ToArray()) {
    if ($entry.Pinned) { continue }
    Remove-EntryFiles $entry
    [void]$State.Entries.Remove($entry)
    $State.Revealed.Remove($entry.Id)
  }
  $State.Deadline = $null
}

# The pinned entries are written down, to be there at the next start.
function Save-Pinned {
  try {
    $items = @($State.Entries | Where-Object { $_.Pinned } | ForEach-Object {
        $item = [ordered]@{ Id = $_.Id; Kind = $_.Kind; Hash = $_.Hash; At = $_.At.ToString('o') }
        if ($_.Kind -eq 'image') { $item.Size = $_.Size; $item.ImageHeight = $_.ImageHeight } else { $item.Text = $_.Text }
        if ($_.Kind -eq 'files') { $item.Paths = @($_.Paths) }
        $item
      })
    if ($items.Count) {
      ConvertTo-Json -InputObject $items -Depth 4 | Set-Content -LiteralPath $PinnedFile -Encoding UTF8
    } else {
      Remove-Item -LiteralPath $PinnedFile -Force -ErrorAction SilentlyContinue
    }
  } catch { Write-Log "Pinned: $($_.Exception.Message)" }
}

# What a text or a list of files needs to be shown blurred: one line of it, drawn and blurred.
function Initialize-TextEntry($entry) {
  if ($entry.Kind -eq 'files') {
    $first = Split-Path -Leaf $entry.Paths[0]
    $line = if ($entry.Paths.Count -gt 1) { "$first (+$($entry.Paths.Count - 1))" } else { $entry.Paths[0] }
  } else {
    $line = $entry.Text
    $entry.Chars = $entry.Text.Length
  }
  $entry.Blur = Join-Path $ImageDir "$($entry.Id).b.png"
  $entry.BlurHeight = Save-TextBlur (Get-OneLine $line) $entry.Blur
}

# The pinned entries of the last run come back; the pictures of the others are removed.
function Restore-Pinned {
  $keep = @{}
  try {
    if (Test-Path -LiteralPath $PinnedFile) {
      $items = @(Get-Content -LiteralPath $PinnedFile -Raw -Encoding UTF8 | ConvertFrom-Json)
      foreach ($item in $items) {
        $id = [int]$item.Id
        $at = [DateTime]::Parse([string]$item.At, [System.Globalization.CultureInfo]::InvariantCulture)
        $entry = @{ Id = $id; Kind = [string]$item.Kind; Hash = [string]$item.Hash; At = $at; Pinned = $true }
        if ($entry.Kind -eq 'image') {
          $entry.Full = Join-Path $ImageDir "$id.png"; $entry.Thumb = Join-Path $ImageDir "$id.t.png"; $entry.Blur = Join-Path $ImageDir "$id.b.png"
          if (-not ((Test-Path -LiteralPath $entry.Full) -and (Test-Path -LiteralPath $entry.Thumb) -and (Test-Path -LiteralPath $entry.Blur))) { continue }
          $entry.Size = [string]$item.Size; $entry.ImageHeight = [double]$item.ImageHeight
        } else {
          $entry.Text = [string]$item.Text
          if ($entry.Kind -eq 'files') { $entry.Paths = @($item.Paths | ForEach-Object { [string]$_ }) }
          Initialize-TextEntry $entry
        }
        $State.Entries.Add($entry)
        $keep["$id"] = $true
        if ($id -ge $State.NextId) { $State.NextId = $id + 1 }
      }
    }
  } catch { Write-Log "Restore: $($_.Exception.Message)" }
  try {
    Get-ChildItem -LiteralPath $ImageDir -File -ErrorAction SilentlyContinue |
      Where-Object { -not $keep.ContainsKey(($_.Name -split '\.')[0]) } |
      Remove-Item -Force -ErrorAction SilentlyContinue
  } catch { Write-Log "Images folder: $($_.Exception.Message)" }
}

function Add-Entry($found) {
  # Copied again: the entry goes back to the top (a pinned one stays where it is).
  $same = $State.Entries | Where-Object { $_.Hash -eq $found.Hash } | Select-Object -First 1
  if ($same) {
    if ($found.Image) { $found.Image.Dispose() }
    if (-not $same.Pinned) {
      [void]$State.Entries.Remove($same)
      $State.Entries.Insert(0, $same)
      $same.At = Get-Date
    }
    return
  }
  $id = $State.NextId++
  $entry = @{ Id = $id; Kind = $found.Kind; Hash = $found.Hash; At = Get-Date; Pinned = $false }
  switch ($found.Kind) {
    'text' {
      $entry.Text = $found.Text
      Initialize-TextEntry $entry
    }
    'files' {
      $entry.Paths = $found.Paths
      $entry.Text = $found.Text
      Initialize-TextEntry $entry
    }
    'image' {
      try {
        $files = Save-Picture $found.Image $found.Png $id
        $entry.Full = $files.Full; $entry.Thumb = $files.Thumb; $entry.Blur = $files.Blur
        $entry.ImageHeight = $files.Height
        $entry.Size = '{0} x {1}' -f $found.Image.Width, $found.Image.Height
      } finally { $found.Image.Dispose() }
    }
  }
  $State.Entries.Insert(0, $entry)
  while ((Get-Unpinned).Count -gt $MaxEntries) { Remove-Entry (Get-Unpinned)[-1] }
  if ($cfg.auto -and -not $State.Deadline) { $State.Deadline = (Get-Date).AddSeconds($cfg.seconds) }
}

# Puts an entry back on the clipboard; what the clipboard then holds is not read as a new copy.
function Copy-Entry($entry) {
  try {
    switch ($entry.Kind) {
      'text' { [System.Windows.Forms.Clipboard]::SetDataObject([string]$entry.Text, $true) }
      'files' {
        $list = New-Object System.Collections.Specialized.StringCollection
        $entry.Paths | ForEach-Object { [void]$list.Add($_) }
        [System.Windows.Forms.Clipboard]::SetFileDropList($list)
      }
      'image' {
        $loaded = New-Object System.Drawing.Bitmap $entry.Full
        try { [System.Windows.Forms.Clipboard]::SetImage($loaded) } finally { $loaded.Dispose() }
      }
    }
  } catch { Write-Log "Copy: $($_.Exception.Message)" }
  $State.LastSeq = Get-ClipboardSequence
  $State.ReadAt = $null
}

function Update-Clipboard {
  $seq = Get-ClipboardSequence
  if ($seq -ne $State.LastSeq) {
    # Programs set the clipboard in steps: it is read once it has been quiet for a moment.
    $State.LastSeq = $seq
    $State.ReadAt = [DateTime]::UtcNow.AddMilliseconds(150)
    return $false
  }
  if ($State.ReadAt -and [DateTime]::UtcNow -ge $State.ReadAt) {
    $State.ReadAt = $null
    $found = Read-Clipboard
    if ($found) { Add-Entry $found; return $true }
  }
  $false
}

# ---- The screen ---------------------------------------------------------------------------
# Blurred: always while "Show entries" is not ticked; otherwise, with "Reveal on click", until the
# entry is clicked.
function Test-Hidden($entry) {
  if (-not $cfg.show) { return $true }
  if (-not $cfg.reveal) { return $false }
  $until = $State.Revealed[$entry.Id]
  -not ($until -and $until -gt [DateTime]::UtcNow)
}

function Get-Screen {
  $rows = New-Object System.Collections.Generic.List[string]
  $trash = [string][char]0xE74D
  # The two boxes; "Reveal on click" works while the entries are shown only: without that its
  # cell has no action.
  $showCell = Cell $T.show "checkbox=$(Format-Flag $cfg.show) action=toggle-show tooltip=""$(Tip $T.showTip)"""
  $revealCell = if ($cfg.show) {
    Cell $T.reveal "checkbox=$(Format-Flag $cfg.reveal) action=toggle-reveal tooltip=""$(Tip ($T.revealTip -f $RevealSeconds))"""
  } else {
    Cell $T.reveal "checkbox=$(Format-Flag $cfg.reveal) color=gray tooltip=""$(Tip $T.revealNeeds)"""
  }
  $rows.Add(':: ' + $showCell + ' :: ' + $revealCell)
  $left = ''
  if ($cfg.auto -and $State.Deadline) {
    # minutes:seconds, counting down in real time (the screen is printed again when it changes).
    $remaining = [Math]::Max(0, [int][Math]::Ceiling(($State.Deadline - (Get-Date)).TotalSeconds))
    $left = '{0:00}:{1:00}' -f [int][Math]::Floor($remaining / 60), ($remaining % 60)
  }
  $autoCells = @(
    (Cell $T.auto "checkbox=$(Format-Flag $cfg.auto) action=toggle-auto tooltip=""$(Tip $T.autoTip)"""),
    (Cell ([string][char]0xE76B) "action=auto-less tooltip=""$($T.less)"""),
    (Cell (Format-Delay $cfg.seconds) "action=auto-set width=84 align=center tooltip=""$(Tip $T.valueTip)"""),
    (Cell ([string][char]0xE76C) "action=auto-more tooltip=""$($T.more)"""))
  if ($left) { $autoCells += (Cell $left 'color=gray width=40') }
  # Delete all, and the purge at its right.
  $rows.Add(':: ' + ((@((Cell $T.deleteAll 'action=delete-all')) + $autoCells) -join ' :: '))
  $rows.Add('---')
  # What is above stays in place; the entries scroll under it.
  $rows.Add(' | pane=entries hidden=true')
  if ($State.Ready -and $State.Entries.Count -eq 0) {
    $rows.Add((Row $T.empty 'color=gray'))
  }
  # The pinned entries first, then the others, newest first; a line between the two groups.
  $pinned = @($State.Entries | Where-Object { $_.Pinned })
  $position = 0
  foreach ($entry in ($pinned + @(Get-Unpinned))) {
    if ($pinned.Count -gt 0 -and $position -eq $pinned.Count) { $rows.Add('---') }
    $position++
    $id = $entry.Id
    $hidden = Test-Hidden $entry
    $when = $entry.At.ToString('t')
    # A blurred entry is shown by a click with "Reveal on click" (and "Show entries"), else copied.
    $canReveal = $hidden -and $cfg.show -and $cfg.reveal
    $action = if ($canReveal) { "reveal:$id" } else { "copy:$id" }
    $hint = if ($canReveal) { $T.revealTipRow -f $RevealSeconds } else { $T.copyTip }
    $label = switch ($entry.Kind) {
      'image' { "$($T.image) $($entry.Size)" }
      'files' { "$($entry.Paths.Count) $(if ($entry.Paths.Count -eq 1) { $T.file } else { $T.files })" }
      default { "$($entry.Chars) $($T.chars)" }
    }
    # What the entry holds, in a cell that takes the width the buttons leave (fill=true): a picture
    # or a line of text that the fence ends with an ellipsis where it does not fit.
    $content = if ($entry.Kind -eq 'image') {
      $picture = if ($hidden) { $entry.Blur } else { $entry.Thumb }
      Cell '' "image=""$picture"" height=$([int][Math]::Ceiling($entry.ImageHeight)) fill=true action=$action tooltip=""$(Tip "$label`n$when`n$hint")"""
    } elseif ($hidden) {
      Cell '' "image=""$($entry.Blur)"" height=$([int]$entry.BlurHeight) fill=true action=$action tooltip=""$(Tip "$label`n$when`n$hint")"""
    } else {
      $source = if ($entry.Kind -eq 'files') { $entry.Paths -join '; ' } else { $entry.Text }
      $line = ($source -replace '\s+', ' ').Trim()
      if ($line.Length -gt 300) { $line = $line.Substring(0, 300) }
      $preview = $entry.Text
      if ($preview.Length -gt 300) { $preview = $preview.Substring(0, 300) + [char]0x2026 }
      Cell $line "fill=true action=$action tooltip=""$(Tip "$preview`n`n$when - $hint")"""
    }
    # Against the right edge, in a column: the trash, and the pin under it.
    $pinText = if ($entry.Pinned) { $T.unpin } else { $T.pin }
    $pin = Cell ([string][char]0xE718) "under=true action=pin:$id$(if ($entry.Pinned) { ' checked=true' }) tooltip=""$(Tip $pinText)"""
    $delete = Cell $trash "action=del:$id tooltip=""$($T.delTip)"""
    $rows.Add(':: ' + (($content, $delete, $pin) -join ' :: '))
  }
  $rows.ToArray()
}

# ---- Clicks -------------------------------------------------------------------------------
function Find-Entry([string]$id) {
  $State.Entries | Where-Object { [string]$_.Id -eq $id } | Select-Object -First 1
}

# 1 min 30 s, 45 s, 10 min: the purge delay as it is shown.
function Format-Delay([int]$total) {
  if ($total -lt 60) { return "$total $($T.seconds)" }
  $minutes = [int][Math]::Floor($total / 60)
  if ($total % 60 -eq 0) { "$minutes $($T.minutes)" } else { "$minutes $($T.minutes) $($total % 60) $($T.seconds)" }
}

# < and > go from one usual delay to the next.
$DelaySteps = 5, 10, 15, 20, 30, 45, 60, 90, 120, 180, 300, 600, 900, 1200, 1800, 2700, 3600, 5400, 7200, 10800, 14400, 28800, 43200, 86400
function Step-Delay([int]$direction) {
  $value = [int]$cfg.seconds
  $next = if ($direction -gt 0) { $DelaySteps | Where-Object { $_ -gt $value } | Select-Object -First 1 }
  else { $DelaySteps | Where-Object { $_ -lt $value } | Select-Object -Last 1 }
  if ($next) { Set-Delay $next }
}

function Set-Delay([int]$value) {
  $cfg.seconds = [Math]::Max(1, [Math]::Min(86400, $value))
  if ($cfg.auto -and (Get-Unpinned).Count) { $State.Deadline = (Get-Date).AddSeconds($cfg.seconds) }
  Save-Settings
}

# The window that asks for the delay, in minutes and seconds: a separate process of its own, as
# the script's thread has no message loop to keep a window alive.
function Start-Dialog {
  if ($State.Dialog) { return }
  $code = @'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$form = New-Object System.Windows.Forms.Form
$form.Text = $env:PF_TITLE
$form.TopMost = $true
$form.StartPosition = 'CenterScreen'
$form.FormBorderStyle = 'FixedDialog'
$form.MaximizeBox = $false
$form.MinimizeBox = $false
$form.ShowInTaskbar = $false
$form.ClientSize = New-Object System.Drawing.Size(380, 100)
$label = New-Object System.Windows.Forms.Label
$label.Text = $env:PF_PROMPT
$label.AutoSize = $true
$label.Location = New-Object System.Drawing.Point(12, 12)
$total = [int]$env:PF_VALUE
$box = New-Object System.Windows.Forms.NumericUpDown
$box.Minimum = 0
$box.Maximum = 1440
$box.Value = [Math]::Floor($total / 60)
$box.Location = New-Object System.Drawing.Point(12, 38)
$box.Size = New-Object System.Drawing.Size(70, 24)
$minutesLabel = New-Object System.Windows.Forms.Label
$minutesLabel.Text = $env:PF_MIN
$minutesLabel.AutoSize = $true
$minutesLabel.Location = New-Object System.Drawing.Point(86, 41)
$secondsBox = New-Object System.Windows.Forms.NumericUpDown
$secondsBox.Minimum = 0
$secondsBox.Maximum = 59
$secondsBox.Value = $total % 60
$secondsBox.Location = New-Object System.Drawing.Point(150, 38)
$secondsBox.Size = New-Object System.Drawing.Size(60, 24)
$secondsLabel = New-Object System.Windows.Forms.Label
$secondsLabel.Text = $env:PF_SEC
$secondsLabel.AutoSize = $true
$secondsLabel.Location = New-Object System.Drawing.Point(214, 41)
$ok = New-Object System.Windows.Forms.Button
$ok.Text = 'OK'
$ok.DialogResult = [System.Windows.Forms.DialogResult]::OK
$ok.Location = New-Object System.Drawing.Point(212, 66)
$cancel = New-Object System.Windows.Forms.Button
$cancel.Text = $env:PF_CANCEL
$cancel.DialogResult = [System.Windows.Forms.DialogResult]::Cancel
$cancel.Location = New-Object System.Drawing.Point(293, 66)
$form.AcceptButton = $ok
$form.CancelButton = $cancel
$form.Controls.AddRange(@($label, $box, $minutesLabel, $secondsBox, $secondsLabel, $ok, $cancel))
$form.Add_Shown({ $form.Activate(); $box.Focus(); $box.Select(0, 10) })
if ($form.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
  [Console]::Out.Write([int]($box.Value * 60 + $secondsBox.Value))
}
'@
  try {
    $psi = New-Object System.Diagnostics.ProcessStartInfo 'powershell.exe'
    $psi.Arguments = '-NoProfile -NonInteractive -STA -ExecutionPolicy Bypass -EncodedCommand ' +
      [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($code))
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $psi.RedirectStandardOutput = $true
    $psi.EnvironmentVariables['PF_TITLE'] = $T.dlgTitle
    $psi.EnvironmentVariables['PF_PROMPT'] = $T.dlgPrompt
    $psi.EnvironmentVariables['PF_CANCEL'] = $T.cancel
    $psi.EnvironmentVariables['PF_VALUE'] = [string]$cfg.seconds
    $psi.EnvironmentVariables['PF_MIN'] = $T.minutes
    $psi.EnvironmentVariables['PF_SEC'] = $T.seconds
    $process = [System.Diagnostics.Process]::Start($psi)
    $State.Dialog = @{ Process = $process; Out = $process.StandardOutput.ReadToEndAsync() }
  } catch {
    Write-Log "Dialog: $($_.Exception.Message)"
  }
}

function Complete-Dialog {
  $dialog = $State.Dialog
  if (-not $dialog -or -not $dialog.Out.IsCompleted) { return $false }
  $typed = ''
  try { $typed = $dialog.Out.Result.Trim([char]0xFEFF, "`r", "`n", ' ') } catch { }
  $dialog.Process.Dispose()
  $State.Dialog = $null
  if ($typed -match '^\d+$' -and [int]$typed -gt 0) { Set-Delay ([int]$typed); return $true }
  $false
}

function Invoke-ActionLine([string]$line) {
  $name, $argument = $line -split ':', 2
  switch ($name) {
    'toggle-show' { $cfg.show = -not $cfg.show; $State.Revealed.Clear(); Save-Settings }
    'toggle-reveal' { if ($cfg.show) { $cfg.reveal = -not $cfg.reveal; $State.Revealed.Clear(); Save-Settings } }
    'toggle-auto' {
      $cfg.auto = -not $cfg.auto
      $State.Deadline = if ($cfg.auto -and (Get-Unpinned).Count) { (Get-Date).AddSeconds($cfg.seconds) } else { $null }
      Save-Settings
    }
    'auto-less' { Step-Delay -1 }
    'auto-more' { Step-Delay 1 }
    'auto-set' { Start-Dialog }
    'delete-all' { Clear-Entries }
    'del' { $entry = Find-Entry $argument; if ($entry) { Remove-Entry $entry } }
    'copy' { $entry = Find-Entry $argument; if ($entry) { Copy-Entry $entry } }
    'pin' {
      $entry = Find-Entry $argument
      if ($entry) {
        $entry.Pinned = -not $entry.Pinned
        Save-Pinned
        if ($cfg.auto) {
          if ((Get-Unpinned).Count -eq 0) { $State.Deadline = $null }
          elseif (-not $State.Deadline) { $State.Deadline = (Get-Date).AddSeconds($cfg.seconds) }
        }
      }
    }
    'reveal' {
      $entry = Find-Entry $argument
      if ($entry -and $cfg.show -and $cfg.reveal) { $State.Revealed[$entry.Id] = [DateTime]::UtcNow.AddSeconds($RevealSeconds) }
    }
  }
}

# ---- Main loop ----------------------------------------------------------------------------
# What a click sends comes as lines on the standard input, read without waiting.
$stdin = [Console]::OpenStandardInput()
$inBuffer = New-Object byte[] 1024
$inRead = $stdin.ReadAsync($inBuffer, 0, $inBuffer.Length)
$inText = ''
if ([System.Threading.Thread]::CurrentThread.GetApartmentState() -ne 'STA') { Write-Log 'Not an STA thread: the clipboard may not be readable.' }

# The fence shows its boxes at once; what takes time to load (Windows Forms, the pictures of the
# pinned entries) comes after.
Show-Screen (Get-Screen)
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
$NativeType = New-NativeType
$BodyFont = New-Object System.Drawing.Font 'Segoe UI', 14, ([System.Drawing.FontStyle]::Regular), ([System.Drawing.GraphicsUnit]::Pixel)
$TextColor = if ($Dark) { [System.Drawing.Color]::FromArgb(240, 240, 240) } else { [System.Drawing.Color]::FromArgb(32, 32, 32) }
$State.LastSeq = Get-ClipboardSequence   # what was on the clipboard at the start is not kept
Restore-Pinned
$State.Ready = $true
Show-Screen (Get-Screen)
while ($true) {
  try {
    $dirty = $false
    if ($inRead.IsCompleted) {
      $count = 0
      try { $count = $inRead.Result } catch { }
      if ($count -le 0) { break }   # PecoFence closed our input: the fence is gone
      $inText += [Text.Encoding]::UTF8.GetString($inBuffer, 0, $count)
      $inRead = $stdin.ReadAsync($inBuffer, 0, $inBuffer.Length)
      while (($newline = $inText.IndexOf("`n")) -ge 0) {
        $line = $inText.Substring(0, $newline).Trim()
        $inText = $inText.Substring($newline + 1)
        $dirty = $true
        Invoke-ActionLine $line
      }
    }
    if (Update-Clipboard) { $dirty = $true }
    if (Complete-Dialog) { $dirty = $true }
    # A revealed entry is blurred again when its time is up.
    foreach ($id in @($State.Revealed.Keys)) {
      if ($State.Revealed[$id] -le [DateTime]::UtcNow) { $State.Revealed.Remove($id); $dirty = $true }
    }
    if ($State.Deadline -and (Get-Date) -ge $State.Deadline) { Clear-Entries; $dirty = $true }
    # The screen is built again when something changed, and each time while the purge counts down
    # (it shows the seconds left); it is printed only when it differs from the last one.
    if ($dirty -or $State.Deadline) { Show-Screen (Get-Screen) }
  } catch {
    Write-Log "$($_.Exception.Message) at line $($_.InvocationInfo.ScriptLineNumber)"
  }
  Start-Sleep -Milliseconds $PollMs
}
Clear-Entries
