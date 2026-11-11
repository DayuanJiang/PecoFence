# <pecofence.type>streamable</pecofence.type>
# <pecofence.title>Media Player</pecofence.title>
# PecoFence script fence: a media player (music, sounds and videos).
#
# The first comment keeps the script running (no 10 second limit); the second one is the title a
# fence made from this script gets (PecoFence otherwise names it after the file). The script
# plays the files itself (Windows' own media engine) and shows the player in the fence, one row per
# line or per button. A click on a row reaches the script as a line on its standard input (the
# row's action=). The buttons, the position slider and the volume slider use PecoFence's rows of
# cells (a line that starts with "::", cells side by side); a click or drag on a slider sends
# "seek:0.37" or "volume:0.37".
#
# Three tabs: Playback (the player), Playlist (the files of the folder or playlist, a page at a
# time; a click selects a file, a double-click plays it and goes back to Playback) and Radios (free
# internet radios from the open Radio Browser directory, see below).
#
# What does not fit the fence scrolls: the mouse wheel and a scrollbar on the right move the lyrics,
# the page of the Playlist and the page of the Radios (50 rows; the paging buttons go to the next
# page). The tabs and the buttons stay in place, and a bigger fence shows more rows. The lyrics
# follow the line being sung, unless you scrolled them yourself a few seconds ago. (A row
# "| pane=key hidden=true" splits the part that stays from the part that scrolls, and
# "| follow=key hidden=true" keeps the row after it in the middle: see docs/CLI.md.)
#
# Radios: nothing is asked of the internet until you click one of the tab's buttons (popular radios,
# by country, by genre, search by name). Radio Browser (https://api.radio-browser.info) is a free,
# open directory: no key, no payment, and no paid or licensed directory (SHOUTcast, TuneIn...) is
# used. Only the MP3 and AAC streams it found alive are listed. A click selects a radio, a
# double-click plays it and goes back to Playback, where Previous and Next move to the radio before
# and after in the list; the Favorites button lists the radios you saved (kept in the fence's own
# folder, so they play without the directory). $CountListens below can tell the directory a listen (off).
#
# Files: a folder (Browse...) or a playlist (Open playlist...: .m3u, .m3u8, .pls, .wpl).
# Audio: mp3, m4a, m4b, aac, wma, wav, flac, aif, aiff, au, mp2, mka and the audio of Ogg / Opus
# files. Video: mp4, m4v, mov, mkv, avi, wmv, asf, mpg, mpeg, m2ts, mts, 3gp, 3g2 and webm / ogv.
# Whatever Windows plays by itself is played; what needs a free codec of the Microsoft Store (Ogg
# Vorbis, Opus, WebM, Theora: "Web Media Extensions"; VP9, HEVC (H.265), AV1, MPEG-2 video: the
# extension of that name) is played once the extension is installed. Without it the file is left
# out (or skipped when it fails), and a line at the bottom of the player says which extension to
# install; a click on it opens its page in the Microsoft Store. The script looks again every 30
# seconds, so the files come in as soon as the extension is installed.
#
# Videos: PecoFence plays them itself, picture and sound, on the graphics card (the "video=" row,
# see docs/CLI.md): the picture is shown where the album cover is for a song, smooth, at the
# video's own rate. The script drives it (play, pause, volume, speed, moves) and is told how it goes
# (position, end, failure) on its standard input, in "video:" lines. A click on the picture plays
# or pauses. Subtitles: a .srt file with the video's name next to it is shown as the lyrics are.
#
# Tags (title, artist, album, year, genre, track) are read from mp3 (ID3v2), m4a/m4b (MP4 atoms),
# flac (Vorbis comments) and ogg / opus files. The cover of the album, from the song's tags or
# from a cover.jpg / folder.jpg (and the like) in its folder, is shown above the title. Under the title: the sampling rate, the bit depth
# (of lossless and uncompressed files) and the bit rate, as Windows' property system gives them;
# for a video, its size and frame rate too.
#
# Lyrics come from, in this order: a .lrc file with the song's name next to the song, a .srt or a
# .txt file with the song's name, the lyrics stored in the song's tags (mp3 USLT, m4a (c)lyr, flac
# LYRICS), and, when the "Online lyrics" box is ticked, lrclib.net (songs only, not videos). A
# line-synchronised .lrc ("[01:23.45] words") highlights the line being sung; with the enhanced
# format ("[01:23.45] <01:23.45>word <01:23.90>word") the words are followed one by one. Where a
# line has no word times, the words are spread over the line's duration; the word being sung is in
# bold. Lyrics without any time are spread over the whole song, which is only approximate.
#
# Next to that box, a spin box (a number with < and > buttons) shifts the lyrics in time
# for the song being played: a positive value shows them later, a negative one earlier, and the
# value is kept for that song.
#
# Online lyrics (off until you tick the box): the artist, title, album and duration of a song
# without lyrics are sent to lrclib.net (https://lrclib.net/docs), one request at a time, after
# the song has played for a few seconds. What it answers, and that it knows nothing, is kept in
# the fence's data folder, so a song is asked once. The script names itself in the User-Agent,
# keeps 500 ms between requests and waits as long as the server's Retry-After asks.
#
# Online subtitles (off until you tick the box, beside the first one): a video without subtitles
# is looked up on opensubtitles.com with your own API key ($OpenSubtitles below), by the
# fingerprint of the file (its "moviehash", so the subtitles fit this very version), else by its
# name. As the service asks (https://opensubtitles.tawk.help/article/getting-started): the key goes
# with every request, the User-Agent names the application and its version, the parameters of a
# search are in alphabetical order with "+" for spaces, and redirects are followed. Downloads are
# counted (5 a day without an account): a subtitle is downloaded once and kept in the fence's data
# folder, nothing is asked while the quota is used up, and the time it comes back is shown.
#
# Synchronize: (under the boxes) shifts the lyrics of a song or the subtitles of a video.
#
# Crossfade (the button after Shuffle): the last seconds of a file ($CrossfadeSeconds, 10) fade
# out while the next one of the play order fades in over them. A file that is too short gets a
# shorter crossfade; nothing fades when nothing comes next (the end of the list without Repeat,
# or Repeat one).
#
# The folder, playlist, volume, mute, shuffle, repeat and the online boxes are kept in this
# fence's own folder ($env:PECOFENCE_DATA\settings.json). Errors go to media-player.log there.

Add-Type -AssemblyName PresentationCore, WindowsBase

# ---- What you may want to change ----------------------------------------------------------
$Language = 'auto'                   # 'auto' (PecoFence's language), or 'en', 'fr', 'de', 'es', 'it',
                                     # 'pt-BR', 'ru', 'ja', 'ko', 'zh-CN', 'zh-TW'
# What Windows' own media engine plays: audio and video, without anything to install.
$AudioExtensions = '.mp3', '.m4a', '.m4b', '.aac', '.wma', '.wav', '.flac', '.aif', '.aiff', '.au', '.mp2', '.mka'
# (.ts, MPEG transport streams, is a video format too, but also the extension of TypeScript files:
# add it here if your folders hold recordings.)
$VideoExtensions = '.mp4', '.m4v', '.mov', '.mkv', '.avi', '.wmv', '.asf', '.mpg', '.mpeg', '.m2ts', '.mts', '.3gp', '.3g2'
# Played by Windows' media player of Store apps, once the Web Media Extensions are installed (WPF's
# player does not use them); the extension of each format. A webm may also need the VP9 one.
$StoreAudioExtensions = '.ogg', '.oga', '.opus'
$StoreVideoExtensions = '.webm', '.ogv'
# The free codecs of the Microsoft Store: the package Windows lists, the name the Store gives it and
# its page there. $StorePackages[...].Id is the "ProductId" of the Store's address.
$StorePackages = [ordered]@{
  webmedia = @{ Package = 'Microsoft.WebMediaExtensions'; Title = 'Web Media Extensions'; Id = '9N5TDP8VCMHS' }
  vp9 = @{ Package = 'Microsoft.VP9VideoExtensions'; Title = 'VP9 Video Extensions'; Id = '9N4D0MSMP0PT' }
  hevc = @{ Package = 'Microsoft.HEVCVideoExtension'; Title = 'HEVC Video Extensions'; Id = '9NMZLZ57R3T7' }
  av1 = @{ Package = 'Microsoft.AV1VideoExtension'; Title = 'AV1 Video Extension'; Id = '9MVZQVXJBQ9V' }
  mpeg2 = @{ Package = 'Microsoft.MPEG2VideoExtension'; Title = 'MPEG-2 Video Extension'; Id = '9N95Q1ZZPMH4' }
}
$MediaKeys = $true                   # the keyboard's media keys (play/pause, next, previous, stop) drive this player while it plays
$RememberSource = $true              # at the next start of PecoFence (Windows start-up too), open the folder or playlist chosen last
                                     # again; $false: start from the Music folder each time
$Recurse = $true                     # also the files in the sub-folders
$MaxSongs = 5000                     # files kept from a folder or a playlist
$LyricsBefore = 100                  # lyrics lines kept above the line being sung (the fence scrolls them: a bigger fence
$LyricsAfter = 100                   # shows more of them) and below it; a song has all its lines unless it has more than that
$CrossfadeSeconds = 10               # the Crossfade button: how long a file fades into the next
$CountListens = $false               # Radios: playing a radio tells Radio Browser one more person listens (it ranks the radios);
                                     # the directory then sees this computer's address
# opensubtitles.com (the Online subtitles box). ApiKey: your own key, free: make an account, then
# "API consumers" in your profile (https://www.opensubtitles.com/en/consumers). Languages: the
# languages of the subtitles, best first, for example 'fr,en'; 'auto' is PecoFence's language,
# then English.
$OpenSubtitles = @{ ApiKey = ''; Languages = 'auto' }

# What a folder or a playlist is searched for.
$ListedExtensions = $AudioExtensions + $VideoExtensions + $StoreAudioExtensions + $StoreVideoExtensions
# The extensions that are played as videos (their picture is shown).
$AllVideoExtensions = $VideoExtensions + $StoreVideoExtensions
# The extensions that need a package of $StorePackages, and which one.
$StoreExtensionPackage = @{}
foreach ($storeExtension in $StoreAudioExtensions + $StoreVideoExtensions) { $StoreExtensionPackage[$storeExtension] = 'webmedia' }

# ---- Constants ----------------------------------------------------------------------------
# Icons of Segoe Fluent Icons / Segoe MDL2 Assets, which PecoFence draws as a button's icon.
$Glyph = @{
  Prev = 0xE892; Play = 0xE768; Pause = 0xE769; Stop = 0xE71A; Next = 0xE893
  RepeatAll = 0xE8EE; RepeatOne = 0xE8ED; Shuffle = 0xE8B1; Crossfade = 0xE8AB; Volume = 0xE767; Mute = 0xE74F
  PageUp = 0xE70E; PageDown = 0xE70D
}
# The marks before the songs of the Playlist tab: playing, selected, the others.
$Mark = @{ Playing = 0x25B6; Selected = 0x25CF; Other = 0x25CB }
# Names of a picture of the album in the song's folder, best first.
$CoverNames = 'cover', 'folder', 'front', 'album', 'albumart', 'albumartsmall'
$CoverExtensions = '.jpg', '.jpeg', '.png', '.bmp', '.gif'

# Emoji used as the icon of a row.
$Emoji = @{
  Folder = 0x1F4C1; Browse = 0x1F50D; Playlist = 0x1F4C2; Music = 0x1F3B5
  Artist = 0x1F3A4; Album = 0x1F4BF; Genre = 0x1F3F7; Tracks = 0x1F3BC; Info = 0x2139; Audio = 0x1F39A; Video = 0x1F3AC
  Radio = 0x1F4FB; Popular = 0x1F525; Globe = 0x1F30D; Star = 0x2B50
}
# Fixed widths (DIPs) of the cells at the two ends of the sliders' rows, so both sliders have the
# same length, and the lyrics' layout.
$CellStyle = @{ Left = 'width=36 align=center'; Right = 'width=44 align=right'; Value = 'width=64 align=right' }
$Layout = @{
  WordGap = 4                        # DIPs between two words of the line being sung (a space)
  MaxCellsLineText = 250             # a longer line is shown as one row, without the bold word
  PlaylistRows = 50                  # songs in a page of the Playlist tab (the fence scrolls the page)
  RadioRows = 50                     # radios (or countries, genres) in a page of the Radios tab
  CoverHeight = 120                  # DIPs: the cover is at most this tall, and as wide as the fence
  VideoHeight = 220                  # DIPs: the picture of a video is at most this tall (and as wide as the fence)
  SpeedStep = 0.25                   # the playback speed changes by this much per click on < and >
  SpeedMin = 0.25
  SpeedMax = 4.0                     # (a decimal: [Math]::Min(4, 1.25) would pick the integer overload)
  SyncStep = 0.1                     # seconds per click on the lyrics' < and > buttons
  SyncMax = 60                       # the lyrics are shifted by 60 seconds at most
  CoversKept = 200                   # pictures kept in the cover cache folder
}
$Timing = @{
  PollMs = 30                        # how often a click is looked for
  RenderMs = 150                     # how often the time and the lyrics are redrawn
  VideoOpenSeconds = 15              # PecoFence has this long to say that it opened a video
  VideoHoldSeconds = 0.6             # after a move, a pause or a play asked of PecoFence's player, what it still reports from before is left out
  SettingsSaveDelayMs = 500          # a slider drag saves its settings this long after its last move
  PrevRestartSeconds = 3             # Previous restarts a song played longer than this
  OverrunSeconds = 2                 # a song still playing this long after its duration is over
  LogMaxBytes = 100KB
  LogRepeatSeconds = 10              # the same message is written once in this time
  CodecRecheckSeconds = 30           # while files wait for a Store extension: look again this often
  RtOpenSeconds = 6                  # the other player has this long to open a song
  RtEndedMarginSeconds = 0.5         # it is over when it stopped this close to its end
  RadioOpenSeconds = 15              # a radio has this long to start playing
  RadioStallSeconds = 3              # a radio that was playing and has been silent this long is opened again
  RadioReconnects = 3                # how many times in a row
}
# lrclib.net asks for these (https://lrclib.net/docs, "Implementation requirements").
$Lrclib = @{
  Url = 'https://lrclib.net/api/get'
  UserAgent = 'PecoFence-MediaPlayer/1.0 (https://github.com/DayuanJiang/PecoFence)'
  MinIntervalMs = 500                # between two requests (the docs say 200 to 500)
  StartAfterSeconds = 3              # a song plays this long before it is looked up
  MissRetryDays = 7                  # "not found" is asked again after this long
  ErrorBackoffSeconds = 300          # after a network error or a server error
  DefaultRetrySeconds = 60           # a 429 or 503 without Retry-After
  TimeoutMs = 10000
}
# opensubtitles.com's REST API (https://opensubtitles.tawk.help/article/getting-started: "Include
# your application name and version in the User-Agent header").
$OpenSubtitlesApi = @{
  Url = 'https://api.opensubtitles.com/api/v1'
  UserAgent = 'PecoFence-MediaPlayer v1.0'
  KeyPage = 'https://www.opensubtitles.com/en/consumers'   # where an API key is made
  MinIntervalMs = 1000               # between two requests
  StartAfterSeconds = 3              # a video plays this long before it is looked up
  MissRetryDays = 7                  # "not found" is asked again after this long
  ErrorBackoffSeconds = 300          # after a network error or a server error
  DefaultRetrySeconds = 60           # a 429 or 503 without Retry-After
  RefusedRetrySeconds = 3600         # the key was refused: asked again after this long
  QuotaRetrySeconds = 3600           # the download quota is used up and the answer says no time
  TimeoutMs = 15000
}
# Radio Browser (https://api.radio-browser.info), the free and open directory behind the Radios tab:
# no key, no payment. It asks for an application name and version in the User-Agent and for the
# list of its servers to be read from ServersUrl (a server may be down: another one is tried).
$RadioBrowserApi = @{
  ServersUrl = 'https://all.api.radio-browser.info/json/servers'
  FallbackServer = 'de1.api.radio-browser.info'
  UserAgent = 'PecoFence-MediaPlayer/1.0'
  TimeoutMs = 15000
  Limit = 100                        # radios asked at a time (the most listened first)
  MaxFavorites = 200
}
# The streams the player can play: MP3 and AAC (not the HLS playlists), found alive by the directory.
$RadioCodecs = 'MP3', 'AAC', 'AAC+'

# ---- Texts --------------------------------------------------------------------------------
$Strings = @{
  'en' = @{
    browse = 'Browse...'; open = 'Open this folder'; choosing = 'Choosing...';
    chooseTitle = 'Choose the folder with your media files'; playlist = 'Open playlist...';
    playlistTitle = 'Choose a playlist'; playlists = 'Playlists'; prev = 'Previous track';
    play = 'Play'; pause = 'Pause'; stop = 'Stop'; next = 'Next track'; repeatOff = 'Repeat: off';
    repeatAll = 'Repeat: all'; repeatOne = 'Repeat: one song'; shuffleOn = 'Shuffle: on';
    shuffleOff = 'Shuffle: off'; mute = 'Mute'; unmute = 'Unmute'; volume = 'Volume';
    seek = 'Position'; online = 'Online lyrics (lrclib.net)';
    onlineTip = 'When a song has no lyrics, look them up on lrclib.net (sends its artist, title, album and duration)';
    noSongs = 'No media files here. Use Browse... or Open playlist...';
    pressPlay = 'Press Play to start.';
    noLyrics = 'No lyrics (put a .lrc file with the song''s name next to it).';
    approx = 'Lyrics without times: only approximate';
    hintMissing = 'Files that need {1}: {0}. Windows cannot play them yet: click to install it from the Microsoft Store.';
    tabPlayback = 'Playback'; tabPlaylist = 'Playlist'; pageUp = 'Previous page';
    pageDown = 'Next page'; scroll = 'Scroll'; syncLabel = 'Synchronize:';
    syncEarlier = 'Show the lyrics or subtitles earlier (-0.1 s)'; syncLater = 'Show the lyrics or subtitles later (+0.1 s)';
    syncReset = 'Shift of the lyrics or subtitles in seconds: positive shows them later, negative earlier. Click to reset to 0.';
    onlineSubs = 'Online subtitles (opensubtitles.com)';
    onlineSubsTip = 'When a video has no subtitles, look them up on opensubtitles.com (sends a fingerprint of the file, its name and the languages wanted)';
    subsKey = 'opensubtitles.com needs your API key: put it in $OpenSubtitles.ApiKey at the top of the script (right-click > Edit script). Click here to create one.';
    subsRefused = 'opensubtitles.com refused the API key: check $OpenSubtitles.ApiKey at the top of the script.';
    subsQuota = 'Daily download quota of opensubtitles.com reached: subtitles are downloaded again from {0}.';
    noSubtitles = 'No subtitles (put a .srt file with the video''s name next to it).';
    crossfadeOn = 'Crossfade: on ({0} s)'; crossfadeOff = 'Crossfade: off'
    speedSlower = 'Slower'; speedFaster = 'Faster'; speedTip = 'Playback speed (click for normal speed)'
    tabRadios = 'Radios'; radioPopular = 'Popular radios'
    radioCountries = 'By country'; radioGenres = 'By genre'
    radioFavorites = 'Favorites'; radioSearch = 'Search by name...'
    radioSearchTitle = 'Search radios'; radioSearchPrompt = 'Name of the radio:'
    radioCountryLabel = 'Country: {0}'; radioGenreLabel = 'Genre: {0}'
    radioSearchLabel = 'Search: {0}'; radioFavAdd = 'Add to favorites'
    radioFavRemove = 'Remove from favorites'; radioNone = 'No radio found.'
    radioNoFavorites = 'No favorite radios yet. Select a radio, then click Add to favorites.'; radioStart = 'Pick a list with the buttons above: popular, by country, by genre or search. This asks radio-browser.info, a free directory of radios.'
    radioError = 'Could not reach the radio directory. Click here to try again.'; radioFailed = 'This radio cannot be played. Try another one.'
    radioLive = 'LIVE'; radioConnecting = 'Connecting...'
    cancel = 'Cancel'
  }
  'fr' = @{
    browse = 'Parcourir...'; open = 'Ouvrir ce dossier'; choosing = 'Choix en cours...';
    chooseTitle = 'Choisissez le dossier qui contient vos médias';
    playlist = 'Ouvrir une playlist...'; playlistTitle = 'Choisissez une playlist';
    playlists = 'Playlists'; prev = 'Piste précédente'; play = 'Lecture'; pause = 'Pause';
    stop = 'Stop'; next = 'Piste suivante'; repeatOff = 'Répéter : non';
    repeatAll = 'Répéter : tout'; repeatOne = 'Répéter : une piste';
    shuffleOn = 'Lecture aléatoire : oui'; shuffleOff = 'Lecture aléatoire : non';
    mute = 'Couper le son'; unmute = 'Rétablir le son'; volume = 'Volume'; seek = 'Position';
    online = 'Paroles en ligne (lrclib.net)';
    onlineTip = 'Si un morceau n''a pas de paroles, les chercher sur lrclib.net (envoie son artiste, son titre, son album et sa durée)';
    noSongs = 'Aucun média ici. Utilisez Parcourir... ou Ouvrir une playlist...';
    pressPlay = 'Appuyez sur Lecture pour commencer.';
    noLyrics = 'Pas de paroles (placez un fichier .lrc du même nom à côté du morceau).';
    approx = 'Paroles sans minutage : approximatif seulement';
    hintMissing = 'Fichiers qui nécessitent {1} : {0}. Windows ne peut pas encore les lire : cliquez pour l''installer depuis le Microsoft Store.';
    tabPlayback = 'Lecture'; tabPlaylist = 'Liste de lecture'; pageUp = 'Page précédente';
    pageDown = 'Page suivante'; scroll = 'Défilement'; syncLabel = 'Synchroniser :';
    syncEarlier = 'Afficher les paroles ou sous-titres plus tôt (-0,1 s)';
    syncLater = 'Afficher les paroles ou sous-titres plus tard (+0,1 s)';
    syncReset = 'Décalage des paroles ou sous-titres en secondes : positif, plus tard ; négatif, plus tôt. Cliquez pour remettre à 0.';
    onlineSubs = 'Sous-titres en ligne (opensubtitles.com)';
    onlineSubsTip = 'Si une vidéo n''a pas de sous-titres, les chercher sur opensubtitles.com (envoie une empreinte du fichier, son nom et les langues voulues)';
    subsKey = 'opensubtitles.com demande votre clé d''API : mettez-la dans $OpenSubtitles.ApiKey en haut du script (clic droit > « Modifier le script »). Cliquez ici pour en créer une.';
    subsRefused = 'opensubtitles.com a refusé la clé d''API : vérifiez $OpenSubtitles.ApiKey en haut du script.';
    subsQuota = 'Quota quotidien de téléchargements d''opensubtitles.com atteint : les sous-titres seront de nouveau téléchargés à partir de {0}.';
    noSubtitles = 'Pas de sous-titres (placez un fichier .srt du même nom à côté de la vidéo).';
    crossfadeOn = 'Fondu enchaîné : oui ({0} s)'; crossfadeOff = 'Fondu enchaîné : non'
    speedSlower = 'Plus lent'; speedFaster = 'Plus rapide'; speedTip = 'Vitesse de lecture (clic : vitesse normale)'
    tabRadios = 'Radios'; radioPopular = 'Radios populaires'
    radioCountries = 'Par pays'; radioGenres = 'Par genre'
    radioFavorites = 'Favorites'; radioSearch = 'Chercher par nom...'
    radioSearchTitle = 'Rechercher une radio'; radioSearchPrompt = 'Nom de la radio :'
    radioCountryLabel = 'Pays : {0}'; radioGenreLabel = 'Genre : {0}'
    radioSearchLabel = 'Recherche : {0}'; radioFavAdd = 'Ajouter aux favorites'
    radioFavRemove = 'Retirer des favorites'; radioNone = 'Aucune radio trouvée.'
    radioNoFavorites = 'Aucune radio favorite. Sélectionnez une radio, puis cliquez sur Ajouter aux favorites.'; radioStart = 'Choisissez une liste avec les boutons ci-dessus : populaires, par pays, par genre ou recherche. Cela interroge radio-browser.info, un annuaire gratuit de radios.'
    radioError = 'Impossible de joindre l''annuaire des radios. Cliquez ici pour réessayer.'; radioFailed = 'Cette radio ne peut pas être lue. Essayez-en une autre.'
    radioLive = 'DIRECT'; radioConnecting = 'Connexion...'
    cancel = 'Annuler'
  }
  'de' = @{
    browse = 'Durchsuchen...'; open = 'Diesen Ordner öffnen'; choosing = 'Auswahl läuft...';
    chooseTitle = 'Wählen Sie den Ordner mit Ihren Mediendateien'; playlist = 'Playlist öffnen...';
    playlistTitle = 'Wählen Sie eine Playlist'; playlists = 'Playlists'; prev = 'Vorheriger Titel';
    play = 'Wiedergabe'; pause = 'Pause'; stop = 'Stopp'; next = 'Nächster Titel';
    repeatOff = 'Wiederholen: aus'; repeatAll = 'Wiederholen: alle';
    repeatOne = 'Wiederholen: ein Titel'; shuffleOn = 'Zufällig: ein'; shuffleOff = 'Zufällig: aus';
    mute = 'Stummschalten'; unmute = 'Ton einschalten'; volume = 'Lautstärke'; seek = 'Position';
    online = 'Online-Liedtexte (lrclib.net)';
    onlineTip = 'Hat ein Titel keinen Liedtext, wird er auf lrclib.net gesucht (sendet Interpret, Titel, Album und Dauer)';
    noSongs = 'Keine Mediendateien hier. Verwenden Sie Durchsuchen... oder Playlist öffnen...';
    pressPlay = 'Drücken Sie Wiedergabe, um zu starten.';
    noLyrics = 'Kein Liedtext (legen Sie eine .lrc-Datei mit dem Namen des Titels daneben).';
    approx = 'Liedtext ohne Zeitangaben: nur ungefähr';
    hintMissing = 'Dateien, die {1} benötigen: {0}. Windows kann sie noch nicht abspielen: Klicken Sie, um es aus dem Microsoft Store zu installieren.';
    tabPlayback = 'Wiedergabe'; tabPlaylist = 'Wiedergabeliste'; pageUp = 'Vorherige Seite';
    pageDown = 'Nächste Seite'; scroll = 'Scrollen'; syncLabel = 'Synchronisieren:';
    syncEarlier = 'Liedtext oder Untertitel früher anzeigen (-0,1 s)';
    syncLater = 'Liedtext oder Untertitel später anzeigen (+0,1 s)';
    syncReset = 'Verschiebung von Liedtext oder Untertiteln in Sekunden: positiv zeigt sie später, negativ früher. Klicken setzt auf 0 zurück.';
    onlineSubs = 'Online-Untertitel (opensubtitles.com)';
    onlineSubsTip = 'Hat ein Video keine Untertitel, werden sie auf opensubtitles.com gesucht (sendet einen Fingerabdruck der Datei, ihren Namen und die gewünschten Sprachen)';
    subsKey = 'opensubtitles.com braucht Ihren API-Schlüssel: Tragen Sie ihn oben im Skript in $OpenSubtitles.ApiKey ein (Rechtsklick > „Skript bearbeiten“). Klicken Sie hier, um einen zu erstellen.';
    subsRefused = 'opensubtitles.com hat den API-Schlüssel abgelehnt: Prüfen Sie $OpenSubtitles.ApiKey oben im Skript.';
    subsQuota = 'Tägliches Download-Kontingent von opensubtitles.com erreicht: Untertitel werden ab {0} wieder heruntergeladen.';
    noSubtitles = 'Keine Untertitel (legen Sie eine .srt-Datei mit dem Namen des Videos daneben).';
    crossfadeOn = 'Überblenden: ein ({0} s)'; crossfadeOff = 'Überblenden: aus'
    speedSlower = 'Langsamer'; speedFaster = 'Schneller'; speedTip = 'Wiedergabegeschwindigkeit (Klick: normale Geschwindigkeit)'
    tabRadios = 'Radios'; radioPopular = 'Beliebte Sender'
    radioCountries = 'Nach Land'; radioGenres = 'Nach Genre'
    radioFavorites = 'Favoriten'; radioSearch = 'Nach Name suchen...'
    radioSearchTitle = 'Sender suchen'; radioSearchPrompt = 'Name des Senders:'
    radioCountryLabel = 'Land: {0}'; radioGenreLabel = 'Genre: {0}'
    radioSearchLabel = 'Suche: {0}'; radioFavAdd = 'Zu Favoriten hinzufügen'
    radioFavRemove = 'Aus Favoriten entfernen'; radioNone = 'Kein Sender gefunden.'
    radioNoFavorites = 'Noch keine Lieblingssender. Wählen Sie einen Sender und klicken Sie auf Zu Favoriten hinzufügen.'; radioStart = 'Wählen Sie oben eine Liste: beliebt, nach Land, nach Genre oder Suche. Dafür wird radio-browser.info abgefragt, ein kostenloses Senderverzeichnis.'
    radioError = 'Das Senderverzeichnis ist nicht erreichbar. Hier klicken, um es erneut zu versuchen.'; radioFailed = 'Dieser Sender lässt sich nicht abspielen. Versuchen Sie einen anderen.'
    radioLive = 'LIVE'; radioConnecting = 'Verbindung wird hergestellt...'
    cancel = 'Abbrechen'
  }
  'es' = @{
    browse = 'Examinar...'; open = 'Abrir esta carpeta'; choosing = 'Eligiendo...';
    chooseTitle = 'Elija la carpeta con sus archivos multimedia'; playlist = 'Abrir lista de reproducción...';
    playlistTitle = 'Elija una lista de reproducción'; playlists = 'Listas de reproducción';
    prev = 'Pista anterior'; play = 'Reproducir'; pause = 'Pausa'; stop = 'Detener';
    next = 'Pista siguiente'; repeatOff = 'Repetir: no'; repeatAll = 'Repetir: todo';
    repeatOne = 'Repetir: una pista'; shuffleOn = 'Aleatorio: sí'; shuffleOff = 'Aleatorio: no';
    mute = 'Silenciar'; unmute = 'Activar el sonido'; volume = 'Volumen'; seek = 'Posición';
    online = 'Letras en línea (lrclib.net)';
    onlineTip = 'Si una canción no tiene letra, la busca en lrclib.net (envía su artista, título, álbum y duración)';
    noSongs = 'No hay archivos multimedia aquí. Use Examinar... o Abrir lista de reproducción...';
    pressPlay = 'Pulse Reproducir para empezar.';
    noLyrics = 'Sin letra (ponga un archivo .lrc con el nombre de la canción a su lado).';
    approx = 'Letra sin tiempos: solo aproximada';
    hintMissing = 'Archivos que necesitan {1}: {0}. Windows aún no puede reproducirlos: haga clic para instalarlo desde Microsoft Store.';
    tabPlayback = 'Reproducción'; tabPlaylist = 'Lista'; pageUp = 'Página anterior';
    pageDown = 'Página siguiente'; scroll = 'Desplazamiento'; syncLabel = 'Sincronizar:';
    syncEarlier = 'Mostrar la letra o los subtítulos antes (-0,1 s)';
    syncLater = 'Mostrar la letra o los subtítulos después (+0,1 s)';
    syncReset = 'Desfase de la letra o los subtítulos en segundos: positivo los muestra después, negativo antes. Haga clic para volver a 0.';
    onlineSubs = 'Subtítulos en línea (opensubtitles.com)';
    onlineSubsTip = 'Si un vídeo no tiene subtítulos, los busca en opensubtitles.com (envía una huella del archivo, su nombre y los idiomas deseados)';
    subsKey = 'opensubtitles.com necesita su clave de API: póngala en $OpenSubtitles.ApiKey al principio del script (clic derecho > «Editar script»). Haga clic aquí para crear una.';
    subsRefused = 'opensubtitles.com rechazó la clave de API: revise $OpenSubtitles.ApiKey al principio del script.';
    subsQuota = 'Se alcanzó la cuota diaria de descargas de opensubtitles.com: los subtítulos se volverán a descargar a partir de las {0}.';
    noSubtitles = 'Sin subtítulos (ponga un archivo .srt con el nombre del vídeo a su lado).';
    crossfadeOn = 'Fundido encadenado: sí ({0} s)'; crossfadeOff = 'Fundido encadenado: no'
    speedSlower = 'Más lento'; speedFaster = 'Más rápido'; speedTip = 'Velocidad de reproducción (clic: velocidad normal)'
    tabRadios = 'Radios'; radioPopular = 'Radios populares'
    radioCountries = 'Por país'; radioGenres = 'Por género'
    radioFavorites = 'Favoritas'; radioSearch = 'Buscar por nombre...'
    radioSearchTitle = 'Buscar radios'; radioSearchPrompt = 'Nombre de la radio:'
    radioCountryLabel = 'País: {0}'; radioGenreLabel = 'Género: {0}'
    radioSearchLabel = 'Búsqueda: {0}'; radioFavAdd = 'Añadir a favoritas'
    radioFavRemove = 'Quitar de favoritas'; radioNone = 'No se encontró ninguna radio.'
    radioNoFavorites = 'Aún no hay radios favoritas. Seleccione una radio y pulse Añadir a favoritas.'; radioStart = 'Elija una lista con los botones de arriba: populares, por país, por género o búsqueda. Esto consulta radio-browser.info, un directorio gratuito de radios.'
    radioError = 'No se pudo conectar con el directorio de radios. Haga clic aquí para reintentar.'; radioFailed = 'Esta radio no se puede reproducir. Pruebe con otra.'
    radioLive = 'EN VIVO'; radioConnecting = 'Conectando...'
    cancel = 'Cancelar'
  }
  'it' = @{
    browse = 'Sfoglia...'; open = 'Apri questa cartella'; choosing = 'Scelta in corso...';
    chooseTitle = 'Scegli la cartella con i tuoi file multimediali'; playlist = 'Apri playlist...';
    playlistTitle = 'Scegli una playlist'; playlists = 'Playlist'; prev = 'Brano precedente';
    play = 'Riproduci'; pause = 'Pausa'; stop = 'Stop'; next = 'Brano successivo';
    repeatOff = 'Ripeti: no'; repeatAll = 'Ripeti: tutto'; repeatOne = 'Ripeti: un brano';
    shuffleOn = 'Casuale: sì'; shuffleOff = 'Casuale: no'; mute = 'Disattiva l''audio';
    unmute = 'Riattiva l''audio'; volume = 'Volume'; seek = 'Posizione';
    online = 'Testi online (lrclib.net)';
    onlineTip = 'Se un brano non ha il testo, lo cerca su lrclib.net (invia artista, titolo, album e durata)';
    noSongs = 'Nessun file multimediale qui. Usa Sfoglia... o Apri playlist...';
    pressPlay = 'Premi Riproduci per iniziare.';
    noLyrics = 'Nessun testo (metti accanto al brano un file .lrc con lo stesso nome).';
    approx = 'Testo senza tempi: solo approssimativo';
    hintMissing = 'File che richiedono {1}: {0}. Windows non può ancora riprodurli: fai clic per installarlo da Microsoft Store.';
    tabPlayback = 'Riproduzione'; tabPlaylist = 'Playlist'; pageUp = 'Pagina precedente';
    pageDown = 'Pagina successiva'; scroll = 'Scorri'; syncLabel = 'Sincronizza:';
    syncEarlier = 'Mostra il testo o i sottotitoli prima (-0,1 s)'; syncLater = 'Mostra il testo o i sottotitoli dopo (+0,1 s)';
    syncReset = 'Scostamento del testo o dei sottotitoli in secondi: positivo li mostra dopo, negativo prima. Fai clic per riportare a 0.';
    onlineSubs = 'Sottotitoli online (opensubtitles.com)';
    onlineSubsTip = 'Se un video non ha sottotitoli, li cerca su opensubtitles.com (invia un''impronta del file, il suo nome e le lingue richieste)';
    subsKey = 'opensubtitles.com richiede la tua chiave API: mettila in $OpenSubtitles.ApiKey all''inizio dello script (clic destro > «Modifica script»). Fai clic qui per crearne una.';
    subsRefused = 'opensubtitles.com ha rifiutato la chiave API: controlla $OpenSubtitles.ApiKey all''inizio dello script.';
    subsQuota = 'Raggiunta la quota giornaliera di download di opensubtitles.com: i sottotitoli verranno scaricati di nuovo dalle {0}.';
    noSubtitles = 'Nessun sottotitolo (metti accanto al video un file .srt con lo stesso nome).';
    crossfadeOn = 'Dissolvenza incrociata: sì ({0} s)'; crossfadeOff = 'Dissolvenza incrociata: no'
    speedSlower = 'Più lento'; speedFaster = 'Più veloce'; speedTip = 'Velocità di riproduzione (clic: velocità normale)'
    tabRadios = 'Radio'; radioPopular = 'Radio popolari'
    radioCountries = 'Per paese'; radioGenres = 'Per genere'
    radioFavorites = 'Preferite'; radioSearch = 'Cerca per nome...'
    radioSearchTitle = 'Cerca radio'; radioSearchPrompt = 'Nome della radio:'
    radioCountryLabel = 'Paese: {0}'; radioGenreLabel = 'Genere: {0}'
    radioSearchLabel = 'Ricerca: {0}'; radioFavAdd = 'Aggiungi alle preferite'
    radioFavRemove = 'Rimuovi dalle preferite'; radioNone = 'Nessuna radio trovata.'
    radioNoFavorites = 'Nessuna radio preferita. Seleziona una radio e fai clic su Aggiungi alle preferite.'; radioStart = 'Scegli un elenco con i pulsanti sopra: popolari, per paese, per genere o ricerca. Viene interrogato radio-browser.info, una directory gratuita di radio.'
    radioError = 'Impossibile raggiungere la directory delle radio. Fai clic qui per riprovare.'; radioFailed = 'Questa radio non può essere riprodotta. Provane un''altra.'
    radioLive = 'DIRETTA'; radioConnecting = 'Connessione...'
    cancel = 'Annulla'
  }
  'pt-BR' = @{
    browse = 'Procurar...'; open = 'Abrir esta pasta'; choosing = 'Escolhendo...';
    chooseTitle = 'Escolha a pasta com seus arquivos de mídia'; playlist = 'Abrir playlist...';
    playlistTitle = 'Escolha uma playlist'; playlists = 'Playlists'; prev = 'Faixa anterior';
    play = 'Reproduzir'; pause = 'Pausar'; stop = 'Parar'; next = 'Próxima faixa';
    repeatOff = 'Repetir: desligado'; repeatAll = 'Repetir: tudo'; repeatOne = 'Repetir: uma faixa';
    shuffleOn = 'Aleatório: ligado'; shuffleOff = 'Aleatório: desligado'; mute = 'Silenciar';
    unmute = 'Ativar o som'; volume = 'Volume'; seek = 'Posição';
    online = 'Letras online (lrclib.net)';
    onlineTip = 'Se uma música não tem letra, procura em lrclib.net (envia artista, título, álbum e duração)';
    noSongs = 'Nenhum arquivo de mídia aqui. Use Procurar... ou Abrir playlist...';
    pressPlay = 'Pressione Reproduzir para começar.';
    noLyrics = 'Sem letra (coloque um arquivo .lrc com o nome da música ao lado dela).';
    approx = 'Letra sem tempos: apenas aproximada';
    hintMissing = 'Arquivos que precisam de {1}: {0}. O Windows ainda não consegue reproduzi-los: clique para instalar pela Microsoft Store.';
    tabPlayback = 'Reprodução'; tabPlaylist = 'Playlist'; pageUp = 'Página anterior';
    pageDown = 'Próxima página'; scroll = 'Rolagem'; syncLabel = 'Sincronizar:';
    syncEarlier = 'Mostrar a letra ou as legendas antes (-0,1 s)'; syncLater = 'Mostrar a letra ou as legendas depois (+0,1 s)';
    syncReset = 'Deslocamento da letra ou das legendas em segundos: positivo mostra depois, negativo antes. Clique para voltar a 0.';
    onlineSubs = 'Legendas online (opensubtitles.com)';
    onlineSubsTip = 'Se um vídeo não tem legendas, procura em opensubtitles.com (envia uma impressão digital do arquivo, seu nome e os idiomas desejados)';
    subsKey = 'O opensubtitles.com precisa da sua chave de API: coloque-a em $OpenSubtitles.ApiKey no início do script (botão direito > “Editar script”). Clique aqui para criar uma.';
    subsRefused = 'O opensubtitles.com recusou a chave de API: verifique $OpenSubtitles.ApiKey no início do script.';
    subsQuota = 'Cota diária de downloads do opensubtitles.com atingida: as legendas voltam a ser baixadas a partir de {0}.';
    noSubtitles = 'Sem legendas (coloque um arquivo .srt com o nome do vídeo ao lado dele).';
    crossfadeOn = 'Crossfade: ligado ({0} s)'; crossfadeOff = 'Crossfade: desligado'
    speedSlower = 'Mais lento'; speedFaster = 'Mais rápido'; speedTip = 'Velocidade de reprodução (clique: velocidade normal)'
    tabRadios = 'Rádios'; radioPopular = 'Rádios populares'
    radioCountries = 'Por país'; radioGenres = 'Por gênero'
    radioFavorites = 'Favoritas'; radioSearch = 'Pesquisar por nome...'
    radioSearchTitle = 'Pesquisar rádios'; radioSearchPrompt = 'Nome da rádio:'
    radioCountryLabel = 'País: {0}'; radioGenreLabel = 'Gênero: {0}'
    radioSearchLabel = 'Pesquisa: {0}'; radioFavAdd = 'Adicionar às favoritas'
    radioFavRemove = 'Remover das favoritas'; radioNone = 'Nenhuma rádio encontrada.'
    radioNoFavorites = 'Nenhuma rádio favorita ainda. Selecione uma rádio e clique em Adicionar às favoritas.'; radioStart = 'Escolha uma lista com os botões acima: populares, por país, por gênero ou pesquisa. Isso consulta o radio-browser.info, um diretório gratuito de rádios.'
    radioError = 'Não foi possível acessar o diretório de rádios. Clique aqui para tentar de novo.'; radioFailed = 'Esta rádio não pode ser tocada. Tente outra.'
    radioLive = 'AO VIVO'; radioConnecting = 'Conectando...'
    cancel = 'Cancelar'
  }
  'ru' = @{
    browse = 'Обзор...'; open = 'Открыть эту папку'; choosing = 'Выбор...';
    chooseTitle = 'Выберите папку с вашими медиафайлами'; playlist = 'Открыть плейлист...';
    playlistTitle = 'Выберите плейлист'; playlists = 'Плейлисты'; prev = 'Предыдущий трек';
    play = 'Воспроизвести'; pause = 'Пауза'; stop = 'Стоп'; next = 'Следующий трек';
    repeatOff = 'Повтор: выкл.'; repeatAll = 'Повтор: все'; repeatOne = 'Повтор: один трек';
    shuffleOn = 'Перемешивание: вкл.'; shuffleOff = 'Перемешивание: выкл.'; mute = 'Выключить звук';
    unmute = 'Включить звук'; volume = 'Громкость'; seek = 'Позиция';
    online = 'Тексты песен онлайн (lrclib.net)';
    onlineTip = 'Если у песни нет текста, искать его на lrclib.net (отправляются исполнитель, название, альбом и длительность)';
    noSongs = 'Здесь нет медиафайлов. Нажмите «Обзор...» или «Открыть плейлист...»';
    pressPlay = 'Нажмите «Воспроизвести», чтобы начать.';
    noLyrics = 'Нет текста (положите рядом с песней файл .lrc с тем же именем).';
    approx = 'Текст без временных меток: только приблизительно';
    hintMissing = 'Файлов, которым нужно {1}: {0}. Windows пока не может их воспроизвести: нажмите, чтобы установить из Microsoft Store.';
    tabPlayback = 'Воспроизведение'; tabPlaylist = 'Плейлист'; pageUp = 'Предыдущая страница';
    pageDown = 'Следующая страница'; scroll = 'Прокрутка'; syncLabel = 'Синхронизация:';
    syncEarlier = 'Показывать текст или субтитры раньше (-0,1 с)'; syncLater = 'Показывать текст или субтитры позже (+0,1 с)';
    syncReset = 'Сдвиг текста или субтитров в секундах: положительный показывает позже, отрицательный раньше. Нажмите, чтобы сбросить на 0.';
    onlineSubs = 'Субтитры онлайн (opensubtitles.com)';
    onlineSubsTip = 'Если у видео нет субтитров, искать их на opensubtitles.com (отправляются отпечаток файла, его имя и нужные языки)';
    subsKey = 'opensubtitles.com нужен ваш ключ API: впишите его в $OpenSubtitles.ApiKey в начале скрипта (правый щелчок > «Изменить скрипт»). Нажмите здесь, чтобы создать ключ.';
    subsRefused = 'opensubtitles.com отклонил ключ API: проверьте $OpenSubtitles.ApiKey в начале скрипта.';
    subsQuota = 'Достигнут дневной лимит загрузок opensubtitles.com: субтитры снова будут загружаться с {0}.';
    noSubtitles = 'Нет субтитров (положите рядом с видео файл .srt с тем же именем).';
    crossfadeOn = 'Плавный переход: вкл. ({0} с)'; crossfadeOff = 'Плавный переход: выкл.'
    speedSlower = 'Медленнее'; speedFaster = 'Быстрее'; speedTip = 'Скорость воспроизведения (щелчок: обычная скорость)'
    tabRadios = 'Радио'; radioPopular = 'Популярные станции'
    radioCountries = 'По странам'; radioGenres = 'По жанрам'
    radioFavorites = 'Избранное'; radioSearch = 'Поиск по названию...'
    radioSearchTitle = 'Поиск радиостанций'; radioSearchPrompt = 'Название станции:'
    radioCountryLabel = 'Страна: {0}'; radioGenreLabel = 'Жанр: {0}'
    radioSearchLabel = 'Поиск: {0}'; radioFavAdd = 'Добавить в избранное'
    radioFavRemove = 'Убрать из избранного'; radioNone = 'Станции не найдены.'
    radioNoFavorites = 'В избранном пока ничего нет. Выберите станцию и нажмите «Добавить в избранное».'; radioStart = 'Выберите список кнопками выше: популярные, по странам, по жанрам или поиск. Запрашивается radio-browser.info, бесплатный каталог радиостанций.'
    radioError = 'Не удалось подключиться к каталогу станций. Нажмите здесь, чтобы повторить.'; radioFailed = 'Эту станцию воспроизвести не удалось. Попробуйте другую.'
    radioLive = 'ЭФИР'; radioConnecting = 'Подключение...'
    cancel = 'Отмена'
  }
  'ja' = @{
    browse = '参照...'; open = 'このフォルダーを開く'; choosing = '選択中...';
    chooseTitle = 'メディアファイルが入っているフォルダーを選択してください'; playlist = 'プレイリストを開く...';
    playlistTitle = 'プレイリストを選択してください'; playlists = 'プレイリスト'; prev = '前のトラック'; play = '再生';
    pause = '一時停止'; stop = '停止'; next = '次のトラック'; repeatOff = 'リピート: オフ'; repeatAll = 'リピート: すべて';
    repeatOne = 'リピート: 1曲'; shuffleOn = 'シャッフル: オン'; shuffleOff = 'シャッフル: オフ'; mute = 'ミュート';
    unmute = 'ミュート解除'; volume = '音量'; seek = '再生位置'; online = 'オンライン歌詞 (lrclib.net)';
    onlineTip = '曲に歌詞がない場合、lrclib.net で検索します (アーティスト、タイトル、アルバム、長さを送信)';
    noSongs = 'ここにはメディアファイルがありません。「参照...」か「プレイリストを開く...」を使ってください'; pressPlay = '「再生」を押して開始します。';
    noLyrics = '歌詞がありません (曲と同じ名前の .lrc ファイルを横に置いてください)。'; approx = '時間情報のない歌詞: おおよその表示のみ';
    hintMissing = '{1} が必要なファイル: {0} 個。Windows ではまだ再生できません。クリックして Microsoft Store からインストールしてください。';
    tabPlayback = '再生'; tabPlaylist = 'プレイリスト'; pageUp = '前のページ'; pageDown = '次のページ';
    scroll = 'スクロール'; syncLabel = '同期:'; syncEarlier = '歌詞や字幕を早く表示 (-0.1 秒)';
    syncLater = '歌詞や字幕を遅く表示 (+0.1 秒)'; syncReset = '歌詞や字幕のずれ (秒): 正の値で遅く、負の値で早く表示します。クリックで 0 に戻します。';
    onlineSubs = 'オンライン字幕 (opensubtitles.com)';
    onlineSubsTip = '動画に字幕がない場合、opensubtitles.com で検索します (ファイルの指紋、ファイル名、希望の言語を送信)';
    subsKey = 'opensubtitles.com には API キーが必要です。スクリプト先頭の $OpenSubtitles.ApiKey に入れてください (右クリック >「スクリプトを編集」)。クリックするとキーを作成できます。';
    subsRefused = 'opensubtitles.com が API キーを拒否しました。スクリプト先頭の $OpenSubtitles.ApiKey を確認してください。';
    subsQuota = 'opensubtitles.com の 1 日のダウンロード上限に達しました。{0} 以降にまたダウンロードします。';
    noSubtitles = '字幕がありません (動画と同じ名前の .srt ファイルを横に置いてください)。';
    crossfadeOn = 'クロスフェード: オン ({0} 秒)'; crossfadeOff = 'クロスフェード: オフ'
    speedSlower = '遅く'; speedFaster = '速く'; speedTip = '再生速度（クリックで通常速度）'
    tabRadios = 'ラジオ'; radioPopular = '人気のラジオ'
    radioCountries = '国別'; radioGenres = 'ジャンル別'
    radioFavorites = 'お気に入り'; radioSearch = '名前で検索...'
    radioSearchTitle = 'ラジオを検索'; radioSearchPrompt = 'ラジオの名前:'
    radioCountryLabel = '国: {0}'; radioGenreLabel = 'ジャンル: {0}'
    radioSearchLabel = '検索: {0}'; radioFavAdd = 'お気に入りに追加'
    radioFavRemove = 'お気に入りから削除'; radioNone = 'ラジオが見つかりません。'
    radioNoFavorites = 'お気に入りのラジオはまだありません。ラジオを選んで「お気に入りに追加」をクリックしてください。'; radioStart = '上のボタンで一覧を選んでください（人気、国別、ジャンル別、検索）。無料のラジオ ディレクトリ radio-browser.info に問い合わせます。'
    radioError = 'ラジオ ディレクトリに接続できません。ここをクリックして再試行してください。'; radioFailed = 'このラジオは再生できません。別のラジオをお試しください。'
    radioLive = 'LIVE'; radioConnecting = '接続中...'
    cancel = 'キャンセル'
  }
  'ko' = @{
    browse = '찾아보기...'; open = '이 폴더 열기'; choosing = '선택 중...'; chooseTitle = '미디어 파일이 있는 폴더를 선택하세요';
    playlist = '재생 목록 열기...'; playlistTitle = '재생 목록을 선택하세요'; playlists = '재생 목록'; prev = '이전 트랙';
    play = '재생'; pause = '일시 중지'; stop = '정지'; next = '다음 트랙'; repeatOff = '반복: 끔';
    repeatAll = '반복: 전체'; repeatOne = '반복: 한 곡'; shuffleOn = '셔플: 켬'; shuffleOff = '셔플: 끔';
    mute = '음소거'; unmute = '음소거 해제'; volume = '볼륨'; seek = '재생 위치'; online = '온라인 가사 (lrclib.net)';
    onlineTip = '곡에 가사가 없으면 lrclib.net에서 검색합니다 (아티스트, 제목, 앨범, 길이를 전송)';
    noSongs = '여기에는 미디어 파일이 없습니다. 찾아보기... 또는 재생 목록 열기...를 사용하세요'; pressPlay = '재생을 눌러 시작하세요.';
    noLyrics = '가사 없음 (곡과 같은 이름의 .lrc 파일을 옆에 두세요).'; approx = '시간 정보가 없는 가사: 대략적으로만 표시';
    hintMissing = '{1}이(가) 필요한 파일: {0}개. Windows에서 아직 재생할 수 없습니다. 클릭하여 Microsoft Store에서 설치하세요.';
    tabPlayback = '재생'; tabPlaylist = '재생 목록'; pageUp = '이전 페이지'; pageDown = '다음 페이지';
    scroll = '스크롤'; syncLabel = '동기화:'; syncEarlier = '가사나 자막을 더 일찍 표시 (-0.1초)';
    syncLater = '가사나 자막을 더 늦게 표시 (+0.1초)';
    syncReset = '가사나 자막의 오프셋(초): 양수는 더 늦게, 음수는 더 일찍 표시합니다. 클릭하면 0으로 되돌립니다.';
    onlineSubs = '온라인 자막 (opensubtitles.com)';
    onlineSubsTip = '동영상에 자막이 없으면 opensubtitles.com에서 검색합니다 (파일 지문, 파일 이름, 원하는 언어를 전송)';
    subsKey = 'opensubtitles.com에는 API 키가 필요합니다. 스크립트 맨 위의 $OpenSubtitles.ApiKey에 넣으세요 (마우스 오른쪽 버튼 > ''스크립트 편집''). 클릭하면 키를 만들 수 있습니다.';
    subsRefused = 'opensubtitles.com이 API 키를 거부했습니다. 스크립트 맨 위의 $OpenSubtitles.ApiKey를 확인하세요.';
    subsQuota = 'opensubtitles.com의 하루 다운로드 한도에 도달했습니다. {0}부터 다시 다운로드합니다.';
    noSubtitles = '자막 없음 (동영상과 같은 이름의 .srt 파일을 옆에 두세요).';
    crossfadeOn = '크로스페이드: 켬 ({0}초)'; crossfadeOff = '크로스페이드: 끔'
    speedSlower = '느리게'; speedFaster = '빠르게'; speedTip = '재생 속도 (클릭하면 보통 속도)'
    tabRadios = '라디오'; radioPopular = '인기 라디오'
    radioCountries = '국가별'; radioGenres = '장르별'
    radioFavorites = '즐겨찾기'; radioSearch = '이름으로 검색...'
    radioSearchTitle = '라디오 검색'; radioSearchPrompt = '라디오 이름:'
    radioCountryLabel = '국가: {0}'; radioGenreLabel = '장르: {0}'
    radioSearchLabel = '검색: {0}'; radioFavAdd = '즐겨찾기에 추가'
    radioFavRemove = '즐겨찾기에서 제거'; radioNone = '라디오를 찾지 못했습니다.'
    radioNoFavorites = '즐겨찾는 라디오가 아직 없습니다. 라디오를 선택하고 즐겨찾기에 추가를 클릭하세요.'; radioStart = '위 버튼으로 목록을 선택하세요(인기, 국가별, 장르별, 검색). 무료 라디오 디렉터리인 radio-browser.info에 요청합니다.'
    radioError = '라디오 디렉터리에 연결하지 못했습니다. 여기를 클릭해 다시 시도하세요.'; radioFailed = '이 라디오는 재생할 수 없습니다. 다른 라디오를 선택해 보세요.'
    radioLive = 'LIVE'; radioConnecting = '연결 중...'
    cancel = '취소'
  }
  'zh-CN' = @{
    browse = '浏览...'; open = '打开此文件夹'; choosing = '正在选择...'; chooseTitle = '请选择存放媒体文件的文件夹';
    playlist = '打开播放列表...'; playlistTitle = '请选择播放列表'; playlists = '播放列表'; prev = '上一曲';
    play = '播放'; pause = '暂停'; stop = '停止'; next = '下一曲'; repeatOff = '循环：关'; repeatAll = '循环：全部';
    repeatOne = '循环：单曲'; shuffleOn = '随机：开'; shuffleOff = '随机：关'; mute = '静音'; unmute = '取消静音';
    volume = '音量'; seek = '进度'; online = '在线歌词 (lrclib.net)';
    onlineTip = '歌曲没有歌词时到 lrclib.net 查找（会发送歌手、标题、专辑和时长）'; noSongs = '这里没有媒体文件。请使用“浏览...”或“打开播放列表...”';
    pressPlay = '按“播放”开始。'; noLyrics = '没有歌词（在歌曲旁放一个同名的 .lrc 文件）。'; approx = '歌词没有时间：只是大致位置';
    hintMissing = '需要 {1} 的文件：{0} 个，Windows 目前无法播放。点击从 Microsoft Store 安装。';
    tabPlayback = '播放'; tabPlaylist = '播放列表'; pageUp = '上一页'; pageDown = '下一页'; scroll = '滚动';
    syncLabel = '同步：'; syncEarlier = '提前显示歌词或字幕 (-0.1 秒)'; syncLater = '延后显示歌词或字幕 (+0.1 秒)';
    syncReset = '歌词或字幕偏移（秒）：正值延后，负值提前。点击恢复为 0。';
    onlineSubs = '在线字幕 (opensubtitles.com)';
    onlineSubsTip = '视频没有字幕时到 opensubtitles.com 查找（会发送文件指纹、文件名和所需语言）';
    subsKey = 'opensubtitles.com 需要你的 API 密钥：把它填到脚本开头的 $OpenSubtitles.ApiKey 里（右键 >「编辑脚本」）。点击这里创建一个。';
    subsRefused = 'opensubtitles.com 拒绝了 API 密钥：请检查脚本开头的 $OpenSubtitles.ApiKey。';
    subsQuota = '已达到 opensubtitles.com 每日下载上限：{0} 起会再次下载字幕。';
    noSubtitles = '没有字幕（在视频旁放一个同名的 .srt 文件）。';
    crossfadeOn = '淡入淡出：开（{0} 秒）'; crossfadeOff = '淡入淡出：关'
    speedSlower = '更慢'; speedFaster = '更快'; speedTip = '播放速度（点击恢复正常速度）'
    tabRadios = '电台'; radioPopular = '热门电台'
    radioCountries = '按国家'; radioGenres = '按风格'
    radioFavorites = '收藏'; radioSearch = '按名称搜索...'
    radioSearchTitle = '搜索电台'; radioSearchPrompt = '电台名称：'
    radioCountryLabel = '国家：{0}'; radioGenreLabel = '风格：{0}'
    radioSearchLabel = '搜索：{0}'; radioFavAdd = '加入收藏'
    radioFavRemove = '取消收藏'; radioNone = '没有找到电台。'
    radioNoFavorites = '还没有收藏的电台。选中一个电台，再点「加入收藏」。'; radioStart = '用上面的按钮选一个列表：热门、按国家、按风格或搜索。这会向 radio-browser.info（免费的电台目录）查询。'
    radioError = '无法连接到电台目录。点这里重试。'; radioFailed = '这个电台无法播放，请换一个试试。'
    radioLive = '直播'; radioConnecting = '正在连接...'
    cancel = '取消'
  }
  'zh-TW' = @{
    browse = '瀏覽...'; open = '開啟此資料夾'; choosing = '正在選擇...'; chooseTitle = '請選擇存放媒體檔案的資料夾';
    playlist = '開啟播放清單...'; playlistTitle = '請選擇播放清單'; playlists = '播放清單'; prev = '上一首';
    play = '播放'; pause = '暫停'; stop = '停止'; next = '下一首'; repeatOff = '重複：關'; repeatAll = '重複：全部';
    repeatOne = '重複：單曲'; shuffleOn = '隨機：開'; shuffleOff = '隨機：關'; mute = '靜音'; unmute = '取消靜音';
    volume = '音量'; seek = '進度'; online = '線上歌詞 (lrclib.net)';
    onlineTip = '歌曲沒有歌詞時到 lrclib.net 尋找（會傳送演出者、標題、專輯和長度）';
    noSongs = '這裡沒有媒體檔案。請使用「瀏覽...」或「開啟播放清單...」'; pressPlay = '按「播放」開始。';
    noLyrics = '沒有歌詞（在歌曲旁放一個同名的 .lrc 檔案）。'; approx = '歌詞沒有時間：只是大致位置';
    hintMissing = '需要 {1} 的檔案：{0} 個，Windows 目前無法播放。按一下即可從 Microsoft Store 安裝。';
    tabPlayback = '播放'; tabPlaylist = '播放清單'; pageUp = '上一頁'; pageDown = '下一頁'; scroll = '捲動';
    syncLabel = '同步：'; syncEarlier = '提前顯示歌詞或字幕 (-0.1 秒)'; syncLater = '延後顯示歌詞或字幕 (+0.1 秒)';
    syncReset = '歌詞或字幕偏移（秒）：正值延後，負值提前。按一下恢復為 0。';
    onlineSubs = '線上字幕 (opensubtitles.com)';
    onlineSubsTip = '影片沒有字幕時到 opensubtitles.com 尋找（會傳送檔案指紋、檔名和所需語言）';
    subsKey = 'opensubtitles.com 需要你的 API 金鑰：把它填到腳本開頭的 $OpenSubtitles.ApiKey 裡（按右鍵 >「編輯腳本」）。按一下這裡建立一個。';
    subsRefused = 'opensubtitles.com 拒絕了 API 金鑰：請檢查腳本開頭的 $OpenSubtitles.ApiKey。';
    subsQuota = '已達到 opensubtitles.com 每日下載上限：{0} 起會再次下載字幕。';
    noSubtitles = '沒有字幕（在影片旁放一個同名的 .srt 檔案）。';
    crossfadeOn = '淡入淡出：開（{0} 秒）'; crossfadeOff = '淡入淡出：關'
    speedSlower = '更慢'; speedFaster = '更快'; speedTip = '播放速度（點擊恢復正常速度）'
    tabRadios = '電台'; radioPopular = '熱門電台'
    radioCountries = '依國家'; radioGenres = '依風格'
    radioFavorites = '收藏'; radioSearch = '依名稱搜尋...'
    radioSearchTitle = '搜尋電台'; radioSearchPrompt = '電台名稱：'
    radioCountryLabel = '國家：{0}'; radioGenreLabel = '風格：{0}'
    radioSearchLabel = '搜尋：{0}'; radioFavAdd = '加入收藏'
    radioFavRemove = '取消收藏'; radioNone = '沒有找到電台。'
    radioNoFavorites = '還沒有收藏的電台。選取一個電台，再按「加入收藏」。'; radioStart = '用上面的按鈕選一個清單：熱門、依國家、依風格或搜尋。這會向 radio-browser.info（免費的網路電台目錄）查詢。'
    radioError = '無法連線到電台目錄。按這裡重試。'; radioFailed = '這個電台無法播放，請換一個試試。'
    radioLive = '直播'; radioConnecting = '正在連線...'
    cancel = '取消'
  }
}


# The language of the texts: PecoFence's display language (it tells the script in
# PECOFENCE_LANGUAGE), else Windows' display language, else English. $Language above may name
# one of the languages of the table to force it.
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
$LanguageCode = Resolve-Language $Language
$T = $Strings[$LanguageCode]
# The languages of the online subtitles, best first, as opensubtitles.com names them ('pt-br').
$SubtitleLanguages = @(@(if ($OpenSubtitles.Languages -and $OpenSubtitles.Languages -ne 'auto') {
      $OpenSubtitles.Languages -split '[,; ]+' | Where-Object { $_ }
    } else { $LanguageCode; 'en' }) | ForEach-Object { $_.ToLowerInvariant() } | Select-Object -Unique)

# ---- State --------------------------------------------------------------------------------
$State = @{
  Files = @()                        # the songs
  Order = @()                        # their play order (indexes of Files)
  Pos = -1                           # place in Order of the song being played
  Cur = -1                           # its index in Files
  Mode = 'stopped'                   # stopped, playing, paused
  Title = ''
  MediaInfo = ''                     # "mp3, 44.1 kHz, 16 bit, 1.41 Mbit/s" (a video: "mp4, 640x360, 25 fps, ...")
  Cover = ''                         # the picture file of the cover shown above the title
  Offset = 0.0                       # the shift of the lyrics of this song, in seconds (+: later)
  Tab = 'playback'                   # playback, playlist, radios
  ListTop = 0                        # the first song shown in the Playlist tab
  Selected = -1                      # the song a click selected there
  Tags = $null
  Lyrics = $null
  Duration = 0.0
  KnownDuration = 0.0                # the duration read from the file itself, when it could be
  StartedAt = [DateTime]::UtcNow
  Speed = 1.0                        # the playback speed (1: normal), for every file until it is changed
  Engine = 'wpf'                     # the media player of this song: wpf, winrt (Ogg, Opus...), or native (a video, played by PecoFence)
  Native = $null                     # the video PecoFence plays: Path, Playing, Volume, Speed, Seek / SeekId, and what it reported (State, Position, Duration, At)
  Retried = $false                   # the other engine was tried after the first failed
  RtSource = $null
  IsVideo = $false                   # the file is a video: PecoFence shows its picture
  Failures = 0                       # songs in a row that could not be played
  Fade = $null                       # a crossfade under way: @{ Tail (the player of the file fading out); Seconds }
  FadeStarting = $false              # the next file is being started by a crossfade
  MediaEnded = $false
  MediaFailed = $false
  Loading = $true                    # the songs are being listed
  Scan = $null                       # the background listing of a folder
  Candidates = $null                 # the files found, before the formats Windows cannot play are left out
  Needs = @{}                        # for each Store extension that is missing: the files that need it
  AllFound = @()                     # every file found, to list again once the extensions are installed
  Installed = $null                  # the Store extensions installed, by key of $StorePackages ($null: not known yet)
  Codec = $null                      # the background check for them
  CodecCheckedAt = [DateTime]::MinValue
  Dialog = $null                     # the folder or playlist picker being open
  SettingsDirtyAt = $null            # settings changed, to be saved
  LastScreen = $null
  LastLog = ''
  LastLogAt = [DateTime]::MinValue
  Subs = @{                          # the online subtitles (opensubtitles.com)
    Request = $null                  # the request in progress
    Next = $null                     # the next request of the same lookup, sent when the spacing allows
    Tried = ''                       # the video already looked up (or ruled out)
    BlockedUntil = [DateTime]::MinValue
    LastRequestAt = [DateTime]::MinValue
    QuotaUntil = [DateTime]::MinValue # the download quota is used up until then
    Refused = $false                 # the API key was refused
    HashPath = ''                    # the video whose fingerprint is Hash
    Hash = ''
  }
  Radio = @{                         # the Radios tab and the radio being played
    On = $false                      # the last thing started was a radio: Play starts it again, and Previous / Next go to the radio before / after
    Station = $null                  # the radio being played (or tried last): Uuid, Name, Url, CountryCode, Country, Codec, Bitrate
    Failed = $false                  # it could not be played
    Heard = $false                   # it has played since it was opened
    Reconnects = 0                   # times in a row that a stream that was playing was opened again
    StalledAt = $null                # since when a stream that was playing is silent
    View = 'stations'                # what the tab lists: stations, countries, genres
    Source = ''                      # top, country, genre, search, favorites, countries, genres ('': nothing asked yet)
    Label = ''                       # the line under the buttons: what the list is
    Stations = @()                   # the radios of the list
    Queue = @()                      # the list the radio being played was chosen from: Previous and Next go along it
    Items = @()                      # the countries or the genres of the list (Name, Value, Count)
    CountryItems = $null             # the lists, kept once they are read
    GenreItems = $null
    ListTop = 0                      # the first row shown
    Selected = -1                    # the radio a click selected
    Request = $null                  # the request in progress: Background, Kind, Label, Source, Path
    Last = $null                     # the last request, to try again after an error
    Error = $false                   # the last request failed
    Server = ''                      # the Radio Browser server that answered last
    CountJob = $null                 # the request that counts a listen
  }
  Online = @{
    Request = $null                  # the lookup in progress
    Tried = ''                       # the song already looked up (or ruled out)
    BlockedUntil = [DateTime]::MinValue
    LastRequestAt = [DateTime]::MinValue
  }
}

# ---- Settings and log ---------------------------------------------------------------------
$DataDir = if ($env:PECOFENCE_DATA) { $env:PECOFENCE_DATA } else { Join-Path $env:TEMP 'pecofence-media' }
$SettingsFile = Join-Path $DataDir 'settings.json'
$cfg = @{
  folder = [Environment]::GetFolderPath('MyMusic')
  playlist = ''
  volume = 60
  muted = $false
  shuffle = $false
  crossfade = $false
  repeat = 'off'
  online = $false
  subtitles = $false
  offsets = @{}                      # the shift of the lyrics of the songs that have one (seconds)
  radioFavorites = @()               # the radios saved in the Radios tab (Uuid, Name, Url, CountryCode, Country, Codec, Bitrate)
}
try {
  $saved = Get-Content -LiteralPath $SettingsFile -Raw -Encoding UTF8 | ConvertFrom-Json
  # The folder or playlist chosen last (kept in the fence's own data folder, so it survives a restart).
  if ($RememberSource -and $saved.folder) { $cfg.folder = [string]$saved.folder }
  if ($RememberSource -and $saved.playlist) { $cfg.playlist = [string]$saved.playlist }
  if ($null -ne $saved.volume) { $cfg.volume = [Math]::Max(0, [Math]::Min(100, [int]$saved.volume)) }
  $cfg.muted = [bool]$saved.muted
  $cfg.shuffle = [bool]$saved.shuffle
  $cfg.crossfade = [bool]$saved.crossfade
  $cfg.online = [bool]$saved.online
  $cfg.subtitles = [bool]$saved.subtitles
  if (@('off', 'all', 'one') -contains $saved.repeat) { $cfg.repeat = [string]$saved.repeat }
  if ($saved.offsets) {
    foreach ($entry in $saved.offsets.PSObject.Properties) { $cfg.offsets[[string]$entry.Name] = [double]$entry.Value }
  }
  foreach ($favorite in @($saved.radioFavorites)) {
    if ($cfg.radioFavorites.Count -ge $RadioBrowserApi.MaxFavorites) { break }
    if ($favorite -and [string]$favorite.Url -match '^https?://' -and [string]$favorite.Name) {
      $cfg.radioFavorites += @{
        Uuid = [string]$favorite.Uuid; Name = [string]$favorite.Name; Url = [string]$favorite.Url
        CountryCode = [string]$favorite.CountryCode; Country = [string]$favorite.Country
        Codec = [string]$favorite.Codec; Bitrate = [int]$favorite.Bitrate
      }
    }
  }
} catch { }

# Errors never show in the fence; they go to media-player.log in the fence's data folder.
function Write-Log([string]$message) {
  try {
    $now = [DateTime]::UtcNow
    if ($message -ceq $State.LastLog -and ($now - $State.LastLogAt).TotalSeconds -lt $Timing.LogRepeatSeconds) { return }
    $State.LastLog = $message
    $State.LastLogAt = $now
    $file = Join-Path $DataDir 'media-player.log'
    New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
    if ((Test-Path -LiteralPath $file) -and (Get-Item -LiteralPath $file).Length -gt $Timing.LogMaxBytes) {
      Remove-Item -LiteralPath $file -Force
    }
    Add-Content -LiteralPath $file -Value ('{0:yyyy-MM-dd HH:mm:ss}  {1}' -f (Get-Date), $message) -Encoding UTF8
  } catch { }
}

# Without -Now the settings are saved a moment later (a slider drag changes them many times a
# second); with it, at once.
function Save-Settings([switch]$Now) {
  if (-not $Now) {
    if ($null -eq $State.SettingsDirtyAt) { $State.SettingsDirtyAt = [DateTime]::UtcNow }
    return
  }
  $State.SettingsDirtyAt = $null
  try {
    New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
    $cfg | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $SettingsFile -Encoding UTF8
  } catch {
    Write-Log "Settings not saved: $($_.Exception.Message)"
  }
}

# ---- Output -------------------------------------------------------------------------------
function Icon([int]$code) { [string][char]$code }
function EmojiText([int]$code) { [char]::ConvertFromUtf32($code) }

# A row is `text | key=value ...`; "|" and line breaks in a song's text are replaced, or the
# rest of a title would be taken for parameters. A long text is cut (a line is read up to 300
# characters, and its parameters are at the end).
function Row([string]$text, [string]$params = '') {
  $clean = ($text -replace '[\r\n]+', ' ' -replace '\|', [string][char]0x00A6).Trim()
  if ($clean.Length -gt 200) { $clean = $clean.Substring(0, 199) + [string][char]0x2026 }
  if ($params) { "$clean | $params" } else { $clean }
}

# A cell of a row of cells: `text | parameters`.
function Cell([string]$text, [string]$params = '') {
  $clean = ($text -replace '[\r\n]+', ' ' -replace '\|', [string][char]0x00A6 -replace '::', ': :').Trim()
  if ($params) { "$clean | $params" } else { $clean }
}

# A whole screen in one write, after a ~~~ line; nothing is written when it did not change.
function Show-Screen([string[]]$rows) {
  $screen = $rows -join "`n"
  if ($screen -ceq $State.LastScreen) { return }
  $State.LastScreen = $screen
  Write-Output "~~~`n$screen"
}

function Format-Time([double]$seconds) {
  $s = [int][Math]::Max([double]0, [Math]::Floor($seconds))
  '{0}:{1:00}' -f [int][Math]::Floor($s / 60), ($s % 60)
}

function Format-Fraction([double]$value) {
  [string]::Format([Globalization.CultureInfo]::InvariantCulture, '{0:0.####}', $value)
}

# ---- Tags ---------------------------------------------------------------------------------
function New-Tags { @{ Title = ''; Artist = ''; Album = ''; Year = ''; Genre = ''; Track = ''; Lyrics = ''; Lossless = $false; Picture = $null } }

# The tags as they are shown: a four digit year, a genre in words, a track number.
function Complete-Tags($tags) {
  foreach ($name in 'Title', 'Artist', 'Album') { $tags[$name] = ([string]$tags[$name]).Trim() }
  $year = [regex]::Match([string]$tags.Year, '\d{4}')
  $tags.Year = if ($year.Success) { $year.Value } else { '' }
  $genre = ([string]$tags.Genre).Trim()
  if ($genre -match '^\((\d+)\)(.*)$') { $genre = $Matches[2] }
  $tags.Genre = if ($genre -match '\p{L}') { $genre.Trim() } else { '' }
  $track = [regex]::Match([string]$tags.Track, '^\s*(\d+)')
  $tags.Track = if ($track.Success -and [int]$track.Groups[1].Value -gt 0) { [string][int]$track.Groups[1].Value } else { '' }
  $tags
}

function ConvertTo-UInt32BE([byte[]]$b, [int]$i) {
  ([int64]$b[$i] -shl 24) -bor ([int64]$b[$i + 1] -shl 16) -bor ([int64]$b[$i + 2] -shl 8) -bor [int64]$b[$i + 3]
}

function ConvertTo-UInt32LE([byte[]]$b, [int]$i) {
  ([int64]$b[$i + 3] -shl 24) -bor ([int64]$b[$i + 2] -shl 16) -bor ([int64]$b[$i + 1] -shl 8) -bor [int64]$b[$i]
}

# Reads `count` bytes of the stream into a new array (fewer at the end of the file).
function Read-Bytes($stream, [int]$count) {
  $buffer = New-Object byte[] $count
  $got = 0
  while ($got -lt $count) {
    $n = $stream.Read($buffer, $got, $count - $got)
    if ($n -le 0) { break }
    $got += $n
  }
  if ($got -lt $count) { $buffer = if ($got -gt 0) { [byte[]]$buffer[0..($got - 1)] } else { [byte[]]@() } }
  , $buffer
}

function Remove-Unsync([byte[]]$bytes) {
  $out = New-Object 'System.Collections.Generic.List[byte]'
  for ($i = 0; $i -lt $bytes.Length; $i++) {
    $out.Add($bytes[$i])
    if ($bytes[$i] -eq 0xFF -and $i + 1 -lt $bytes.Length -and $bytes[$i + 1] -eq 0) { $i++ }
  }
  , $out.ToArray()
}

# A copy of `count` bytes of `source` from `from`.
function Copy-Bytes([byte[]]$source, [int]$from, [int]$count) {
  if ($count -le 0) { return , ([byte[]]@()) }
  $copy = New-Object byte[] $count
  [Array]::Copy($source, $from, $copy, 0, $count)
  , $copy
}

# A FLAC "picture" block (Ogg stores the same, base64 encoded): the picture is kept when it is the
# front cover, or when it is the first one.
function Set-FlacPicture($tags, [byte[]]$b) {
  if ($b.Length -lt 32) { return }
  $type = ConvertTo-UInt32BE $b 0
  $i = 4
  $i += 4 + (ConvertTo-UInt32BE $b $i)                                 # the MIME type
  if ($i + 4 -gt $b.Length) { return }
  $i += 4 + (ConvertTo-UInt32BE $b $i) + 16                            # the description, width, height, depth, colours
  if ($i + 4 -gt $b.Length) { return }
  $length = ConvertTo-UInt32BE $b $i
  $i += 4
  if ($length -le 0 -or $i + $length -gt $b.Length) { return }
  if ($tags.Picture -and $type -ne 3) { return }
  $tags.Picture = Copy-Bytes $b ([int]$i) ([int]$length)
}

# Text of an ID3 frame from byte `from` on, in the frame's encoding byte `enc`.
function ConvertFrom-Id3Text([byte[]]$bytes, [int]$from, [int]$enc) {
  $len = $bytes.Length - $from
  if ($len -le 0) { return '' }
  if ($enc -eq 0) {
    $e = [Text.Encoding]::GetEncoding(28591)
  } elseif ($enc -eq 2) {
    $e = [Text.Encoding]::BigEndianUnicode
  } elseif ($enc -eq 1) {
    $e = [Text.Encoding]::Unicode
    if ($len -ge 2 -and $bytes[$from] -eq 0xFE -and $bytes[$from + 1] -eq 0xFF) {
      $e = [Text.Encoding]::BigEndianUnicode; $from += 2; $len -= 2
    } elseif ($len -ge 2 -and $bytes[$from] -eq 0xFF -and $bytes[$from + 1] -eq 0xFE) {
      $from += 2; $len -= 2
    }
  } else {
    $e = New-Object Text.UTF8Encoding($false)
  }
  if ($len -le 0) { return '' }
  $e.GetString($bytes, $from, $len).Trim([char]0)
}

# The tags of an mp3 (ID3v2.3 and v2.4); a field that is not there is an empty string.
function Read-Id3([string]$path) {
  $tags = New-Tags
  $fields = @{ TIT2 = 'Title'; TPE1 = 'Artist'; TALB = 'Album'; TYER = 'Year'; TDRC = 'Year'; TCON = 'Genre'; TRCK = 'Track' }
  $fs = [IO.File]::Open($path, 'Open', 'Read', 'ReadWrite')
  try {
    $h = Read-Bytes $fs 10
    if ($h.Length -lt 10 -or $h[0] -ne 0x49 -or $h[1] -ne 0x44 -or $h[2] -ne 0x33) { return $tags }
    $ver = [int]$h[3]
    if ($ver -ne 3 -and $ver -ne 4) { return $tags }
    $size = ([int]$h[6] -shl 21) -bor ([int]$h[7] -shl 14) -bor ([int]$h[8] -shl 7) -bor [int]$h[9]
    $tag = Read-Bytes $fs ([Math]::Min($size, 16MB))
  } finally { $fs.Dispose() }
  if ($ver -eq 3 -and ($h[5] -band 0x80)) { $tag = Remove-Unsync $tag }
  $end = $tag.Length
  $i = 0
  if ($h[5] -band 0x40) {
    # Extended header: its size includes itself in v2.4, and does not in v2.3.
    if ($ver -eq 4) {
      $i = ([int]$tag[0] -shl 21) -bor ([int]$tag[1] -shl 14) -bor ([int]$tag[2] -shl 7) -bor [int]$tag[3]
    } else {
      $i = 4 + (([int]$tag[0] -shl 24) -bor ([int]$tag[1] -shl 16) -bor ([int]$tag[2] -shl 8) -bor [int]$tag[3])
    }
  }
  while ($i + 10 -le $end -and $tag[$i] -ne 0) {
    $id = [Text.Encoding]::ASCII.GetString($tag, $i, 4)
    if ($ver -eq 4) {
      $fsz = ([int]$tag[$i + 4] -shl 21) -bor ([int]$tag[$i + 5] -shl 14) -bor ([int]$tag[$i + 6] -shl 7) -bor [int]$tag[$i + 7]
    } else {
      $fsz = ([int]$tag[$i + 4] -shl 24) -bor ([int]$tag[$i + 5] -shl 16) -bor ([int]$tag[$i + 6] -shl 8) -bor [int]$tag[$i + 7]
    }
    $f2 = [int]$tag[$i + 9]
    $start = $i + 10
    if ($fsz -le 0 -or $start + $fsz -gt $end) { break }
    $i = $start + $fsz
    if (-not $fields.ContainsKey($id) -and $id -ne 'USLT' -and $id -ne 'APIC') { continue }
    # Compressed or encrypted frames are skipped.
    if ($ver -eq 3 -and ($f2 -band 0xC0)) { continue }
    if ($ver -eq 4 -and ($f2 -band 0x0C)) { continue }
    $data = New-Object byte[] $fsz
    [Array]::Copy($tag, $start, $data, 0, $fsz)
    if ($ver -eq 4 -and ($f2 -band 0x01)) { if ($data.Length -le 4) { continue }; $data = $data[4..($data.Length - 1)] }
    if ($ver -eq 4 -and ($f2 -band 0x02)) { $data = Remove-Unsync $data }
    if ($data.Length -lt 2) { continue }
    $enc = [int]$data[0]
    if ($id -eq 'USLT') {
      # encoding, language (3 bytes), a description ended by a null, then the text.
      $from = 4
      if ($enc -eq 1 -or $enc -eq 2) {
        while ($from + 1 -lt $data.Length -and -not ($data[$from] -eq 0 -and $data[$from + 1] -eq 0)) { $from += 2 }
        $from += 2
      } else {
        while ($from -lt $data.Length -and $data[$from] -ne 0) { $from++ }
        $from += 1
      }
      if ($tags.Lyrics -eq '') { $tags.Lyrics = ConvertFrom-Id3Text $data $from $enc }
    } elseif ($id -eq 'APIC') {
      # encoding, MIME type (ended by a null), picture type (3: front cover), description, picture.
      $at = 1
      while ($at -lt $data.Length -and $data[$at] -ne 0) { $at++ }
      if ($at + 2 -lt $data.Length) {
        $type = [int]$data[$at + 1]
        $from = $at + 2
        if ($enc -eq 1 -or $enc -eq 2) {
          while ($from + 1 -lt $data.Length -and -not ($data[$from] -eq 0 -and $data[$from + 1] -eq 0)) { $from += 2 }
          $from += 2
        } else {
          while ($from -lt $data.Length -and $data[$from] -ne 0) { $from++ }
          $from += 1
        }
        if ($from -lt $data.Length -and (-not $tags.Picture -or $type -eq 3)) {
          $tags.Picture = Copy-Bytes $data $from ($data.Length - $from)
        }
      }
    } elseif ($tags[$fields[$id]] -eq '') {
      $tags[$fields[$id]] = ConvertFrom-Id3Text $data 1 $enc
    }
  }
  $tags
}

# The child box `type` of the box whose content is $b[$start .. $end): its content's range, or $null.
function Find-Mp4Box([byte[]]$b, [int]$start, [int]$end, [string]$type) {
  $latin1 = [Text.Encoding]::GetEncoding(28591)
  $i = $start
  while ($i + 8 -le $end) {
    $size = ConvertTo-UInt32BE $b $i
    if ($size -lt 8) { return $null }
    if ($latin1.GetString($b, $i + 4, 4) -ceq $type) {
      return @{ Start = $i + 8; End = [int][Math]::Min($i + $size, $end) }
    }
    $i += $size
  }
  $null
}

# The tags of an m4a / m4b (the "ilst" atom of "moov/udta/meta"), lyrics comprised.
function Read-Mp4([string]$path) {
  $tags = New-Tags
  $moov = $null
  $fs = [IO.File]::Open($path, 'Open', 'Read', 'ReadWrite')
  try {
    $pos = [int64]0
    while ($pos + 8 -le $fs.Length) {
      $fs.Position = $pos
      $h = Read-Bytes $fs 16
      if ($h.Length -lt 8) { break }
      $size = ConvertTo-UInt32BE $h 0
      $header = 8
      if ($size -eq 1 -and $h.Length -ge 16) {
        $size = (ConvertTo-UInt32BE $h 8) * 4294967296 + (ConvertTo-UInt32BE $h 12)
        $header = 16
      } elseif ($size -eq 0) {
        $size = $fs.Length - $pos
      }
      if ($size -lt $header) { break }
      if ([Text.Encoding]::ASCII.GetString($h, 4, 4) -ceq 'moov') {
        if ($size - $header -gt 32MB) { break }
        $fs.Position = $pos + $header
        $moov = Read-Bytes $fs ([int]($size - $header))
        break
      }
      $pos += $size
    }
  } finally { $fs.Dispose() }
  if ($null -eq $moov) { return $tags }
  # An Apple Lossless file has an "alac" sample entry; AAC has none.
  $tags.Lossless = [Text.Encoding]::GetEncoding(28591).GetString($moov).Contains('alac')
  $udta = Find-Mp4Box $moov 0 $moov.Length 'udta'
  if (-not $udta) { return $tags }
  $meta = Find-Mp4Box $moov $udta.Start $udta.End 'meta'
  if (-not $meta) { return $tags }
  # "meta" is a full box: four bytes of version and flags come before its children.
  $ilst = Find-Mp4Box $moov ($meta.Start + 4) $meta.End 'ilst'
  if (-not $ilst) { return $tags }
  $c = [string][char]0xA9
  $names = @{ ($c + 'nam') = 'Title'; ($c + 'ART') = 'Artist'; ($c + 'alb') = 'Album'
    ($c + 'day') = 'Year'; ($c + 'gen') = 'Genre'; ($c + 'lyr') = 'Lyrics'; 'trkn' = 'Track'; 'covr' = 'Picture' }
  $latin1 = [Text.Encoding]::GetEncoding(28591)
  $i = $ilst.Start
  while ($i + 8 -le $ilst.End) {
    $size = ConvertTo-UInt32BE $moov $i
    if ($size -lt 8) { break }
    $name = $latin1.GetString($moov, $i + 4, 4)
    $field = $names[$name]
    if ($field) {
      $data = Find-Mp4Box $moov ($i + 8) ([int][Math]::Min($i + $size, $ilst.End)) 'data'
      # A "data" box: version and flags (4 bytes), locale (4 bytes), then the value.
      if ($data -and $data.End - $data.Start -gt 8) {
        $from = $data.Start + 8
        if ($name -ceq 'trkn') {
          if ($data.End - $from -ge 4) { $tags.Track = [string](([int]$moov[$from + 2] -shl 8) -bor [int]$moov[$from + 3]) }
        } elseif ($name -ceq 'covr') {
          $tags.Picture = Copy-Bytes $moov $from ($data.End - $from)
        } else {
          $tags[$field] = [Text.Encoding]::UTF8.GetString($moov, $from, $data.End - $from)
        }
      }
    }
    $i += $size
  }
  $tags
}

# The comments of a Vorbis comment block that starts at $start of $block (a vendor string, then
# a count and "NAME=value" strings, little endian), as tags. Flac, Ogg Vorbis and Opus carry them.
function Read-VorbisComments([byte[]]$block, [int]$start, $tags) {
  $fields = @{ TITLE = 'Title'; ARTIST = 'Artist'; ALBUM = 'Album'; DATE = 'Year'; YEAR = 'Year'; GENRE = 'Genre'
    TRACKNUMBER = 'Track'; TRACK = 'Track'; LYRICS = 'Lyrics'; UNSYNCEDLYRICS = 'Lyrics'; METADATA_BLOCK_PICTURE = 'Picture' }
  if ($block.Length -lt $start + 8) { return $tags }
  $i = $start + 4 + [int](ConvertTo-UInt32LE $block $start)
  if ($i + 4 -gt $block.Length) { return $tags }
  $count = ConvertTo-UInt32LE $block $i
  $i += 4
  for ($n = 0; $n -lt $count -and $i + 4 -le $block.Length; $n++) {
    $len = ConvertTo-UInt32LE $block $i
    $i += 4
    if ($len -gt $block.Length - $i) { break }
    $entry = [Text.Encoding]::UTF8.GetString($block, $i, [int]$len)
    $i += [int]$len
    $eq = $entry.IndexOf('=')
    if ($eq -lt 1) { continue }
    $field = $fields[$entry.Substring(0, $eq).ToUpperInvariant()]
    if ($field -eq 'Picture') {
      try { Set-FlacPicture $tags ([Convert]::FromBase64String($entry.Substring($eq + 1))) } catch { }
    } elseif ($field -and $tags[$field] -eq '') {
      $tags[$field] = $entry.Substring($eq + 1)
    }
  }
  $tags
}

# The tags of a flac file (its Vorbis comments), lyrics comprised.
function Read-Flac([string]$path) {
  $tags = New-Tags
  $block = $null
  $fs = [IO.File]::Open($path, 'Open', 'Read', 'ReadWrite')
  try {
    $h = Read-Bytes $fs 10
    # An ID3v2 tag may come before the "fLaC" mark.
    if ($h.Length -ge 10 -and $h[0] -eq 0x49 -and $h[1] -eq 0x44 -and $h[2] -eq 0x33) {
      $skip = ([int]$h[6] -shl 21) -bor ([int]$h[7] -shl 14) -bor ([int]$h[8] -shl 7) -bor [int]$h[9]
      $fs.Position = 10 + $skip
    } else {
      $fs.Position = 0
    }
    $mark = Read-Bytes $fs 4
    if ($mark.Length -lt 4 -or [Text.Encoding]::ASCII.GetString($mark) -cne 'fLaC') { return $tags }
    while ($true) {
      $bh = Read-Bytes $fs 4
      if ($bh.Length -lt 4) { break }
      $last = ($bh[0] -band 0x80) -ne 0
      $type = [int]($bh[0] -band 0x7F)
      $len = ([int]$bh[1] -shl 16) -bor ([int]$bh[2] -shl 8) -bor [int]$bh[3]
      if ($type -eq 4 -and $len -le 4MB) {
        $block = Read-Bytes $fs $len
      } elseif ($type -eq 6 -and $len -le 16MB) {
        Set-FlacPicture $tags (Read-Bytes $fs $len)
      } else {
        $fs.Position += $len
      }
      if ($last) { break }
    }
  } finally { $fs.Dispose() }
  if ($null -eq $block) { return $tags }
  Read-VorbisComments $block 0 $tags
}

# The tags of an Ogg Vorbis or Opus file: the comments are the second packet of the stream, which
# may be spread over several Ogg pages.
function Read-Ogg([string]$path) {
  $tags = New-Tags
  $fs = [IO.File]::Open($path, 'Open', 'Read', 'ReadWrite')
  try { $data = Read-Bytes $fs ([int][Math]::Min($fs.Length, 4MB)) } finally { $fs.Dispose() }
  $packets = @()
  $current = New-Object IO.MemoryStream
  $i = 0
  while ($packets.Count -lt 2 -and $i + 27 -le $data.Length) {
    if ([Text.Encoding]::ASCII.GetString($data, $i, 4) -cne 'OggS') { break }
    $segments = [int]$data[$i + 26]
    $pos = $i + 27 + $segments
    if ($pos -gt $data.Length) { break }
    for ($s = 0; $s -lt $segments -and $packets.Count -lt 2; $s++) {
      $len = [int]$data[$i + 27 + $s]
      if ($pos + $len -gt $data.Length) { $pos = $data.Length + 1; break }
      $current.Write($data, $pos, $len)
      $pos += $len
      if ($len -lt 255) {   # a segment of 255 bytes goes on in the next one
        $packets += , $current.ToArray()
        $current.SetLength(0)
      }
    }
    if ($pos -gt $data.Length) { break }
    $i = $pos
  }
  if ($packets.Count -lt 2) { return $tags }
  $packet = [byte[]]$packets[1]
  $head = [Text.Encoding]::ASCII.GetString($packet, 0, [Math]::Min(8, $packet.Length))
  $start = if ($head.StartsWith('OpusTags')) { 8 } elseif ($packet[0] -eq 3 -and $head.Substring(1).StartsWith('vorbis')) { 7 } else { -1 }
  if ($start -lt 0) { return $tags }
  Read-VorbisComments $packet $start $tags
}

# The exact duration of an mp3 from the Xing / Info header of its first frame (the encoder wrote
# the number of frames there); 0 when it has none. Windows only estimates the duration of an
# mp3 from its size, which can be several seconds off, and lrclib.net matches within 2 seconds.
function Get-Mp3Duration([string]$path) {
  try {
    $fs = [IO.File]::Open($path, 'Open', 'Read', 'ReadWrite')
    try {
      $h = Read-Bytes $fs 10
      $skip = 0
      if ($h.Length -ge 10 -and $h[0] -eq 0x49 -and $h[1] -eq 0x44 -and $h[2] -eq 0x33) {
        $skip = 10 + (([int]$h[6] -shl 21) -bor ([int]$h[7] -shl 14) -bor ([int]$h[8] -shl 7) -bor [int]$h[9])
      }
      $fs.Position = $skip
      $b = Read-Bytes $fs 4096
    } finally { $fs.Dispose() }
    for ($i = 0; $i + 40 -lt $b.Length; $i++) {
      if ($b[$i] -ne 0xFF -or ($b[$i + 1] -band 0xE0) -ne 0xE0) { continue }
      $version = ($b[$i + 1] -shr 3) -band 3       # 3 = MPEG 1, 2 = MPEG 2, 0 = MPEG 2.5
      $layer = ($b[$i + 1] -shr 1) -band 3          # 1 = layer III
      $rate = ($b[$i + 2] -shr 2) -band 3
      if ($version -eq 1 -or $layer -ne 1 -or $rate -eq 3) { continue }
      $mono = (($b[$i + 3] -shr 6) -band 3) -eq 3
      $mpeg1 = $version -eq 3
      $offset = $i + 4 + $(if (($b[$i + 1] -band 1) -eq 0) { 2 } else { 0 }) + $(if ($mpeg1) { if ($mono) { 17 } else { 32 } } else { if ($mono) { 9 } else { 17 } })
      if ($offset + 12 -gt $b.Length) { return 0.0 }
      $mark = [Text.Encoding]::ASCII.GetString($b, $offset, 4)
      if ($mark -cne 'Xing' -and $mark -cne 'Info') { return 0.0 }
      if (($b[$offset + 7] -band 1) -eq 0) { return 0.0 }   # no frame count
      $frames = ConvertTo-UInt32BE $b ($offset + 8)
      $rates = if ($mpeg1) { 44100, 48000, 32000 } elseif ($version -eq 2) { 22050, 24000, 16000 } else { 11025, 12000, 8000 }
      $samples = if ($mpeg1) { 1152 } else { 576 }
      return [double]($frames * $samples / $rates[$rate])
    }
  } catch { Write-Log "Duration of ${path}: $($_.Exception.Message)" }
  0.0
}

$ShellApp = $null
function Get-ShellItem([string]$path) {
  if ($null -eq $script:ShellApp) { $script:ShellApp = New-Object -ComObject Shell.Application }
  $folder = $script:ShellApp.NameSpace([IO.Path]::GetDirectoryName($path))
  if ($null -eq $folder) { return $null }
  $folder.ParseName([IO.Path]::GetFileName($path))
}

# "mp3, 44.1 kHz, 16 bit, 1.41 Mbit/s" from the file's extension and Windows' property system (what
# File Explorer shows): the sampling rate, the bit depth (only for lossless and uncompressed files:
# a lossy file has none) and the bit rate; for a video "mp4, 1920x1080, 25 fps, 48 kHz, 4.2 Mbit/s".
# Just the extension when Windows knows none of the rest.
function Get-MediaInfo([string]$path, $tags) {
  try {
    $item = Get-ShellItem $path
    if ($null -eq $item) { return '' }
    $rate = 0.0; $bits = 0.0; $bps = 0.0; $width = 0.0; $height = 0.0; $fps = 0.0
    $value = $item.ExtendedProperty('System.Audio.SampleRate'); if ($value) { $rate = [double]$value }
    $value = $item.ExtendedProperty('System.Audio.SampleSize'); if ($value) { $bits = [double]$value }
    $ext = [IO.Path]::GetExtension($path).ToLowerInvariant()
    $video = $AllVideoExtensions -contains $ext
    if ($video) {
      $value = $item.ExtendedProperty('System.Video.FrameWidth'); if ($value) { $width = [double]$value }
      $value = $item.ExtendedProperty('System.Video.FrameHeight'); if ($value) { $height = [double]$value }
      $value = $item.ExtendedProperty('System.Video.FrameRate'); if ($value) { $fps = [double]$value / 1000 }   # in 1/1000 frame per second
      $value = $item.ExtendedProperty('System.Video.TotalBitrate'); if ($value) { $bps = [double]$value }
    } else {
      $value = $item.ExtendedProperty('System.Audio.EncodingBitrate'); if ($value) { $bps = [double]$value }
    }
    $lossless = $tags.Lossless -or (@('.flac', '.wav', '.aif', '.aiff', '.au') -contains $ext)
    # The file's extension comes first (mp3, mp4, wav...).
    $parts = @()
    if ($ext) { $parts += $ext.TrimStart('.') }
    if ($width -gt 0 -and $height -gt 0) { $parts += ('{0:0}x{1:0}' -f $width, $height) }
    if ($fps -gt 0) { $parts += ('{0:0.##} fps' -f $fps) }
    if ($rate -gt 0) { $parts += ('{0:0.###} kHz' -f ($rate / 1000)) }
    if ($lossless -and $bits -gt 0) { $parts += ('{0:0} bit' -f $bits) }
    if ($bps -ge 1000000) { $parts += ('{0:0.##} Mbit/s' -f ($bps / 1000000)) }
    elseif ($bps -gt 0) { $parts += ('{0:0} kbit/s' -f ($bps / 1000)) }
    return ($parts -join ', ')
  } catch {
    Write-Log "Media properties of ${path}: $($_.Exception.Message)"
    return ''
  }
}

# The Store extension a video's codec needs, as a key of $StorePackages ('' when none, or unknown):
# from the FourCC code or the compression GUID Windows' property system gives (empty for mkv).
function Get-VideoCodecKey([string]$path) {
  try {
    $item = Get-ShellItem $path
    if ($null -eq $item) { return '' }
    $four = ''
    $value = $item.ExtendedProperty('System.Video.FourCC')
    $guid = [string]$item.ExtendedProperty('System.Video.Compression')
    if ($value) {
      $four = [Text.Encoding]::ASCII.GetString([BitConverter]::GetBytes([int64]$value), 0, 4)
    } elseif ($guid -match '^\{([0-9A-Fa-f]{8})-') {
      $four = [Text.Encoding]::ASCII.GetString([BitConverter]::GetBytes([Convert]::ToInt64($Matches[1], 16)), 0, 4)
    }
    $four = ($four -replace '[^\x20-\x7E]', '').Trim().ToUpperInvariant()
    if (@('HEVC', 'HVC1', 'HEV1', 'H265', 'X265') -contains $four) { return 'hevc' }
    if ($four -eq 'AV01') { return 'av1' }
    if (@('VP90', 'VP09', 'VP9') -contains $four) { return 'vp9' }
    if (@('MPG2', 'MP2V', 'MPEG', 'MPG1') -contains $four -or $guid -match '^\{E06D8026-') { return 'mpeg2' }
  } catch { Write-Log "Codec of ${path}: $($_.Exception.Message)" }
  ''
}

# A file that no player could play, or a video that shows nothing: the extensions it may lack.
function Register-Needs([string]$path) {
  $ext = [IO.Path]::GetExtension($path).ToLowerInvariant()
  if ($StoreExtensionPackage[$ext]) { Add-Need $StoreExtensionPackage[$ext] $path }
  if ($ext -eq '.webm') { Add-Need 'vp9' $path }
  if ($AllVideoExtensions -contains $ext) {
    $key = Get-VideoCodecKey $path
    if ($key) { Add-Need $key $path }
  }
}

# The picture file of the song's cover, shown above its title: the picture of its tags, else a
# cover.jpg (and the like) of its folder. The picture is kept once in the covers folder of the
# data folder, named after its content, so the songs of an album share it and the path is
# short. An empty string when there is none.
function Get-CoverPath([string]$path, $tags) {
  try {
    $bytes = $tags.Picture
    if (-not $bytes -or $bytes.Length -lt 16) {
      $bytes = $null
      $dir = [IO.Path]::GetDirectoryName($path)
      $files = @(Get-ChildItem -LiteralPath $dir -File -ErrorAction SilentlyContinue |
          Where-Object { $CoverExtensions -contains $_.Extension.ToLowerInvariant() -and $CoverNames -contains $_.BaseName.ToLowerInvariant() })
      $best = $files | Sort-Object { [Array]::IndexOf($CoverNames, $_.BaseName.ToLowerInvariant()) } | Select-Object -First 1
      if ($best -and $best.Length -le 16MB) { $bytes = [IO.File]::ReadAllBytes($best.FullName) }
    }
    if (-not $bytes -or $bytes.Length -lt 16) { return '' }
    $ext = if ($bytes[0] -eq 0xFF -and $bytes[1] -eq 0xD8) { '.jpg' }
      elseif ($bytes[0] -eq 0x89 -and $bytes[1] -eq 0x50) { '.png' }
      elseif ($bytes[0] -eq 0x47 -and $bytes[1] -eq 0x49) { '.gif' }
      elseif ($bytes[0] -eq 0x42 -and $bytes[1] -eq 0x4D) { '.bmp' }
      else { '.img' }
    $sha = [Security.Cryptography.SHA1]::Create()
    $hash = ($sha.ComputeHash($bytes) | ForEach-Object { $_.ToString('x2') }) -join ''
    $sha.Dispose()
    $folder = Join-Path $DataDir 'covers'
    $file = Join-Path $folder "$hash$ext"
    New-Item -ItemType Directory -Force -Path $folder | Out-Null
    if (Test-Path -LiteralPath $file -PathType Leaf) {
      (Get-Item -LiteralPath $file).LastWriteTime = Get-Date   # in use: the last to go
    } else {
      [IO.File]::WriteAllBytes($file, $bytes)
      # The cache keeps the newest pictures only.
      $old = @(Get-ChildItem -LiteralPath $folder -File | Sort-Object LastWriteTime -Descending | Select-Object -Skip $Layout.CoversKept)
      foreach ($stale in $old) { Remove-Item -LiteralPath $stale.FullName -Force -ErrorAction SilentlyContinue }
    }
    # A line is read up to 300 characters, and the path is written in it.
    if ($file.Length -gt 240) { return '' }
    return $file
  } catch {
    Write-Log "Cover of ${path}: $($_.Exception.Message)"
    return ''
  }
}

# The tags of a song, whatever its format (empty ones when it has none or cannot be read).
function Get-Tags([string]$path) {
  $tags = New-Tags
  try {
    switch ([IO.Path]::GetExtension($path).ToLowerInvariant()) {
      '.mp3' { $tags = Read-Id3 $path }
      { $_ -eq '.m4a' -or $_ -eq '.m4b' } { $tags = Read-Mp4 $path }
      '.flac' { $tags = Read-Flac $path }
      { $_ -eq '.ogg' -or $_ -eq '.oga' -or $_ -eq '.opus' } { $tags = Read-Ogg $path }
    }
  } catch {
    Write-Log "Tags of ${path}: $($_.Exception.Message)"
  }
  Complete-Tags $tags
}

# ---- Lyrics -------------------------------------------------------------------------------
# A text file in UTF-8 (with or without BOM) or UTF-16, else in the system's ANSI code page.
function Read-TextFile([string]$path) {
  $b = [IO.File]::ReadAllBytes($path)
  if ($b.Length -ge 3 -and $b[0] -eq 0xEF -and $b[1] -eq 0xBB -and $b[2] -eq 0xBF) { return [Text.Encoding]::UTF8.GetString($b, 3, $b.Length - 3) }
  if ($b.Length -ge 2 -and $b[0] -eq 0xFF -and $b[1] -eq 0xFE) { return [Text.Encoding]::Unicode.GetString($b, 2, $b.Length - 2) }
  if ($b.Length -ge 2 -and $b[0] -eq 0xFE -and $b[1] -eq 0xFF) { return [Text.Encoding]::BigEndianUnicode.GetString($b, 2, $b.Length - 2) }
  try { (New-Object Text.UTF8Encoding($false, $true)).GetString($b) } catch { [Text.Encoding]::Default.GetString($b) }
}

# The pieces of a line that are highlighted one after the other: words (each with the space
# that follows it), or single characters in a text without spaces (Chinese, Japanese, Korean).
# Their times are spread over `span` seconds from `t0`, longer pieces taking longer.
function New-Chunks([string]$text, [double]$t0, [double]$span) {
  $pieces = @()
  if ($text -notmatch '\s' -and $text -match '[぀-ヿ㐀-鿿가-힯]') {
    $pieces = @([regex]::Matches($text, '.') | ForEach-Object { $_.Value })
  } else {
    $pieces = @([regex]::Matches($text, '\S+\s*|\s+') | ForEach-Object { $_.Value })
  }
  $total = 0.0
  foreach ($p in $pieces) { $total += [Math]::Max(1, $p.Trim().Length) }
  $chunks = New-Object System.Collections.ArrayList
  $before = 0.0
  foreach ($p in $pieces) {
    [void]$chunks.Add(@{ t = $t0 + $span * $before / [Math]::Max(1, $total); s = $p })
    $before += [Math]::Max(1, $p.Trim().Length)
  }
  , $chunks
}

# How long a line of `chars` characters takes to sing at most (an instrumental part after it
# must not stretch the last word over the whole gap).
function Get-SingSpan([int]$chars, [double]$gap) {
  [Math]::Max(0.5, [Math]::Min($gap, 0.5 + 0.18 * $chars))
}

# Lyrics from text: @{ Synced = $true/$false; Lines = @( @{ t; text; chunks } ... ) }. Without
# any time stamp the lines get no time yet (Set-UnsyncedTimes spreads them over the song).
function New-Lyrics([string]$text) {
  $offset = 0.0
  $found = New-Object System.Collections.ArrayList
  $plain = New-Object System.Collections.ArrayList
  $n = 0
  foreach ($raw in ($text -split "\r?\n")) {
    $line = $raw.Trim()
    if ($line -match '^\[offset:\s*([+-]?\d+)\s*\]$') { $offset = [double]$Matches[1] / 1000; continue }
    if ($line -match '^\[[A-Za-z]+:[^\]]*\]$') { continue }
    $stamp = [regex]::Match($line, '^((?:\[\d+:\d+(?:[.:]\d+)?\])+)(.*)$')
    if ($stamp.Success) {
      $body = $stamp.Groups[2].Value
      $parts = [regex]::Split($body, '<(\d+):(\d+(?:[.:]\d+)?)>')
      foreach ($m in [regex]::Matches($stamp.Groups[1].Value, '\[(\d+):(\d+(?:[.:]\d+)?)\]')) {
        $t = [double]$m.Groups[1].Value * 60 + [double]($m.Groups[2].Value -replace ':', '.') - $offset
        $entry = @{ t = [Math]::Max(0.0, $t); i = $n++; chunks = $null; text = '' }
        if ($parts.Count -ge 3) {
          # Enhanced: <mm:ss.xx> before each piece of text.
          $chunks = New-Object System.Collections.ArrayList
          if ($parts[0].Trim()) { [void]$chunks.Add(@{ t = $entry.t; s = $parts[0] }) }
          for ($k = 1; $k + 2 -lt $parts.Count; $k += 3) {
            $wt = [double]$parts[$k] * 60 + [double]($parts[$k + 1] -replace ':', '.') - $offset
            [void]$chunks.Add(@{ t = [Math]::Max(0.0, $wt); s = $parts[$k + 2] })
          }
          $entry.chunks = $chunks
          $entry.text = (($chunks | ForEach-Object { $_.s }) -join '').Trim()
        } else {
          $entry.text = $body.Trim()
        }
        [void]$found.Add($entry)
      }
    } elseif ($line) {
      [void]$plain.Add($line)
    }
  }
  if ($found.Count -gt 0) {
    $sorted = @($found | Sort-Object { $_.t }, { $_.i })
    for ($k = 0; $k -lt $sorted.Count; $k++) {
      $e = $sorted[$k]
      if ($null -eq $e.chunks) {
        $gap = if ($k + 1 -lt $sorted.Count) { $sorted[$k + 1].t - $e.t } else { 8.0 }
        $e.chunks = New-Chunks $e.text $e.t (Get-SingSpan $e.text.Length $gap)
      }
    }
    return @{ Synced = $true; Timed = $true; Lines = $sorted }
  }
  $lines = @()
  foreach ($p in $plain) { $lines += , @{ t = 0.0; text = $p; chunks = $null } }
  @{ Synced = $false; Timed = $false; Lines = $lines }
}

# Spreads lyrics without times over the song's duration (the first and last few percent are
# left free for the intro and the end), longer lines taking longer.
function Set-UnsyncedTimes($lyrics, [double]$duration) {
  $lines = $lyrics.Lines
  $weights = @($lines | ForEach-Object { [Math]::Max(1, $_.text.Length) + 8 })
  $total = ($weights | Measure-Object -Sum).Sum
  $from = $duration * 0.06
  $room = $duration * 0.88
  $before = 0.0
  for ($k = 0; $k -lt $lines.Count; $k++) {
    $lines[$k].t = $from + $room * $before / $total
    $before += $weights[$k]
    $gap = $room * $weights[$k] / $total
    $lines[$k].chunks = New-Chunks $lines[$k].text $lines[$k].t (Get-SingSpan $lines[$k].text.Length $gap)
  }
  $lyrics.Timed = $true
}

# A .srt (subtitles) as a .lrc: "[mm:ss.xx]text" for the start of each cue, its lines joined.
function ConvertFrom-Srt([string]$text) {
  $out = New-Object 'System.Collections.Generic.List[string]'
  $invariant = [Globalization.CultureInfo]::InvariantCulture
  foreach ($block in [regex]::Split($text.Trim([char]0xFEFF), '\r?\n[ \t]*\r?\n')) {
    $lines = @($block -split '\r?\n' | ForEach-Object { $_.Trim() } | Where-Object { $_ })
    $at = -1
    for ($k = 0; $k -lt $lines.Count; $k++) { if ($lines[$k] -match '-->') { $at = $k; break } }
    if ($at -lt 0) { continue }
    $time = [regex]::Match($lines[$at], '(?:(\d+):)?(\d+):(\d+)[,.](\d+)\s*-->')
    if (-not $time.Success) { continue }
    $hours = if ($time.Groups[1].Success) { [double]$time.Groups[1].Value } else { 0.0 }
    $seconds = $hours * 3600 + [double]$time.Groups[2].Value * 60 + [double]$time.Groups[3].Value +
      [double]('0.' + $time.Groups[4].Value)
    # The text of the cue, without its tags: <i>...</i> and the {\an8} of a position.
    $words = (($lines | Select-Object -Skip ($at + 1)) -join ' ') -replace '<[^>]+>', '' -replace '\{\\[^}]*\}', ''
    if (-not $words.Trim()) { continue }
    $out.Add([string]::Format($invariant, '[{0:00}:{1:00.00}]{2}', [Math]::Floor($seconds / 60), $seconds % 60, $words.Trim()))
  }
  $out -join "`n"
}

# The lyrics of a song (or the subtitles of a video): .lrc, .srt, .txt, then its tags; $null when
# there are none.
function Get-Lyrics([string]$path, $tags) {
  $dir = [IO.Path]::GetDirectoryName($path)
  $base = Join-Path $dir ([IO.Path]::GetFileNameWithoutExtension($path))
  foreach ($ext in '.lrc', '.srt', '.txt') {
    if (Test-Path -LiteralPath ($base + $ext) -PathType Leaf) {
      try {
        $text = Read-TextFile ($base + $ext)
        if ($ext -eq '.srt') { $text = ConvertFrom-Srt $text }
        if ($text.Trim()) { return New-Lyrics $text }
      } catch { }
    }
  }
  if ($tags.Lyrics.Trim()) { return New-Lyrics $tags.Lyrics }
  $null
}

# ---- Work in the background ---------------------------------------------------------------
# A script block that runs on a thread of its own (a runspace): the main loop goes on reading
# clicks and drawing the player meanwhile. The block knows nothing of this script, only of
# its arguments.
function Start-Background([scriptblock]$code, [object[]]$arguments) {
  $shell = [powershell]::Create()
  [void]$shell.AddScript($code.ToString())
  foreach ($argument in $arguments) { [void]$shell.AddArgument($argument) }
  @{ Shell = $shell; Handle = $shell.BeginInvoke() }
}

function Stop-Background($job) {
  if ($job) { try { [void]$job.Shell.BeginStop($null, $null) } catch { } }
}

# What a finished background job returned, as an array.
function Receive-Background($job) {
  $results = @()
  try { $results = @($job.Shell.EndInvoke($job.Handle)) } catch { Write-Log "Background job: $($_.Exception.Message)" }
  $job.Shell.Dispose()
  , $results
}

# Lists the songs of a folder, sorted.
$ScanCode = {
  param($folder, $recurse, $extensions, $max)
  Get-ChildItem -LiteralPath $folder -File -Recurse:$recurse -ErrorAction SilentlyContinue |
    Where-Object { $extensions -contains $_.Extension.ToLowerInvariant() } |
    Sort-Object FullName |
    Select-Object -First $max |
    ForEach-Object { $_.FullName }
}

# One request (GET, or `method` with a JSON `body`), answered as @{ Status; Body; RetryAfter;
# ContentType } (Status 0: no answer). `headers`: more headers (an API key); redirects are followed.
$HttpCode = {
  param($url, $userAgent, $timeoutMs, $accept, $headers, $method, $body)
  try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $request = [Net.HttpWebRequest]::Create($url)
    $request.UserAgent = $userAgent
    $request.Accept = if ($accept) { $accept } else { 'application/json' }
    $request.Timeout = $timeoutMs
    if ($headers) { foreach ($name in $headers.Keys) { $request.Headers[$name] = [string]$headers[$name] } }
    if ($method) { $request.Method = $method }
    if ($body) {
      $bytes = [Text.Encoding]::UTF8.GetBytes($body)
      $request.ContentType = 'application/json'
      $request.ContentLength = $bytes.Length
      $stream = $request.GetRequestStream()
      try { $stream.Write($bytes, 0, $bytes.Length) } finally { $stream.Dispose() }
    }
    try {
      $response = $request.GetResponse()
    } catch [Net.WebException] {
      $response = $_.Exception.Response
      if ($null -eq $response) { throw }
    }
    $reader = New-Object IO.StreamReader($response.GetResponseStream(), [Text.Encoding]::UTF8)
    $body = $reader.ReadToEnd()
    $reader.Dispose()
    $result = @{ Status = [int]$response.StatusCode; Body = $body; RetryAfter = $response.Headers['Retry-After']; ContentType = $response.ContentType }
    $response.Close()
    $result
  } catch {
    @{ Status = 0; Body = $_.Exception.Message; RetryAfter = $null; ContentType = '' }
  }
}

# Which of the Store extensions are installed, as the keys of $StorePackages. Windows' other media
# player (the one of Store apps) must be there too, as it is what plays the files that need them.
$CodecCode = {
  param($packages)
  try {
    [void][Windows.Media.Playback.MediaPlayer, Windows.Media, ContentType = WindowsRuntime]
    [void][Windows.Media.Core.MediaSource, Windows.Media, ContentType = WindowsRuntime]
    $names = @(Get-AppxPackage -ErrorAction Stop | ForEach-Object { $_.Name })
    foreach ($key in $packages.Keys) {
      if ($names -contains $packages[$key]) { [string]$key }
    }
  } catch { }
}

function Start-CodecCheck {
  if ($State.Codec) { return }
  $State.CodecCheckedAt = [DateTime]::UtcNow
  $names = @{}
  foreach ($key in $StorePackages.Keys) { $names[$key] = $StorePackages[$key].Package }
  $State.Codec = Start-Background $CodecCode @($names)
}

function Test-StoreInstalled([string]$key) {
  [bool]($State.Installed -and $State.Installed[$key])
}

# Notes that `path` needs the extension `key`, which is not installed (for the hints at the bottom).
function Add-Need([string]$key, [string]$path) {
  if (-not $StorePackages.Contains($key) -or (Test-StoreInstalled $key)) { return }
  if (-not $State.Needs.ContainsKey($key)) { $State.Needs[$key] = @() }
  if ($State.Needs[$key] -notcontains $path) { $State.Needs[$key] += $path }
}

# ---- Songs --------------------------------------------------------------------------------
# The songs of a playlist file (.m3u, .m3u8, .pls or .wpl) that exist and are of a known format.
function Read-Playlist([string]$path) {
  $dir = [IO.Path]::GetDirectoryName($path)
  $ext = [IO.Path]::GetExtension($path).ToLowerInvariant()
  $entries = New-Object 'System.Collections.Generic.List[string]'
  if ($ext -eq '.wpl') {
    $doc = New-Object System.Xml.XmlDocument
    $doc.Load($path)
    foreach ($node in $doc.SelectNodes('//media')) { $entries.Add($node.GetAttribute('src')) }
  } else {
    foreach ($raw in ((Read-TextFile $path) -split "\r?\n")) {
      $line = $raw.Trim()
      if (-not $line -or $line.StartsWith('#') -or $line.StartsWith(';') -or $line.StartsWith('[')) { continue }
      if ($ext -eq '.pls') {
        if ($line -match '^File\d+=(.+)$') { $entries.Add($Matches[1]) }
      } else {
        $entries.Add($line)
      }
    }
  }
  $songs = @()
  foreach ($entry in $entries) {
    $file = $entry.Trim()
    if ($file -match '^file:///') {
      try { $file = ([Uri]$file).LocalPath } catch { continue }
    } elseif ($file -match '^[A-Za-z][A-Za-z0-9+.-]+://') {
      continue   # a stream on the Internet
    }
    try {
      $file = $file -replace '/', '\'
      if (-not [IO.Path]::IsPathRooted($file)) { $file = Join-Path $dir $file }
      $file = [IO.Path]::GetFullPath($file)
    } catch { continue }
    if ($ListedExtensions -contains [IO.Path]::GetExtension($file).ToLowerInvariant() -and (Test-Path -LiteralPath $file -PathType Leaf)) {
      $songs += $file
    }
    if ($songs.Count -ge $MaxSongs) { break }
  }
  , $songs
}

# Lists the songs of the folder in the background.
function Start-Scan {
  Stop-Background $State.Scan
  $State.Loading = $true
  $State.Scan = Start-Background $ScanCode @($cfg.folder, $Recurse, $ListedExtensions, $MaxSongs)
}

# The play order: the songs as listed, or shuffled with `first` (when it is 0 or more) played
# first.
function New-Order([int]$first) {
  $n = $State.Files.Count
  if ($n -eq 0) { $State.Order = @(); return }
  if ($cfg.shuffle) {
    $rest = @(0..($n - 1) | Where-Object { $_ -ne $first } | Sort-Object { Get-Random })
    $State.Order = if ($first -ge 0) { @($first) + $rest } else { $rest }
  } else {
    $State.Order = @(0..($n - 1))
  }
}

# Makes the songs of a playlist file the songs to play.
function Open-Playlist([string]$path) {
  $songs = @()
  try { $songs = Read-Playlist $path } catch { Write-Log "Playlist ${path}: $($_.Exception.Message)" }
  Stop-Background $State.Scan
  $State.Scan = $null
  $cfg.playlist = $path
  Save-Settings -Now
  Stop-Media
  $State.Radio.On = $false   # Play now plays the files of the playlist
  $State.Loading = $true
  $State.Candidates = @($songs)
}

# The files to play out of the files found: the ones that need a Store extension (Ogg, Opus, WebM...)
# when it is installed, else counted for the hint at the bottom. The file being played, if any,
# goes on.
function Set-Songs {
  $found = @($State.Candidates)
  $State.Candidates = $null
  $State.Needs = @{}
  $playable = @()
  foreach ($file in $found) {
    $key = $StoreExtensionPackage[[IO.Path]::GetExtension($file).ToLowerInvariant()]
    if ($key -and -not (Test-StoreInstalled $key)) { Add-Need $key $file } else { $playable += $file }
  }
  $current = if ($State.Cur -ge 0) { $State.Files[$State.Cur] } else { $null }
  $State.Files = $playable
  $State.Loading = $false
  if ($current) {
    $State.Cur = [Array]::IndexOf($State.Files, $current)
    if ($State.Cur -lt 0) { Stop-Media }
  }
  New-Order $State.Cur
  if ($State.Cur -ge 0) { $State.Pos = if ($cfg.shuffle) { 0 } else { $State.Cur } }
}

# ---- Playback -----------------------------------------------------------------------------
# A crossfade hands the file that fades out to a player of its own (Split-Tail), so each WPF
# player tells about its own file only while it is the one being played.
function New-WpfPlayer {
  $new = New-Object System.Windows.Media.MediaPlayer
  $new.Add_MediaEnded({ param($sender) if ([object]::ReferenceEquals($sender, $script:player)) { $State.MediaEnded = $true } })
  $new.Add_MediaFailed({ param($sender) if ([object]::ReferenceEquals($sender, $script:player)) { $State.MediaFailed = $true } })
  $new
}
$player = New-WpfPlayer

# The player tells about a finished or failed song through the thread's dispatcher.
function Pump {
  $script:pumpFrame = New-Object System.Windows.Threading.DispatcherFrame
  [void][System.Windows.Threading.Dispatcher]::CurrentDispatcher.BeginInvoke(
    [System.Windows.Threading.DispatcherPriority]::Background,
    [Action] { $script:pumpFrame.Continue = $false })
  [System.Windows.Threading.Dispatcher]::PushFrame($script:pumpFrame)
}

# Songs are played by one of two Windows media players (videos by PecoFence's own, see Open-Engine).
# The one above (WPF's) plays what Windows plays by itself. The one of Store apps (WinRT) also plays
# Ogg Vorbis, Opus and the codecs of the Store's extensions once they are installed; it is made the
# first time a file needs it, and its state is read at every redraw.
$rt = $null

function Get-WinRtPlayer {
  if ($null -eq $script:rt) {
    [void][Windows.Media.Playback.MediaPlayer, Windows.Media, ContentType = WindowsRuntime]
    [void][Windows.Media.Core.MediaSource, Windows.Media, ContentType = WindowsRuntime]
    $script:rt = New-Object Windows.Media.Playback.MediaPlayer
  }
  $script:rt
}

# The volume of the players; during a crossfade the file that comes fades in and the one that
# goes fades out, at constant power (no dip in the middle).
function Set-Volume {
  $level = if ($cfg.muted) { 0.0 } else { $cfg.volume / 100.0 }
  $in = 1.0
  if ($State.Fade) {
    $angle = (Get-FadeProgress) * [Math]::PI / 2
    $in = [Math]::Sin($angle)
    $State.Fade.Tail.Player.Volume = $level * [Math]::Cos($angle)
  }
  $player.Volume = $level * $in
  if ($script:rt) { $script:rt.Volume = $level * $in }
  if ($State.Native) { $State.Native.Volume = $level * $in }
}

# The player that plays the file first: PecoFence's own for a video, Windows' other media player for
# the audio formats that need a Store extension (once it is installed), WPF's for the rest. When
# WPF's cannot play a file (a codec that only the other one has), Update-Playback tries the other one.
function Get-EngineFor([string]$path) {
  # A video is played by PecoFence itself, with the codecs Windows has (and the Store's, once installed).
  if ($AllVideoExtensions -contains [IO.Path]::GetExtension($path).ToLowerInvariant()) { return 'native' }
  $key = $StoreExtensionPackage[[IO.Path]::GetExtension($path).ToLowerInvariant()]
  if ($key -and (Test-StoreInstalled $key)) { 'winrt' } else { 'wpf' }
}

# Numbers the moves asked of PecoFence's player (the "seekid" of the row), over all the videos.
$script:NativeSeeks = 0

# Starts the file in the player `engine`.
function Open-Engine([string]$path, [string]$engine) {
  if ($engine -eq 'native') {
    # The "video=" row of the screen makes PecoFence open the file; the other players are silent.
    $player.Stop()
    $player.Close()
    if ($script:rt) { try { $script:rt.Pause(); $script:rt.Source = $null } catch { } }
    $State.Engine = 'native'
    $State.RtSource = $null
    # Every start asks for a move to 0 with a number of its own: the same file played again (Repeat
    # one, a list of one file) starts over instead of staying where it ended.
    $script:NativeSeeks++
    $State.Native = @{
      Path = $path; Playing = $true; Volume = 0.0; Speed = $State.Speed; Seek = 0.0; SeekId = $script:NativeSeeks
      State = 'loading'; Position = 0.0; Duration = 0.0; At = [DateTime]::UtcNow; HoldUntil = [DateTime]::MinValue
    }
    Set-Volume
    return
  }
  if ($engine -eq 'winrt') {
    $State.Engine = 'winrt'
    $State.Native = $null
    $player.Stop()
    $player.Close()
    $mp = Get-WinRtPlayer
    $State.RtSource = [Windows.Media.Core.MediaSource]::CreateFromUri((New-Object System.Uri($path)))
    $mp.Source = $State.RtSource
    Set-Volume
    $mp.Play()
    Set-EngineSpeed
  } else {
    $State.Engine = 'wpf'
    $State.Native = $null
    $State.RtSource = $null
    if ($script:rt) { try { $script:rt.Pause(); $script:rt.Source = $null } catch { } }
    $player.Open((New-Object System.Uri($path)))
    Set-Volume
    $player.Play()
    Set-EngineSpeed
  }
}

# The playback speed goes to the player of the file being played; PecoFence's own gets it with the
# video row (the row says "speed=").
function Set-EngineSpeed {
  # A radio is live: it cannot go faster than it comes.
  $speed = if ($State.Radio.On) { 1.0 } else { $State.Speed }
  if ($State.Native) { $State.Native.Speed = $speed }
  try { $player.SpeedRatio = $speed } catch { Write-Log "Speed: $($_.Exception.Message)" }
  try { if ($script:rt) { $script:rt.PlaybackSession.PlaybackRate = $speed } } catch { Write-Log "Speed: $($_.Exception.Message)" }
}

# The < and > buttons, and a click on the value (back to normal speed).
function Set-Speed([double]$speed) {
  if ($State.Radio.On -and $State.Mode -ne 'stopped') { return }
  $speed = [Math]::Round([Math]::Max([double]$Layout.SpeedMin, [Math]::Min([double]$Layout.SpeedMax, $speed)) / $Layout.SpeedStep) * $Layout.SpeedStep
  # The position of a video that PecoFence plays goes on by the clock: from now at the new speed.
  if ($State.Native) {
    $State.Native.Position = Get-EnginePosition
    $State.Native.At = [DateTime]::UtcNow
  }
  $State.Speed = $speed
  Set-EngineSpeed
}

function Stop-Engines {
  $player.Stop()
  $player.Close()
  if ($script:rt) {
    try { $script:rt.Pause(); $script:rt.Source = $null } catch { }
  }
  $State.RtSource = $null
  $State.Native = $null
}

# Pause and Play: the file fading out of a crossfade too.
function Suspend-Engine {
  if ($State.Engine -eq 'native') {
    $State.Native.Position = Get-EnginePosition
    $State.Native.At = [DateTime]::UtcNow
    $State.Native.HoldUntil = [DateTime]::UtcNow.AddSeconds($Timing.VideoHoldSeconds)
    $State.Native.Playing = $false
    $State.Native.State = 'paused'
  } elseif ($State.Engine -eq 'winrt') { $script:rt.Pause() } else { $player.Pause() }
  if ($State.Fade) { $State.Fade.Tail.Player.Pause() }
}
function Resume-Engine {
  if ($State.Engine -eq 'native') {
    $State.Native.At = [DateTime]::UtcNow
    $State.Native.HoldUntil = [DateTime]::UtcNow.AddSeconds($Timing.VideoHoldSeconds)
    $State.Native.Playing = $true
    $State.Native.State = 'playing'
  } elseif ($State.Engine -eq 'winrt') { $script:rt.Play() } else { $player.Play() }
  if ($State.Fade) { $State.Fade.Tail.Player.Play() }
}

function Get-EnginePosition {
  if ($State.Engine -eq 'native') {
    # PecoFence reports a few times a second: between two reports the position goes on by the clock.
    $n = $State.Native
    if ($n.State -ne 'playing') { return $n.Position }
    $now = $n.Position + ([DateTime]::UtcNow - $n.At).TotalSeconds * $n.Speed
    return $(if ($n.Duration -gt 0) { [Math]::Min($now, $n.Duration) } else { $now })
  }
  if ($State.Engine -eq 'winrt') { $script:rt.PlaybackSession.Position.TotalSeconds } else { $player.Position.TotalSeconds }
}

function Set-EnginePosition([double]$seconds) {
  if ($State.Engine -eq 'native') {
    # The row asks for the move (a new number makes it a new one); the slider shows it at once.
    $n = $State.Native
    $n.Seek = $seconds
    $script:NativeSeeks++
    $n.SeekId = $script:NativeSeeks
    $n.Position = $seconds
    $n.At = [DateTime]::UtcNow
    $n.HoldUntil = [DateTime]::UtcNow.AddSeconds($Timing.VideoHoldSeconds)
    return
  }
  if ($State.Engine -eq 'winrt') { $script:rt.PlaybackSession.Position = [TimeSpan]::FromSeconds($seconds) }
  else { $player.Position = [TimeSpan]::FromSeconds($seconds) }
}

# A line of PecoFence about the video it plays: "video:<state>:<position>:<duration>" (state:
# loading, playing, paused, ended), or "video:error:0:0:<code>" (4: the file is missing or Windows lacks its codec).
function Receive-VideoLine([string]$line) {
  $n = $State.Native
  if (-not $n) { return }
  $part = $line.Split(':')
  if ($part.Count -lt 4) { return }
  $invariant = [Globalization.CultureInfo]::InvariantCulture
  $position = 0.0
  $duration = 0.0
  [void][double]::TryParse($part[2], [Globalization.NumberStyles]::Float, $invariant, [ref]$position)
  [void][double]::TryParse($part[3], [Globalization.NumberStyles]::Float, $invariant, [ref]$duration)
  if ($duration -gt 0) { $n.Duration = $duration }
  if ($part[1] -eq 'error') {
    $n.State = 'error'
    Write-Log "Video: PecoFence could not play it (code $($part[4]))"
    return
  }
  # Just after a move, a pause or a play, the reports still say how it was before; an end is
  # taken at once (no other report follows it).
  $now = [DateTime]::UtcNow
  if ($now -lt $n.HoldUntil -and $part[1] -ne 'ended') { return }
  $n.State = $part[1]
  $n.Position = $position
  $n.At = $now
}

# ---- Crossfade ----------------------------------------------------------------------------
# How far the crossfade is, from 0 to 1: the position of the file that comes (it started with the
# crossfade), so a pause holds it and a move in the file moves it.
function Get-FadeProgress {
  [Math]::Max(0.0, [Math]::Min(1.0, (Get-EnginePosition) / $State.Fade.Seconds))
}

# The player of the file being played becomes the one that fades out; the next file gets a new
# one.
function Split-Tail {
  if ($State.Engine -eq 'winrt') {
    $tail = @{ Engine = 'winrt'; Player = $script:rt }
    $script:rt = $null
    $State.RtSource = $null
  } else {
    $tail = @{ Engine = 'wpf'; Player = $script:player }
    $script:player = New-WpfPlayer
  }
  $tail
}

# Whether a file comes after this one (the play order goes on, or starts again with Repeat all).
function Test-NextTrack {
  ($State.Pos + 1 -lt $State.Files.Count) -or ($cfg.repeat -eq 'all' -and $State.Files.Count -gt 0)
}

# The next file is a video: a crossfade is for songs (a video plays its own sound, from its start).
function Test-NextIsVideo {
  $next = $State.Pos + 1
  if ($next -ge $State.Order.Count) { $next = 0 }
  if ($next -ge $State.Order.Count) { return $false }
  $AllVideoExtensions -contains [IO.Path]::GetExtension($State.Files[$State.Order[$next]]).ToLowerInvariant()
}

# The file fades out over its last `seconds` while the next one starts and fades in.
function Start-Crossfade([double]$seconds) {
  $State.Fade = @{ Tail = (Split-Tail); Seconds = $seconds }
  $State.FadeStarting = $true
  try { Step 1 $true } finally { $State.FadeStarting = $false }
}

# Ends the crossfade: the file that faded out stops (at once, when another file is chosen).
function Stop-Crossfade {
  $fade = $State.Fade
  if (-not $fade) { return }
  $State.Fade = $null
  try {
    if ($fade.Tail.Engine -eq 'winrt') {
      $fade.Tail.Player.Pause()
      $fade.Tail.Player.Source = $null
      $fade.Tail.Player.Dispose()
    } else {
      $fade.Tail.Player.Stop()
      $fade.Tail.Player.Close()
    }
  } catch { Write-Log "Crossfade: $($_.Exception.Message)" }
  Set-Volume
}

# The volumes of a crossfade under way, and its end.
function Update-Crossfade {
  if (-not $State.Fade) { return }
  if ((Get-FadeProgress) -ge 1) { Stop-Crossfade } else { Set-Volume }
}

function Invoke-Crossfade {
  $cfg.crossfade = -not $cfg.crossfade
  Save-Settings -Now
  if (-not $cfg.crossfade) { Stop-Crossfade }
}

function Stop-Media {
  Stop-Crossfade
  Stop-Engines
  $State.Mode = 'stopped'
  $State.Cur = -1
  $State.Pos = -1
  $State.Title = ''
  $State.MediaInfo = ''
  $State.Cover = ''
  $State.IsVideo = $false
  $State.Offset = 0.0
  $State.Tags = $null
  $State.Lyrics = $null
  $State.Duration = 0.0
  $State.MediaEnded = $false
  $State.MediaFailed = $false
}

function Start-Track([int]$index) {
  if (-not $State.FadeStarting) { Stop-Crossfade }
  $State.Radio.On = $false
  $State.Radio.Failed = $false
  $path = $State.Files[$index]
  $ext = [IO.Path]::GetExtension($path).ToLowerInvariant()
  $State.Cur = $index
  $State.Mode = 'playing'
  $State.MediaEnded = $false
  $State.MediaFailed = $false
  $State.Retried = $false
  $State.IsVideo = $AllVideoExtensions -contains $ext
  $State.KnownDuration = if ($ext -eq '.mp3') { Get-Mp3Duration $path } else { 0.0 }
  $State.Duration = $State.KnownDuration
  $State.StartedAt = [DateTime]::UtcNow
  $tags = Get-Tags $path
  $State.Tags = $tags
  $State.MediaInfo = Get-MediaInfo $path $tags
  $State.Cover = if ($State.IsVideo) { '' } else { Get-CoverPath $path $tags }
  $State.Offset = if ($cfg.offsets.ContainsKey($path)) { [double]$cfg.offsets[$path] } else { 0.0 }
  $tags.Picture = $null   # the picture is in its file now
  $State.Title = if ($tags.Title) { $tags.Title } else { [IO.Path]::GetFileNameWithoutExtension($path) }
  $State.Lyrics = Get-Lyrics $path $tags
  # Played again: the online lyrics or subtitles come back (from the cache, or asked again).
  $State.Online.Tried = ''
  $State.Subs.Tried = ''
  try {
    Open-Engine $path (Get-EngineFor $path)
  } catch {
    Write-Log "Cannot open ${path}: $($_.Exception.Message)"
    $State.MediaFailed = $true
  }
  Show-CurrentInList
}

# The name of the file that Previous (-1) or Next (1) would play now, for their tooltips; '' when
# nothing is playing, or when it is not known yet (a new random order is made at the end of a
# shuffled list). Previous restarts a song that has played longer than $Timing.PrevRestartSeconds:
# then it is the file being played.
function Get-StepName([int]$dir, [double]$position) {
  # A radio: the radio before or after it in the list.
  if ($State.Radio.On -and $State.Mode -ne 'stopped') {
    $neighbor = Get-RadioNeighbor $dir
    if (-not $neighbor) { return '' }
    $label = [string]$neighbor.Name
    if ($label.Length -gt 120) { $label = $label.Substring(0, 119) + [string][char]0x2026 }
    return $label
  }
  $n = $State.Files.Count
  if ($State.Mode -eq 'stopped' -or $n -eq 0 -or $State.Cur -lt 0) { return '' }
  if ($dir -lt 0 -and $position -gt $Timing.PrevRestartSeconds) {
    $index = $State.Cur
  } else {
    if ($State.Order.Count -ne $n) { return '' }
    $next = $State.Pos + $dir
    if ($next -ge $n -or $next -lt 0) {
      if ($cfg.shuffle) { return '' }
      $next = if ($dir -gt 0) { 0 } else { $n - 1 }
    }
    $index = $State.Order[$next]
  }
  $name = [IO.Path]::GetFileName($State.Files[$index])
  if ($name.Length -gt 120) { $name = $name.Substring(0, 119) + [string][char]0x2026 }
  $name
}

# A button's tooltip: its name, then on a new line (`\n` in a tooltip) the file it leads to.
function Get-StepTip([string]$name, [int]$dir, [double]$position) {
  $file = Get-StepName $dir $position
  if ($file) { "$name\n$file" } else { $name }
}

# Goes `dir` (1 or -1) songs on in the play order. `auto` = the song ended by itself: then the
# end of the list stops the music unless the list repeats.
function Step([int]$dir, [bool]$auto) {
  $n = $State.Files.Count
  if ($n -eq 0) { return }
  if ($State.Order.Count -ne $n) { New-Order -1 }
  $next = $State.Pos + $dir
  if ($next -ge $n -or $next -lt 0) {
    if ($auto -and $cfg.repeat -ne 'all') { Stop-Media; return }
    if ($cfg.shuffle) {
      New-Order -1
      if ($n -gt 1 -and $State.Order[0] -eq $State.Cur) {
        $swap = $State.Order[0]; $State.Order[0] = $State.Order[1]; $State.Order[1] = $swap
      }
    }
    $next = if ($dir -gt 0) { 0 } else { $n - 1 }
  }
  $State.Pos = $next
  Start-Track $State.Order[$next]
}

function Invoke-Play {
  if ($State.Mode -eq 'playing') { return }
  # A radio is live: Play joins the stream again instead of going on from where it was paused.
  if ($State.Radio.On -and $State.Radio.Station) { Start-Radio $State.Radio.Station; return }
  if ($State.Mode -eq 'paused') { Resume-Engine; $State.Mode = 'playing'; return }
  $n = $State.Files.Count
  if ($n -eq 0) { return }
  New-Order $(if ($cfg.shuffle) { Get-Random -Maximum $n } else { 0 })
  $State.Pos = 0
  $State.Failures = 0
  Start-Track $State.Order[0]
}

function Invoke-Pause {
  if ($State.Mode -ne 'playing') { return }
  Suspend-Engine
  $State.Mode = 'paused'
}

function Invoke-Prev {
  if ($State.Mode -eq 'stopped') { return }
  if ($State.Radio.On) { Start-Radio (Get-RadioNeighbor -1); return }
  # As in most players: a song well under way starts again first.
  if ((Get-EnginePosition) -gt $Timing.PrevRestartSeconds) {
    Set-EnginePosition 0
    return
  }
  Step -1 $false
}

function Invoke-Next {
  if ($State.Mode -eq 'stopped') { return }
  if ($State.Radio.On) { Start-Radio (Get-RadioNeighbor 1); return }
  Step 1 $false
}

function Invoke-Shuffle {
  $cfg.shuffle = -not $cfg.shuffle
  Save-Settings -Now
  if ($State.Cur -ge 0) {
    New-Order $State.Cur
    $State.Pos = if ($cfg.shuffle) { 0 } else { $State.Cur }
  }
}

function Invoke-Repeat {
  $cfg.repeat = switch ($cfg.repeat) { 'off' { 'all' } 'all' { 'one' } default { 'off' } }
  Save-Settings -Now
}

# The volume slider: a position from 0 to 1. It is saved a moment after the last move.
function Set-VolumeFraction([double]$fraction) {
  $cfg.volume = [int][Math]::Round([Math]::Max([double]0, [Math]::Min([double]1, $fraction)) * 100)
  $cfg.muted = $false
  Set-Volume
  Save-Settings
}

function Invoke-Mute {
  $cfg.muted = -not $cfg.muted
  if (-not $cfg.muted -and $cfg.volume -eq 0) { $cfg.volume = 30 }
  Set-Volume
  Save-Settings -Now
}

# The position slider: a position from 0 to 1 in the song.
function Invoke-Seek([double]$fraction) {
  if ($State.Mode -eq 'stopped' -or $State.Duration -le 0) { return }
  $f = [Math]::Max([double]0, [Math]::Min([double]1, $fraction))
  Set-EnginePosition ($f * $State.Duration)
}

# The Playlist tab: the first song shown, moved by pages or by its slider, and kept so that the
# song being played is on the page.
function Get-ListMaxTop { [Math]::Max(0, $State.Files.Count - $Layout.PlaylistRows) }

function Move-ListTop([int]$delta) {
  $State.ListTop = [Math]::Max(0, [Math]::Min((Get-ListMaxTop), $State.ListTop + $delta))
}

function Set-ListFraction([double]$fraction) {
  $f = [Math]::Max([double]0, [Math]::Min([double]1, $fraction))
  $State.ListTop = [int][Math]::Round($f * (Get-ListMaxTop))
}

function Show-CurrentInList {
  if ($State.Cur -lt 0) { return }
  $page = $Layout.PlaylistRows
  if ($State.Cur -lt $State.ListTop -or $State.Cur -ge $State.ListTop + $page) {
    $State.ListTop = [Math]::Max(0, [Math]::Min((Get-ListMaxTop), $State.Cur - [int][Math]::Floor($page / 2)))
  }
}

# Plays the song at `index` of the songs now (a double-click in the Playlist tab).
function Start-FileAt([int]$index) {
  if ($index -lt 0 -or $index -ge $State.Files.Count) { return }
  if ($State.Order.Count -ne $State.Files.Count) { New-Order -1 }
  $pos = [Array]::IndexOf($State.Order, $index)
  if ($pos -lt 0) { return }
  $State.Pos = $pos
  $State.Failures = 0
  Start-Track $index
}

# The shift of the lyrics of the song being played, in seconds: a positive one shows them later.
# Kept for that song, a moment after the last change.
function Set-LyricsOffset([double]$seconds) {
  if ($State.Cur -lt 0) { return }
  $limit = [double]$Layout.SyncMax
  $value = [Math]::Round([Math]::Max(-$limit, [Math]::Min($limit, $seconds)), 3)
  $State.Offset = $value
  $path = $State.Files[$State.Cur]
  if ($value -eq 0) { $cfg.offsets.Remove($path) } else { $cfg.offsets[$path] = $value }
  Save-Settings
}

function Invoke-Online {
  $cfg.online = -not $cfg.online
  Save-Settings -Now
  # Ticked in the middle of a song without lyrics: that song is looked up too.
  $State.Online.Tried = ''
}

# The folder and playlist pickers run in a process of their own: this script goes on (the song,
# the lyrics) while one is open, and the dialog gets a thread of its own to wait in.
function Start-Dialog([string]$kind) {
  if ($State.Dialog) { return }
  $owner = @'
Add-Type -AssemblyName System.Windows.Forms
$owner = New-Object System.Windows.Forms.Form
$owner.TopMost = $true
$owner.ShowInTaskbar = $false
$owner.StartPosition = 'CenterScreen'
$owner.Opacity = 0
$owner.Show()

'@
  $code = if ($kind -eq 'folder') {
    $owner + @'
$dialog = New-Object System.Windows.Forms.FolderBrowserDialog
$dialog.Description = $env:PF_TITLE
$dialog.SelectedPath = $env:PF_START
$dialog.ShowNewFolderButton = $false
if ($dialog.ShowDialog($owner) -eq [System.Windows.Forms.DialogResult]::OK) {
  [Console]::OutputEncoding = [Text.Encoding]::UTF8
  [Console]::Out.Write($dialog.SelectedPath)
}
'@
  } elseif ($kind -eq 'search') {
    # A small window with one line to type in: the name of a radio.
    @'
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
$form.ClientSize = New-Object System.Drawing.Size(380, 96)
$label = New-Object System.Windows.Forms.Label
$label.Text = $env:PF_PROMPT
$label.AutoSize = $true
$label.Location = New-Object System.Drawing.Point(12, 12)
$box = New-Object System.Windows.Forms.TextBox
$box.Location = New-Object System.Drawing.Point(12, 34)
$box.Size = New-Object System.Drawing.Size(356, 24)
$box.MaxLength = 100
$ok = New-Object System.Windows.Forms.Button
$ok.Text = 'OK'
$ok.DialogResult = [System.Windows.Forms.DialogResult]::OK
$ok.Location = New-Object System.Drawing.Point(212, 62)
$cancel = New-Object System.Windows.Forms.Button
$cancel.Text = $env:PF_CANCEL
$cancel.DialogResult = [System.Windows.Forms.DialogResult]::Cancel
$cancel.Location = New-Object System.Drawing.Point(293, 62)
$form.AcceptButton = $ok
$form.CancelButton = $cancel
$form.Controls.AddRange(@($label, $box, $ok, $cancel))
$form.Add_Shown({ $form.Activate(); $box.Focus() })
if ($form.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
  [Console]::OutputEncoding = [Text.Encoding]::UTF8
  [Console]::Out.Write($box.Text)
}
'@
  } else {
    $owner + @'
$dialog = New-Object System.Windows.Forms.OpenFileDialog
$dialog.Title = $env:PF_TITLE
$dialog.Filter = $env:PF_FILTER
$dialog.InitialDirectory = $env:PF_START
$dialog.CheckFileExists = $true
if ($dialog.ShowDialog($owner) -eq [System.Windows.Forms.DialogResult]::OK) {
  [Console]::OutputEncoding = [Text.Encoding]::UTF8
  [Console]::Out.Write($dialog.FileName)
}
'@
  }
  try {
    $psi = New-Object System.Diagnostics.ProcessStartInfo 'powershell.exe'
    $psi.Arguments = '-NoProfile -NonInteractive -STA -ExecutionPolicy Bypass -EncodedCommand ' +
      [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($code))
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $psi.RedirectStandardOutput = $true
    $psi.StandardOutputEncoding = [Text.Encoding]::UTF8
    $start = if ($kind -eq 'folder') { $cfg.folder } elseif ($cfg.playlist) { [IO.Path]::GetDirectoryName($cfg.playlist) } else { $cfg.folder }
    $psi.EnvironmentVariables['PF_TITLE'] = if ($kind -eq 'folder') { $T.chooseTitle } elseif ($kind -eq 'search') { $T.radioSearchTitle } else { $T.playlistTitle }
    $psi.EnvironmentVariables['PF_PROMPT'] = $T.radioSearchPrompt
    $psi.EnvironmentVariables['PF_CANCEL'] = $T.cancel
    $psi.EnvironmentVariables['PF_START'] = $start
    $psi.EnvironmentVariables['PF_FILTER'] = "$($T.playlists) (*.m3u;*.m3u8;*.pls;*.wpl)|*.m3u;*.m3u8;*.pls;*.wpl|*.*|*.*"
    $process = [System.Diagnostics.Process]::Start($psi)
    $State.Dialog = @{ Kind = $kind; Process = $process; Out = $process.StandardOutput.ReadToEndAsync() }
  } catch {
    Write-Log "Dialog: $($_.Exception.Message)"
  }
}

function Complete-Dialog {
  $dialog = $State.Dialog
  if (-not $dialog -or -not $dialog.Out.IsCompleted) { return }
  $chosen = ''
  try { $chosen = $dialog.Out.Result.Trim([char]0xFEFF, "`r", "`n", ' ') } catch { }
  $dialog.Process.Dispose()
  $State.Dialog = $null
  if (-not $chosen) { return }
  if ($dialog.Kind -eq 'search') {
    Start-RadioSearch $chosen
  } elseif ($dialog.Kind -eq 'folder' -and (Test-Path -LiteralPath $chosen -PathType Container)) {
    $cfg.folder = $chosen
    $cfg.playlist = ''
    Save-Settings -Now
    Stop-Media
    $State.Radio.On = $false   # Play now plays the files of the folder
    Start-Scan
  } elseif ($dialog.Kind -eq 'playlist' -and (Test-Path -LiteralPath $chosen -PathType Leaf)) {
    Open-Playlist $chosen
  }
}

# ---- Online lyrics (lrclib.net) -----------------------------------------------------------
# The artist, title and album to look a song up with: its tags, else a file named "Artist - Title".
function Get-LookupNames([string]$path, $tags) {
  $artist = $tags.Artist
  $title = $tags.Title
  if (-not $artist -or -not $title) {
    $name = [IO.Path]::GetFileNameWithoutExtension($path)
    $dash = $name.IndexOf(' - ')
    if ($dash -gt 0) { $artist = $name.Substring(0, $dash).Trim(); $title = $name.Substring($dash + 3).Trim() }
  }
  if (-not $artist -or -not $title) { return $null }
  @{ Artist = $artist; Title = $title; Album = $tags.Album }
}

# The file of the cache folder `folder` that keeps the answer for `key`, named after its hash.
function Get-CacheFile([string]$folder, [string]$key) {
  $sha = [Security.Cryptography.SHA1]::Create()
  $hash = ($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($key.ToLowerInvariant())) | ForEach-Object { $_.ToString('x2') }) -join ''
  $sha.Dispose()
  Join-Path (Join-Path $DataDir $folder) "$hash.json"
}

function Get-LookupCacheFile($names, [double]$duration) {
  Get-CacheFile 'lyrics-cache' ('{0}|{1}|{2}|{3}' -f $names.Artist, $names.Title, $names.Album, [int][Math]::Round($duration))
}

function Write-LookupCache([string]$file, [bool]$found, [string]$text) {
  try {
    New-Item -ItemType Directory -Force -Path (Split-Path $file) | Out-Null
    @{ Found = $found; Text = $text; At = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds() } |
      ConvertTo-Json | Set-Content -LiteralPath $file -Encoding UTF8
  } catch { Write-Log "Cache ${file}: $($_.Exception.Message)" }
}

# Looks up the lyrics of the song being played when the box is ticked, it has none, and
# lrclib.net's conditions allow a request now.
function Update-OnlineLyrics([double]$position) {
  $online = $State.Online
  if ($online.Request) {
    $request = $online.Request
    if (-not $request.Background.Handle.IsCompleted) { return }
    $online.Request = $null
    $result = (Receive-Background $request.Background)[0]
    # The delay between two requests runs from the end of the previous one.
    $online.LastRequestAt = [DateTime]::UtcNow
    Complete-Lookup $request $result
    return
  }
  if (-not $cfg.online -or $State.Mode -eq 'stopped' -or $State.IsVideo -or $State.Lyrics -or $State.Cur -lt 0) { return }
  $path = $State.Files[$State.Cur]
  if ($online.Tried -ceq $path) { return }
  $now = [DateTime]::UtcNow
  if (($now - $State.StartedAt).TotalSeconds -lt $Lrclib.StartAfterSeconds -or $State.Duration -lt 1 -or $State.Duration -gt 3600) { return }
  $names = Get-LookupNames $path $State.Tags
  if (-not $names) { $online.Tried = $path; return }
  $file = Get-LookupCacheFile $names $State.Duration
  if (Test-Path -LiteralPath $file -PathType Leaf) {
    try {
      $cached = Get-Content -LiteralPath $file -Raw -Encoding UTF8 | ConvertFrom-Json
      $age = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds() - [int64]$cached.At
      if ($cached.Found -and $cached.Text) {
        $online.Tried = $path
        $State.Lyrics = New-Lyrics ([string]$cached.Text)
        return
      }
      if ($age -lt $Lrclib.MissRetryDays * 86400) { $online.Tried = $path; return }
    } catch { }
  }
  # One request at a time, spaced, and none while the server has asked us to wait.
  if ($now -lt $online.BlockedUntil -or ($now - $online.LastRequestAt).TotalMilliseconds -lt $Lrclib.MinIntervalMs) { return }
  $online.Tried = $path
  $online.LastRequestAt = $now
  $query = 'artist_name={0}&track_name={1}&duration={2}' -f [Uri]::EscapeDataString($names.Artist),
    [Uri]::EscapeDataString($names.Title), [int][Math]::Round($State.Duration)
  if ($names.Album) { $query += '&album_name=' + [Uri]::EscapeDataString($names.Album) }
  $url = "$($Lrclib.Url)?$query"
  $online.Request = @{
    Background = Start-Background $HttpCode @($url, $Lrclib.UserAgent, $Lrclib.TimeoutMs)
    Path = $path
    File = $file
    Url = $url
  }
}

# How long a Retry-After header asks to wait: seconds, or an HTTP date; `default` seconds when it
# is neither.
function Get-RetrySeconds($header, [double]$default) {
  $seconds = 0.0
  if ([double]::TryParse([string]$header, [Globalization.NumberStyles]::Float,
      [Globalization.CultureInfo]::InvariantCulture, [ref]$seconds) -and $seconds -gt 0) { return $seconds }
  $date = [DateTimeOffset]::MinValue
  if ([DateTimeOffset]::TryParse([string]$header, [Globalization.CultureInfo]::InvariantCulture,
      [Globalization.DateTimeStyles]::AssumeUniversal, [ref]$date)) {
    $seconds = ($date - [DateTimeOffset]::UtcNow).TotalSeconds
    if ($seconds -gt 0) { return $seconds }
  }
  $default
}

# What lrclib.net answered: lyrics (kept), "not found" (kept for a week), a request it refused
# (not asked again for that song), or a reason to wait before any other request.
function Complete-Lookup($request, $result) {
  $online = $State.Online
  $now = [DateTime]::UtcNow
  $status = [int]$result.Status
  if ($status -eq 200 -and [string]$result.ContentType -notmatch 'json') {
    # The docs: look at the status and the Content-Type before taking the body for JSON.
    $online.BlockedUntil = $now.AddSeconds($Lrclib.ErrorBackoffSeconds)
    Write-Log "lrclib.net: an answer that is not JSON ($($result.ContentType))"
  } elseif ($status -eq 200) {
    try {
      $record = $result.Body | ConvertFrom-Json
      $text = if ($record.syncedLyrics) { [string]$record.syncedLyrics } elseif ($record.plainLyrics) { [string]$record.plainLyrics } else { '' }
      Write-LookupCache $request.File ([bool]$text) $text
      if ($text -and $State.Cur -ge 0 -and $State.Files[$State.Cur] -ceq $request.Path -and -not $State.Lyrics) {
        $State.Lyrics = New-Lyrics $text
      }
    } catch { Write-Log "lrclib.net answer: $($_.Exception.Message)" }
  } elseif ($status -eq 404) {
    Write-Log "lrclib.net: no lyrics for $($request.Url)"
    Write-LookupCache $request.File $false ''
  } elseif ($status -eq 400) {
    # A parameter was refused (this song's names, say): the song is not asked again, the others are.
    Write-Log "lrclib.net refused $($request.Url): $($result.Body)"
  } elseif ($status -eq 429 -or $status -eq 503) {
    # Retry-After is honoured, with up to 30 % more so that clients do not return together.
    $seconds = Get-RetrySeconds $result.RetryAfter $Lrclib.DefaultRetrySeconds
    $online.BlockedUntil = $now.AddSeconds($seconds * (1 + 0.3 * (Get-Random -Minimum 0.0 -Maximum 1.0)))
    Write-Log "lrclib.net asks to wait $([int]$seconds) s (status $status)"
  } else {
    $online.BlockedUntil = $now.AddSeconds($Lrclib.ErrorBackoffSeconds)
    Write-Log "lrclib.net: status $status $($result.Body)"
  }
}

# ---- Online subtitles (opensubtitles.com) ------------------------------------------------
# The fingerprint opensubtitles.com finds a video by (its "moviehash"): the size of the file plus
# its first and last 64 KB read as 64-bit little-endian numbers, modulo 2^64, in 16 hex digits.
# '' for a file under 128 KB. The sum is kept in two 32-bit halves: PowerShell has no 64-bit sum
# that wraps around.
function Get-OpenSubtitlesHash([string]$path) {
  try {
    $fs = [IO.File]::Open($path, 'Open', 'Read', 'ReadWrite')
    try {
      $size = $fs.Length
      if ($size -lt 131072) { return '' }
      $head = Read-Bytes $fs 65536
      $fs.Position = $size - 65536
      $tail = Read-Bytes $fs 65536
    } finally { $fs.Dispose() }
    $half = [int64]4294967296
    [int64]$low = $size % $half
    [int64]$high = [Math]::Floor($size / $half)
    foreach ($chunk in $head, $tail) {
      for ($i = 0; $i -lt 65536; $i += 8) {
        $low += [BitConverter]::ToUInt32($chunk, $i)
        $high += [BitConverter]::ToUInt32($chunk, $i + 4)
      }
    }
    $high = ($high + [Math]::Floor($low / $half)) % $half
    $low = $low % $half
    return ('{0:x8}{1:x8}' -f [int64]$high, [int64]$low)
  } catch {
    Write-Log "Fingerprint of ${path}: $($_.Exception.Message)"
    return ''
  }
}

# A value of a search's address as opensubtitles.com asks: lower case, "+" for the spaces.
function ConvertTo-SubtitleParameter([string]$value) {
  [Uri]::EscapeDataString($value.ToLowerInvariant()) -replace '%20', '+'
}

# The subtitle to download among the answer of a search: one made for this very file (its
# fingerprint) first, then in the language wanted first, made by people rather than by a machine,
# and the most downloaded. $null when there is none.
function Select-Subtitle($answer) {
  $choices = foreach ($item in @($answer.data)) {
    $a = $item.attributes
    if (-not $a -or -not $a.files) { continue }
    $id = @($a.files)[0].file_id
    if (-not $id) { continue }
    $rank = [Array]::IndexOf($SubtitleLanguages, ([string]$a.language).ToLowerInvariant())
    [pscustomobject]@{
      Id = [int64]$id
      Match = [bool]$a.moviehash_match
      Rank = $(if ($rank -lt 0) { $SubtitleLanguages.Count } else { $rank })
      Machine = [bool]$a.machine_translated -or [bool]$a.ai_translated
      Downloads = [double]$a.download_count
    }
  }
  @($choices | Sort-Object @{ Expression = 'Match'; Descending = $true }, 'Rank', 'Machine',
    @{ Expression = 'Downloads'; Descending = $true }) | Select-Object -First 1
}

# Sends a request of a lookup: `Api` ones to opensubtitles.com with the API key; the file of a
# download link without it (another server).
function Send-SubtitleRequest($spec) {
  $subs = $State.Subs
  $subs.LastRequestAt = [DateTime]::UtcNow
  $headers = if ($spec.Api) { @{ 'Api-Key' = $OpenSubtitles.ApiKey } } else { $null }
  $accept = if ($spec.Api) { 'application/json' } else { '*/*' }
  $spec.Background = Start-Background $HttpCode @($spec.Url, $OpenSubtitlesApi.UserAgent,
    $OpenSubtitlesApi.TimeoutMs, $accept, $headers, $spec.Method, $spec.Body)
  $subs.Request = $spec
}

# Looks up the subtitles of the video being played when the box is ticked, it has none, a key is
# given, and the service's conditions allow a request now. A lookup is a search by fingerprint
# (then by name when that finds nothing), a download (it counts in the daily quota), and the
# file of its link.
function Update-OnlineSubtitles {
  $subs = $State.Subs
  $now = [DateTime]::UtcNow
  if ($subs.Request) {
    $request = $subs.Request
    if (-not $request.Background.Handle.IsCompleted) { return }
    $subs.Request = $null
    $result = (Receive-Background $request.Background)[0]
    # The delay between two requests runs from the end of the previous one.
    $subs.LastRequestAt = [DateTime]::UtcNow
    Complete-SubtitleStep $request $result
    return
  }
  $ready = $now -ge $subs.BlockedUntil -and ($now - $subs.LastRequestAt).TotalMilliseconds -ge $OpenSubtitlesApi.MinIntervalMs
  if ($subs.Next) {
    if (-not $ready) { return }
    $next = $subs.Next
    $subs.Next = $null
    if ($next.Step -eq 'download' -and $now -lt $subs.QuotaUntil) { $subs.Tried = ''; return }   # again once the quota is back
    Send-SubtitleRequest $next
    return
  }
  if (-not $cfg.subtitles -or -not $OpenSubtitles.ApiKey -or $State.Mode -eq 'stopped' -or -not $State.IsVideo -or
    $State.Lyrics -or $State.Cur -lt 0) { return }
  $path = $State.Files[$State.Cur]
  if ($subs.Tried -ceq $path) { return }
  # Skipping through videos sends nothing (and reads nothing).
  if (($now - $State.StartedAt).TotalSeconds -lt $OpenSubtitlesApi.StartAfterSeconds) { return }
  if ($subs.HashPath -cne $path) {
    $subs.Hash = Get-OpenSubtitlesHash $path
    $subs.HashPath = $path
  }
  $name = ([IO.Path]::GetFileNameWithoutExtension($path) -replace '[._]+', ' ' -replace '\s+', ' ').Trim()
  $languages = (@($SubtitleLanguages | Sort-Object) -join ',')
  $file = Get-CacheFile 'subtitles-cache' ('{0}|{1}|{2}' -f $subs.Hash, $name, $languages)
  if (Test-Path -LiteralPath $file -PathType Leaf) {
    try {
      $cached = Get-Content -LiteralPath $file -Raw -Encoding UTF8 | ConvertFrom-Json
      $age = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds() - [int64]$cached.At
      if ($cached.Found -and $cached.Text) {
        $subs.Tried = $path
        $State.Lyrics = New-Lyrics (ConvertFrom-Srt ([string]$cached.Text))
        return
      }
      if ($age -lt $OpenSubtitlesApi.MissRetryDays * 86400) { $subs.Tried = $path; return }
    } catch { }
  }
  # One request at a time, spaced, none while the server has asked us to wait, and no lookup
  # while the download quota is used up.
  if (-not $ready -or $now -lt $subs.QuotaUntil) { return }
  $subs.Tried = $path
  # The parameters in alphabetical order (languages, moviehash, query), as the service asks.
  $base = "$($OpenSubtitlesApi.Url)/subtitles?languages=$languages"
  $byName = if ($name) { "$base&query=$(ConvertTo-SubtitleParameter $name)" } else { $null }
  $search = @{ Step = 'search'; Api = $true; Method = 'GET'; Path = $path; File = $file }
  if ($subs.Hash) {
    $search.Url = "$base&moviehash=$($subs.Hash)"
    $search.Fallback = $byName
  } elseif ($byName) {
    $search.Url = $byName
  } else {
    return
  }
  Send-SubtitleRequest $search
}

# What opensubtitles.com answered to a step of a lookup. A refused key, a request to wait or a
# used-up quota stop the lookups for that long (the video is looked up again afterwards).
function Complete-SubtitleStep($request, $result) {
  $subs = $State.Subs
  $now = [DateTime]::UtcNow
  $status = [int]$result.Status
  if ($status -eq 401 -or $status -eq 403) {
    $subs.Refused = $true
    $subs.BlockedUntil = $now.AddSeconds($OpenSubtitlesApi.RefusedRetrySeconds)
    $subs.Tried = ''
    Write-Log "opensubtitles.com refused the API key (status $status): $($result.Body)"
    return
  }
  if ($status -eq 429 -or $status -eq 503) {
    # Retry-After is honoured, with up to 30 % more so that clients do not return together.
    $seconds = Get-RetrySeconds $result.RetryAfter $OpenSubtitlesApi.DefaultRetrySeconds
    $subs.BlockedUntil = $now.AddSeconds($seconds * (1 + 0.3 * (Get-Random -Minimum 0.0 -Maximum 1.0)))
    $subs.Tried = ''
    Write-Log "opensubtitles.com asks to wait $([int]$seconds) s (status $status)"
    return
  }
  if ($request.Step -eq 'download' -and $status -eq 406) {
    # The daily download quota is used up: the answer says when it comes back.
    $subs.QuotaUntil = $now.AddSeconds($OpenSubtitlesApi.QuotaRetrySeconds)
    try {
      $reset = [DateTimeOffset]::MinValue
      if ([DateTimeOffset]::TryParse([string]($result.Body | ConvertFrom-Json).reset_time_utc,
          [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::AssumeUniversal, [ref]$reset) -and
        $reset.UtcDateTime -gt $now) { $subs.QuotaUntil = $reset.UtcDateTime }
    } catch { }
    $subs.Tried = ''
    Write-Log "opensubtitles.com: download quota used up until $($subs.QuotaUntil.ToLocalTime()) $($result.Body)"
    return
  }
  if ($status -ne 200) {
    if ($status -eq 0 -or $status -ge 500) {
      $subs.BlockedUntil = $now.AddSeconds($OpenSubtitlesApi.ErrorBackoffSeconds)
      $subs.Tried = ''
    }
    # Another refusal (a request it does not take): that video is not asked again.
    Write-Log "opensubtitles.com ($($request.Step)): status $status $($result.Body)"
    return
  }
  $subs.Refused = $false
  try {
    switch ($request.Step) {
      'search' {
        $best = Select-Subtitle ($result.Body | ConvertFrom-Json)
        if ($best) {
          $subs.Next = @{ Step = 'download'; Api = $true; Method = 'POST'; Url = "$($OpenSubtitlesApi.Url)/download"
            Body = ('{{"file_id":{0},"sub_format":"srt"}}' -f $best.Id); Path = $request.Path; File = $request.File }
        } elseif ($request.Fallback) {
          $subs.Next = @{ Step = 'search'; Api = $true; Method = 'GET'; Url = $request.Fallback; Path = $request.Path; File = $request.File }
        } else {
          Write-Log "opensubtitles.com: no subtitles for $($request.Path)"
          Write-LookupCache $request.File $false ''
        }
      }
      'download' {
        $answer = $result.Body | ConvertFrom-Json
        # The last download of the quota: no other one is asked for until it comes back.
        if ($null -ne $answer.remaining -and [int]$answer.remaining -le 0) {
          $reset = [DateTimeOffset]::MinValue
          $subs.QuotaUntil = if ([DateTimeOffset]::TryParse([string]$answer.reset_time_utc, [Globalization.CultureInfo]::InvariantCulture,
              [Globalization.DateTimeStyles]::AssumeUniversal, [ref]$reset)) { $reset.UtcDateTime } else { $now.AddSeconds($OpenSubtitlesApi.QuotaRetrySeconds) }
        }
        if ($answer.link) {
          $subs.Next = @{ Step = 'file'; Api = $false; Method = 'GET'; Url = [string]$answer.link; Path = $request.Path; File = $request.File }
        } else {
          Write-Log "opensubtitles.com: a download without a link $($result.Body)"
        }
      }
      'file' {
        $text = ConvertFrom-Srt $result.Body
        # Kept, so the subtitle is never downloaded again (the quota counts downloads).
        Write-LookupCache $request.File ([bool]$text) $(if ($text) { $result.Body } else { '' })
        if ($text -and $State.Cur -ge 0 -and $State.Files[$State.Cur] -ceq $request.Path -and -not $State.Lyrics) {
          $State.Lyrics = New-Lyrics $text
        }
      }
    }
  } catch {
    $subs.BlockedUntil = $now.AddSeconds($OpenSubtitlesApi.ErrorBackoffSeconds)
    $subs.Tried = ''
    Write-Log "opensubtitles.com answer ($($request.Step)): $($_.Exception.Message)"
  }
}

function Invoke-Subtitles {
  $cfg.subtitles = -not $cfg.subtitles
  Save-Settings -Now
  # Ticked in the middle of a video without subtitles: that video is looked up too.
  $State.Subs.Tried = ''
}

# ---- What the fence shows -----------------------------------------------------------------
# The line being sung as a row of cells: one cell per word, side by side (they go on to the next
# line when the fence is narrow), the word being sung in bold. The space after a word is a gap;
# the pieces of a word (syllables, characters) touch.
function Get-LyricCells($line, [double]$position) {
  $words = New-Object 'System.Collections.Generic.List[object]'
  $current = -1
  foreach ($chunk in $line.chunks) {
    $text = [string]$chunk.s
    $word = $text.Trim()
    if (-not $word) {
      if ($words.Count -gt 0) { $words[$words.Count - 1].Space = $true }
      continue
    }
    if ([char]::IsWhiteSpace($text[0]) -and $words.Count -gt 0) { $words[$words.Count - 1].Space = $true }
    [void]$words.Add(@{ Text = $word; Space = [char]::IsWhiteSpace($text[$text.Length - 1]) })
    if ($chunk.t -le $position) { $current = $words.Count - 1 }
  }
  # An empty line (an instrumental part), or one too long for a line of 2000 characters.
  if ($words.Count -eq 0) { return Row ([string][char]0x266A) 'color=blue' }
  if ($line.text.Length -gt $Layout.MaxCellsLineText) { return Row $line.text 'color=blue' }
  $cells = for ($i = 0; $i -lt $words.Count; $i++) {
    $style = 'color=blue gap=' + $(if ($words[$i].Space) { $Layout.WordGap } else { 0 })
    if ($i -eq $current) { $style += ' bold=true' }
    Cell $words[$i].Text $style
  }
  ':: ' + ($cells -join ' :: ')
}

# ---- Radios (Radio Browser) ---------------------------------------------------------------
# A request of the directory, on a thread of its own: the list of its servers is read once (any
# of them may be down), then the path is asked of up to three of them. It answers
# @{ Status; Body; Server } (Status 0: no answer). Only servers of the directory's own domain are used.
$RadioFetchCode = {
  param($server, $path, $userAgent, $timeoutMs, $httpText, $serversUrl, $fallback)
  $http = [scriptblock]::Create($httpText)
  $servers = @()
  if ($server) {
    $servers = @($server)
  } else {
    $listed = & $http $serversUrl $userAgent $timeoutMs
    if ($listed.Status -eq 200) {
      try {
        $names = @()
        foreach ($entry in ($listed.Body | ConvertFrom-Json)) {
          if ([string]$entry.name -match '^[a-z0-9-]+\.api\.radio-browser\.info$') { $names += [string]$entry.name }
        }
        $servers = @($names | Select-Object -Unique | Sort-Object { Get-Random })
      } catch { }
    }
    if ($servers.Count -eq 0) { $servers = @($fallback) }
  }
  $answer = @{ Status = 0; Body = 'no server answered'; Server = '' }
  foreach ($name in @($servers | Select-Object -First 3)) {
    $answer = & $http "https://$name$path" $userAgent $timeoutMs
    if ($answer.Status -eq 200) { $answer.Server = $name; return $answer }
  }
  $answer.Server = ''
  $answer
}

# What every list of radios asks for: the radios the directory found alive, the most listened first.
$RadioListPath = "/json/stations/search?hidebroken=true&order=clickcount&reverse=true&limit=$($RadioBrowserApi.Limit)"

function Start-RadioRequest([string]$kind, [string]$path, [string]$label, [string]$source) {
  $r = $State.Radio
  if ($r.Request) { Stop-Background $r.Request.Background }
  $r.Error = $false
  $r.Last = @{ Kind = $kind; Path = $path; Label = $label; Source = $source }
  $r.Request = @{
    Kind = $kind; Label = $label; Source = $source
    Background = Start-Background $RadioFetchCode @($r.Server, $path, $RadioBrowserApi.UserAgent, $RadioBrowserApi.TimeoutMs,
      $HttpCode.ToString(), $RadioBrowserApi.ServersUrl, $RadioBrowserApi.FallbackServer)
  }
}

# The radios the player can play: an http(s) address, MP3 or AAC, not an HLS playlist, found alive.
function Select-RadioStations($data) {
  $found = @()
  foreach ($s in $data) {
    $url = if ($s.url_resolved) { [string]$s.url_resolved } else { [string]$s.url }
    $codec = ([string]$s.codec).Trim().ToUpperInvariant()
    $name = ([string]$s.name).Trim()
    if (-not $name -or $url.Trim() -notmatch '^https?://' -or $RadioCodecs -notcontains $codec) { continue }
    if ([int]$s.hls -ne 0 -or [int]$s.lastcheckok -ne 1) { continue }
    $found += @{
      Uuid = [string]$s.stationuuid; Name = $name; Url = $url.Trim(); CountryCode = [string]$s.countrycode
      Country = [string]$s.country; Codec = $codec; Bitrate = [int]$s.bitrate
    }
  }
  $found
}

# The countries (Value: the two-letter code) or the genres (Value: the tag) of an answer.
function Select-RadioItems($data, [string]$kind) {
  $items = @()
  foreach ($d in $data) {
    $name = ([string]$d.name).Trim()
    $count = [int]$d.stationcount
    if (-not $name -or $name.Length -gt 60 -or $count -le 0) { continue }
    if ($kind -eq 'countries') {
      $code = [string]$d.iso_3166_1
      if ($code -notmatch '^[A-Za-z]{2}$') { continue }
      $items += @{ Name = $name; Value = $code.ToUpperInvariant(); Count = $count }
    } else {
      $items += @{ Name = $name; Value = $name; Count = $count }
    }
  }
  $items
}

function Complete-RadioRequest($request, $result) {
  $r = $State.Radio
  if ([int]$result.Status -ne 200) {
    $r.Server = ''
    $r.Error = $true
    $body = [string]$result.Body
    Write-Log "Radio Browser: status $($result.Status) $($body.Substring(0, [Math]::Min(200, $body.Length)))"
    return
  }
  $r.Server = [string]$result.Server
  $data = @()
  try {
    foreach ($item in ($result.Body | ConvertFrom-Json)) { $data += $item }
  } catch {
    $r.Error = $true
    Write-Log "Radio Browser answer: $($_.Exception.Message)"
    return
  }
  $r.ListTop = 0
  $r.Selected = -1
  $r.Label = $request.Label
  $r.Source = $request.Source
  switch ($request.Kind) {
    'stations' { $r.Stations = @(Select-RadioStations $data); $r.View = 'stations' }
    'countries' { $r.CountryItems = @(Select-RadioItems $data 'countries'); $r.Items = $r.CountryItems; $r.View = 'countries' }
    'genres' { $r.GenreItems = @(Select-RadioItems $data 'genres'); $r.Items = $r.GenreItems; $r.View = 'genres' }
  }
}

# The answer of a request that is done; the request that counts a listen too.
function Update-Radio {
  $r = $State.Radio
  if ($r.CountJob -and $r.CountJob.Handle.IsCompleted) {
    [void](Receive-Background $r.CountJob)
    $r.CountJob = $null
  }
  if (-not $r.Request) { return }
  $request = $r.Request
  if (-not $request.Background.Handle.IsCompleted) { return }
  $r.Request = $null
  $result = (Receive-Background $request.Background)[0]
  Complete-RadioRequest $request $result
}

# The buttons of the tab.
function Show-RadioItems([string]$view, $items, [string]$label) {
  $r = $State.Radio
  if ($r.Request) { Stop-Background $r.Request.Background; $r.Request = $null }
  $r.Error = $false
  $r.Items = $items
  $r.View = $view
  $r.Source = $view
  $r.Label = $label
  $r.ListTop = 0
  $r.Selected = -1
}

function Invoke-RadioPopular { Start-RadioRequest 'stations' $RadioListPath $T.radioPopular 'top' }

function Invoke-RadioCountries {
  if ($State.Radio.CountryItems) { Show-RadioItems 'countries' $State.Radio.CountryItems $T.radioCountries; return }
  Start-RadioRequest 'countries' '/json/countries?hidebroken=true&order=stationcount&reverse=true' $T.radioCountries 'countries'
}

function Invoke-RadioGenres {
  if ($State.Radio.GenreItems) { Show-RadioItems 'genres' $State.Radio.GenreItems $T.radioGenres; return }
  Start-RadioRequest 'genres' '/json/tags?hidebroken=true&order=stationcount&reverse=true&limit=200' $T.radioGenres 'genres'
}

# A click on a country or a genre of the list: its radios.
function Invoke-RadioPick([int]$index) {
  $r = $State.Radio
  if ($r.View -eq 'stations' -or $index -lt 0 -or $index -ge $r.Items.Count) { return }
  $item = $r.Items[$index]
  if ($r.View -eq 'countries') {
    Start-RadioRequest 'stations' "$RadioListPath&countrycode=$($item.Value)" ($T.radioCountryLabel -f $item.Name) 'country'
  } else {
    Start-RadioRequest 'stations' "$RadioListPath&tag=$([Uri]::EscapeDataString($item.Value))&tagExact=true" ($T.radioGenreLabel -f $item.Name) 'genre'
  }
}

function Start-RadioSearch([string]$text) {
  $text = $text.Trim()
  if ($text.Length -gt 100) { $text = $text.Substring(0, 100) }
  if (-not $text) { return }
  Start-RadioRequest 'stations' "$RadioListPath&name=$([Uri]::EscapeDataString($text))" ($T.radioSearchLabel -f $text) 'search'
}

function Invoke-RadioRetry {
  $last = $State.Radio.Last
  if ($last) { Start-RadioRequest $last.Kind $last.Path $last.Label $last.Source }
}

function Show-RadioFavorites {
  $r = $State.Radio
  if ($r.Request) { Stop-Background $r.Request.Background; $r.Request = $null }
  $r.Error = $false
  $r.Stations = @($cfg.radioFavorites)
  $r.View = 'stations'
  $r.Source = 'favorites'
  $r.Label = $T.radioFavorites
  $r.ListTop = 0
  $r.Selected = -1
}

function Test-RadioFavorite($station) {
  foreach ($favorite in $cfg.radioFavorites) { if ($favorite.Url -ceq $station.Url) { return $true } }
  $false
}

# Adds the radio to the favorites, or takes it out of them.
function Invoke-RadioFavorite($station) {
  if (-not $station) { return }
  if (Test-RadioFavorite $station) {
    $cfg.radioFavorites = @($cfg.radioFavorites | Where-Object { $_.Url -cne $station.Url })
  } elseif ($cfg.radioFavorites.Count -lt $RadioBrowserApi.MaxFavorites) {
    $cfg.radioFavorites += @{
      Uuid = [string]$station.Uuid; Name = [string]$station.Name; Url = [string]$station.Url
      CountryCode = [string]$station.CountryCode; Country = [string]$station.Country
      Codec = [string]$station.Codec; Bitrate = [int]$station.Bitrate
    }
  }
  Save-Settings -Now
  $r = $State.Radio
  if ($r.Source -eq 'favorites' -and $r.View -eq 'stations') {
    $r.Stations = @($cfg.radioFavorites)
    if ($r.Selected -ge $r.Stations.Count) { $r.Selected = -1 }
  }
}

function Get-RadioSelected {
  $r = $State.Radio
  if ($r.View -eq 'stations' -and $r.Selected -ge 0 -and $r.Selected -lt $r.Stations.Count) { $r.Stations[$r.Selected] } else { $null }
}

# The Radios tab lists the radios, the countries or the genres a page at a time, as the Playlist tab does.
function Get-RadioListCount {
  $r = $State.Radio
  if ($r.View -eq 'stations') { $r.Stations.Count } else { $r.Items.Count }
}

function Get-RadioMaxTop { [Math]::Max(0, (Get-RadioListCount) - $Layout.RadioRows) }

function Move-RadioTop([int]$delta) {
  $State.Radio.ListTop = [Math]::Max(0, [Math]::Min((Get-RadioMaxTop), $State.Radio.ListTop + $delta))
}

function Set-RadioFraction([double]$fraction) {
  $f = [Math]::Max([double]0, [Math]::Min([double]1, $fraction))
  $State.Radio.ListTop = [int][Math]::Round($f * (Get-RadioMaxTop))
}

# Plays a radio: its stream is opened by Windows' media player of Store apps, which reads the live
# streams of Icecast and SHOUTcast servers (what WPF's player is less good at). The radio replaces
# what played; Previous and Next then go to the radio before and after it in the list.
function Start-Radio($station) {
  if (-not $station -or [string]$station.Url -notmatch '^https?://') { return }
  $r = $State.Radio
  Stop-Media
  # Chosen from the list on show: Previous and Next go along it, even after another list is shown.
  foreach ($listed in $r.Stations) {
    if ($listed.Url -ceq $station.Url) { $r.Queue = @($r.Stations); break }
  }
  $r.On = $true
  $r.Station = $station
  $r.Failed = $false
  $r.Heard = $false
  $r.Reconnects = 0
  $r.StalledAt = $null
  $State.Mode = 'playing'
  $State.Failures = 0
  $State.Title = [string]$station.Name
  $details = @($station.Codec, $(if ($station.Bitrate -gt 0) { "$($station.Bitrate) kbps" }), $station.Country) | Where-Object { $_ }
  $State.MediaInfo = $details -join ', '
  $State.StartedAt = [DateTime]::UtcNow
  try {
    Open-Engine ([string]$station.Url) 'winrt'
  } catch {
    Write-Log "Cannot open a radio: $($_.Exception.Message)"
    Stop-RadioFailed
    return
  }
  if ($CountListens -and [string]$station.Uuid -match '^[0-9a-fA-F-]+$' -and $r.Server -and -not $r.CountJob) {
    $r.CountJob = Start-Background $HttpCode @("https://$($r.Server)/json/url/$($station.Uuid)", $RadioBrowserApi.UserAgent, $RadioBrowserApi.TimeoutMs)
  }
}

# The radio cannot be played: stopped, and the hint at the bottom of the player says so. Play tries it again.
function Stop-RadioFailed {
  Write-Log "Cannot play a radio ($($State.Radio.Station.Url))"
  Stop-Media
  $State.Radio.Failed = $true
}

# The radio before (-1) or after (1) the one being played in the list it came from; $null when it
# is not in the list or alone in it.
function Get-RadioNeighbor([int]$dir) {
  $r = $State.Radio
  $list = @($r.Queue)
  if (-not $r.Station -or $list.Count -lt 2) { return $null }
  for ($i = 0; $i -lt $list.Count; $i++) {
    if ($list[$i].Url -ceq $r.Station.Url) { return $list[($i + $dir + $list.Count) % $list.Count] }
  }
  $null
}

# What happens while a radio plays: the time it has played, a stream that never starts is given up,
# one that was playing and is silent is opened again (up to $Timing.RadioReconnects times).
function Update-RadioPlayback {
  $r = $State.Radio
  if (-not $script:rt) { return 0.0 }
  # Paused: the time it played goes on showing.
  if ($State.Mode -ne 'playing') { return [Math]::Max(0.0, $script:rt.PlaybackSession.Position.TotalSeconds) }
  $session = $script:rt.PlaybackSession
  $playback = [string]$session.PlaybackState
  $now = [DateTime]::UtcNow
  $elapsed = ($now - $State.StartedAt).TotalSeconds
  if ($playback -eq 'Playing') {
    $r.Heard = $true
    $r.StalledAt = $null
    if ($elapsed -gt 30) { $r.Reconnects = 0 }
  } elseif ($r.Heard -and ($playback -eq 'Paused' -or $playback -eq 'None') -and $null -eq $r.StalledAt) {
    $r.StalledAt = $now
  }
  $opened = $State.RtSource
  $broken = ($opened -and [string]$opened.State -eq 'Failed') -or (-not $r.Heard -and $elapsed -gt $Timing.RadioOpenSeconds) -or
    ($null -ne $r.StalledAt -and ($now - $r.StalledAt).TotalSeconds -ge $Timing.RadioStallSeconds)
  if (-not $broken) { return [Math]::Max(0.0, $session.Position.TotalSeconds) }
  if ($r.Heard -and $r.Reconnects -lt $Timing.RadioReconnects) {
    $r.Reconnects++
    Write-Log "Radio: the stream was cut, opening it again ($($r.Reconnects))"
    $r.Heard = $false
    $r.StalledAt = $null
    $State.StartedAt = $now
    try { Open-Engine ([string]$r.Station.Url) 'winrt' } catch { Stop-RadioFailed }
  } else {
    Stop-RadioFailed
  }
  0.0
}

# The rows of the Radios tab: the buttons that choose the list, what the list is, then a page of it.
function Add-RadioRows($rows) {
  $r = $State.Radio
  $on = ' checked=true'
  $source = $r.Source
  $button = { param($emoji, $tip, $action, $active) Cell (EmojiText $emoji) "action=$action tooltip=""$tip""$(if ($active) { $on })" }
  $rows.Add(':: ' + (@(
        (& $button $Emoji.Popular $T.radioPopular 'rpopular' ($source -eq 'top')),
        (& $button $Emoji.Globe $T.radioCountries 'rcountries' ($source -eq 'countries' -or $source -eq 'country')),
        (& $button $Emoji.Genre $T.radioGenres 'rgenres' ($source -eq 'genres' -or $source -eq 'genre')),
        (& $button $Emoji.Star $T.radioFavorites 'rfavorites' ($source -eq 'favorites')),
        (& $button $Emoji.Browse $T.radioSearch 'rsearch' ($source -eq 'search'))) -join ' :: '))
  if ($r.Label) { $rows.Add((Row $r.Label "icon=$(EmojiText $Emoji.Radio) color=gray")) }
  if ($State.Dialog) { $rows.Add((Row $T.choosing 'color=gray')) }
  if ($r.Request) { $rows.Add((Row '...' 'color=gray')); return }
  if ($r.Error) { $rows.Add((Row $T.radioError "icon=$(EmojiText $Emoji.Info) color=orange action=rretry")); return }
  if (-not $r.Source) {
    $rows.Add((Row $T.radioStart 'color=gray'))
    return
  }
  $n = Get-RadioListCount
  if ($n -eq 0) {
    $rows.Add((Row $(if ($r.Source -eq 'favorites') { $T.radioNoFavorites } else { $T.radioNone }) 'color=gray'))
    return
  }
  # What a selected radio can be asked: play, add to the favorites or take out of them.
  $selected = Get-RadioSelected
  if ($selected) {
    $favorite = Test-RadioFavorite $selected
    $rows.Add(':: ' + (@(
          (Cell $T.play 'action=rplaysel'),
          (Cell "$(EmojiText $Emoji.Star) $(if ($favorite) { $T.radioFavRemove } else { $T.radioFavAdd })" "action=rfavsel$(if ($favorite) { $on })")) -join ' :: '))
  }
  $page = $Layout.RadioRows
  $maxTop = Get-RadioMaxTop
  $top = [Math]::Max(0, [Math]::Min($r.ListTop, $maxTop))
  $last = [Math]::Min($n, $top + $page)
  $fraction = if ($maxTop -gt 0) { $top / $maxTop } else { 0.0 }
  $rows.Add(':: ' + (@(
        (Cell (Icon $Glyph.PageUp) "action=rlistup tooltip=""$($T.pageUp)"""),
        (Cell (Icon $Glyph.PageDown) "action=rlistdown tooltip=""$($T.pageDown)"""),
        (Cell '' "slider=$(Format-Fraction $fraction) action=rlistpos tooltip=""$($T.scroll)"""),
        (Cell ('{0}-{1} / {2}' -f ($top + 1), $last, $n) 'color=gray width=90 align=right')) -join ' :: '))
  # Above this row everything stays in place; the page scrolls under it, back from its top for
  # another page or another list.
  $label = ([string]$r.Label -replace '[^\w]', '')
  if ($label.Length -gt 40) { $label = $label.Substring(0, 40) }
  $rows.Add((Row '' "pane=radios:$($r.Source):$($r.View):${top}:$label hidden=true"))
  for ($i = $top; $i -lt $last; $i++) {
    if ($r.View -eq 'stations') {
      $station = $r.Stations[$i]
      $playing = $r.On -and $State.Mode -ne 'stopped' -and $r.Station -and $r.Station.Url -ceq $station.Url
      $symbol = if ($playing) { $Mark.Playing } elseif ($i -eq $r.Selected) { $Mark.Selected } else { $Mark.Other }
      $detail = @($station.CountryCode, $(if ($station.Bitrate -gt 0) { "$($station.Bitrate) kbps" })) | Where-Object { $_ }
      $text = '{0}. {1}{2}' -f ($i + 1), $station.Name, $(if ($detail) { "  ($($detail -join ', '))" } else { '' })
      $rows.Add((Row $text "action=rsel:$i dblaction=rplay:$i icon=$([char]$symbol)$(if ($playing) { ' color=blue' })"))
    } else {
      $item = $r.Items[$i]
      $icon = EmojiText $(if ($r.View -eq 'countries') { $Emoji.Globe } else { $Emoji.Genre })
      $rows.Add((Row ('{0} ({1})' -f $item.Name, $item.Count) "action=rpick:$i icon=$icon"))
    }
  }
}

# The Playlist tab: a row of paging buttons, a slider and the position, then a page of songs.
# A click selects a song (its mark), a double-click plays it (dblaction=).
function Add-PlaylistRows($rows) {
  $n = $State.Files.Count
  if ($State.Loading) { $rows.Add((Row '...' 'color=gray')); return }
  if ($n -eq 0) { $rows.Add((Row $T.noSongs 'color=gray')); return }
  $page = $Layout.PlaylistRows
  $maxTop = Get-ListMaxTop
  $top = [Math]::Max(0, [Math]::Min($State.ListTop, $maxTop))
  $last = [Math]::Min($n, $top + $page)
  $fraction = if ($maxTop -gt 0) { $top / $maxTop } else { 0.0 }
  $rows.Add(':: ' + (@(
        (Cell (Icon $Glyph.PageUp) "action=listup tooltip=""$($T.pageUp)"""),
        (Cell (Icon $Glyph.PageDown) "action=listdown tooltip=""$($T.pageDown)"""),
        (Cell '' "slider=$(Format-Fraction $fraction) action=listpos tooltip=""$($T.scroll)"""),
        (Cell ('{0}-{1} / {2}' -f ($top + 1), $last, $n) 'color=gray width=90 align=right')) -join ' :: '))
  # Above this row everything stays in place; the page scrolls under it, back from its top for
  # another page.
  $rows.Add((Row '' "pane=playlist:${top}:$n hidden=true"))
  for ($i = $top; $i -lt $last; $i++) {
    $name = [IO.Path]::GetFileName($State.Files[$i])
    # (a variable called $mark would be $Mark: PowerShell names are not case sensitive)
    $symbol = if ($i -eq $State.Cur) { $Mark.Playing } elseif ($i -eq $State.Selected) { $Mark.Selected } else { $Mark.Other }
    $style = "action=sel:$i dblaction=playitem:$i icon=$([char]$symbol)$(if ($i -eq $State.Cur) { ' color=blue' })"
    $rows.Add((Row ('{0}. {1}' -f ($i + 1), $name) $style))
  }
}

# What PecoFence's player is told of the video, as parameters of a row: the file, play or pause,
# volume, speed, and a move (made once for each new seekid).
function Get-VideoParams {
  $n = $State.Native
  $seek = if ($null -ne $n.Seek) { " seek=$(Format-Fraction $n.Seek) seekid=$($n.SeekId)" } else { '' }
  "video=""$($n.Path)"" play=$(if ($n.Playing) { 'true' } else { 'false' }) volume=$(Format-Fraction $n.Volume) speed=$(Format-Fraction $n.Speed)$seek"
}

# While something plays (or is paused) the keyboard's media keys are this player's: PecoFence sends
# "mediakey:playpause" and the like. When it is stopped they go back to the other programs.
function Get-MediaKeysParam {
  if ($MediaKeys -and $State.Mode -ne 'stopped') { ' mediakeys=true' } else { '' }
}

function Get-Screen($position) {
  $rows = New-Object System.Collections.Generic.List[string]
  # The tabs.
  $on = ' checked=true'
  $rows.Add(':: ' + (@(
        (Cell $T.tabPlayback "action=tabplayback$(if ($State.Tab -eq 'playback') { $on })"),
        (Cell $T.tabPlaylist "action=tabplaylist$(if ($State.Tab -eq 'playlist') { $on })"),
        (Cell $T.tabRadios "action=tabradios$(if ($State.Tab -eq 'radios') { $on })")) -join ' :: '))
  $rows.Add('---')
  if ($State.Tab -eq 'playlist' -or $State.Tab -eq 'radios') {
    if ($State.Tab -eq 'radios') { Add-RadioRows $rows } else { Add-PlaylistRows $rows }
    # Out of sight, a video plays on (its sound, its place) and the media keys stay this player's:
    # a hidden row carries them.
    $hidden = "$(if ($State.Mode -ne 'stopped' -and $State.Native) { Get-VideoParams })$(Get-MediaKeysParam)".Trim()
    if ($hidden) { $rows.Add((Row '' "$hidden hidden=true")) }
    return $rows.ToArray()
  }
  # The folder (or the playlist) of the songs; a click opens its folder.
  $source = if ($cfg.playlist) { $cfg.playlist } else { $cfg.folder }
  $sourceDir = if ($cfg.playlist) { [IO.Path]::GetDirectoryName($cfg.playlist) } else { $cfg.folder }
  $sourceIcon = EmojiText $(if ($cfg.playlist) { $Emoji.Playlist } else { $Emoji.Folder })
  # A line is read up to 300 characters: a long path cannot also be written as the action.
  $open = if ($source.Length + $sourceDir.Length -lt 200) { " open=""$($sourceDir -replace '"', "'")""" } else { '' }
  $rows.Add((Row $source "icon=$sourceIcon$open tooltip=""$($T.open)"""))
  if ($State.Dialog) {
    $rows.Add((Row $T.choosing 'color=gray'))
  } else {
    $rows.Add(':: ' + (@(
          (Cell "$(EmojiText $Emoji.Browse) $($T.browse)" 'action=browse'),
          (Cell "$(EmojiText $Emoji.Playlist) $($T.playlist)" 'action=playlist')) -join ' :: '))
  }

  # The tags of the song, framed by two lines; nothing at all when it has none.
  $playing = $State.Mode -ne 'stopped'
  $tags = if ($playing) { $State.Tags } else { $null }
  $tagRows = @()
  if ($tags) {
    if ($tags.Artist) { $tagRows += Row $tags.Artist "icon=$(EmojiText $Emoji.Artist)" }
    if ($tags.Album) { $tagRows += Row $(if ($tags.Year) { "$($tags.Album) ($($tags.Year))" } else { $tags.Album }) "icon=$(EmojiText $Emoji.Album)" }
    elseif ($tags.Year) { $tagRows += Row $tags.Year "icon=$(EmojiText $Emoji.Album)" }
    $detail = @($tags.Genre, $(if ($tags.Track) { "#$($tags.Track)" })) | Where-Object { $_ }
    if ($detail) { $tagRows += Row ($detail -join '  ·  ') "icon=$(EmojiText $(if ($tags.Genre) { $Emoji.Genre } else { $Emoji.Tracks }))" }
  }
  if ($tagRows) {
    $rows.Add('---')
    foreach ($row in $tagRows) { $rows.Add($row) }
    $rows.Add('---')
  }

  # The cover of a song, or the picture of a video, centred above the title (an image row).
  if ($playing -and $State.Native) {
    # PecoFence plays the video in this row; a click on it plays or pauses.
    $rows.Add((Row '' "$(Get-VideoParams) height=$($Layout.VideoHeight) action=videotoggle"))
  } elseif ($playing -and $State.Cover) {
    $rows.Add((Row '' "image=""$($State.Cover)"" height=$($Layout.CoverHeight)"))
  }
  $title = if (-not $playing -or -not $State.Title) { '-' } else { $State.Title }
  $keys = Get-MediaKeysParam
  $radio = $playing -and $State.Radio.On
  $titleEmoji = if ($radio) { $Emoji.Radio } elseif ($playing -and $State.IsVideo) { $Emoji.Video } else { $Emoji.Music }
  $rows.Add((Row $title "icon=$(EmojiText $titleEmoji)$keys"))
  if ($playing -and $State.MediaInfo) { $rows.Add((Row $State.MediaInfo "icon=$(EmojiText $Emoji.Audio) color=gray")) }

  # Time, a slider to move in the song, duration.
  $fraction = if ($State.Duration -gt 0) { [Math]::Min(1.0, $position / $State.Duration) } else { 0.0 }
  $rows.Add(':: ' + (@(
        (Cell (Format-Time $position) "color=gray $($CellStyle.Left)"),
        (Cell '' "slider=$(Format-Fraction $fraction) action=seek tooltip=""$($T.seek)"""),
        (Cell $(if ($radio) { $T.radioLive } else { Format-Time $State.Duration }) "color=gray $($CellStyle.Right)")) -join ' :: '))
  $rows.Add('---')

  # The playback buttons, side by side: icons only, their names are the tooltips. A button
  # that is on (playing, shuffle...) is drawn pressed in.
  $repeatIcon = if ($cfg.repeat -eq 'one') { $Glyph.RepeatOne } else { $Glyph.RepeatAll }
  $repeatTip = switch ($cfg.repeat) { 'all' { $T.repeatAll } 'one' { $T.repeatOne } default { $T.repeatOff } }
  $repeatStyle = if ($cfg.repeat -ne 'off') { $on } else { ' color=gray' }
  $shuffleTip = if ($cfg.shuffle) { $T.shuffleOn } else { $T.shuffleOff }
  $shuffleStyle = if ($cfg.shuffle) { $on } else { ' color=gray' }
  $crossfadeTip = if ($cfg.crossfade) { $T.crossfadeOn -f $CrossfadeSeconds } else { $T.crossfadeOff }
  $crossfadeStyle = if ($cfg.crossfade) { $on } else { ' color=gray' }
  $speedText = [string]::Format([Globalization.CultureInfo]::InvariantCulture, '{0:0.00}', $State.Speed)
  $prevTip = Get-StepTip $T.prev -1 $position
  $nextTip = Get-StepTip $T.next 1 $position
  $rows.Add(':: ' + (@(
        (Cell (Icon $Glyph.Prev) "action=prev tooltip=""$prevTip"""),
        (Cell (Icon $Glyph.Play) "action=play tooltip=""$($T.play)""$(if ($State.Mode -eq 'playing') { $on })"),
        (Cell (Icon $Glyph.Pause) "action=pause tooltip=""$($T.pause)""$(if ($State.Mode -eq 'paused') { $on })"),
        (Cell (Icon $Glyph.Stop) "action=stop tooltip=""$($T.stop)"""),
        (Cell (Icon $Glyph.Next) "action=next tooltip=""$nextTip"""),
        (Cell (Icon $repeatIcon) "action=repeat tooltip=""$repeatTip""$repeatStyle"),
        (Cell (Icon $Glyph.Shuffle) "action=shuffle tooltip=""$shuffleTip""$shuffleStyle"),
        (Cell (Icon $Glyph.Crossfade) "action=crossfade tooltip=""$crossfadeTip""$crossfadeStyle"),
        (Cell $speedText "action=speedreset tooltip=""$($T.speedTip)"" color=gray width=44 align=center"),
        (Cell '<' "action=speeddown tooltip=""$($T.speedSlower)"""),
        (Cell '>' "action=speedup tooltip=""$($T.speedFaster)""")) -join ' :: '))

  # One volume icon (a click mutes), the volume slider and its percentage; the fixed widths
  # put both sliders under each other with the same length.
  $level = if ($cfg.muted) { 0.0 } else { $cfg.volume / 100.0 }
  $percent = [int][Math]::Round($level * 100)
  $volumeIcon = if ($cfg.muted -or $cfg.volume -eq 0) { $Glyph.Mute } else { $Glyph.Volume }
  $muteTip = if ($cfg.muted) { $T.unmute } else { $T.mute }
  $rows.Add(':: ' + (@(
        (Cell (Icon $volumeIcon) "action=mute tooltip=""$muteTip"" $($CellStyle.Left)$(if ($cfg.muted) { $on })"),
        (Cell '' "slider=$(Format-Fraction $level) action=volume tooltip=""$($T.volume) $percent%"""),
        (Cell "$percent%" "color=gray $($CellStyle.Right)")) -join ' :: '))
  $rows.Add('---')

  # A radio has no lyrics or subtitles: neither the boxes nor the shift are shown.
  if (-not $radio) {
    # The boxes of the online lyrics and of the online subtitles, side by side.
    $rows.Add(':: ' + (@(
          (Cell $T.online "checkbox=$(if ($cfg.online) { 'true' } else { 'false' }) action=online tooltip=""$($T.onlineTip)"""),
          (Cell $T.onlineSubs "checkbox=$(if ($cfg.subtitles) { 'true' } else { 'false' }) action=subtitles tooltip=""$($T.onlineSubsTip)""")) -join ' :: '))
    # Under them, a spin box for the shift of the lyrics or subtitles: its value (a click sets it to
    # 0) and the buttons that raise and lower it.
    $rows.Add(':: ' + (@(
          (Cell $T.syncLabel 'color=gray'),
          (Cell ('{0:0.000} s' -f $State.Offset) "action=syncreset tooltip=""$($T.syncReset)"" $($CellStyle.Value)"),
          (Cell '<' "action=syncearlier tooltip=""$($T.syncEarlier)"""),
          (Cell '>' "action=synclater tooltip=""$($T.syncLater)""")) -join ' :: '))
  }

  # Above this row everything stays in place; the lyrics scroll under it (the fence is told which
  # line is being sung, below), back from their top for another song.
  $rows.Add((Row '' "pane=playback:$($State.Cur) hidden=true"))
  if ($radio) {
    # Until the stream is heard: that it is being opened.
    if (-not $State.Radio.Heard) { $rows.Add((Row $T.radioConnecting 'color=gray')) }
  } elseif ($State.Loading) {
    $rows.Add((Row '...' 'color=gray'))
  } elseif ($State.Files.Count -eq 0) {
    $rows.Add((Row $T.noSongs 'color=gray'))
  } elseif (-not $playing) {
    $rows.Add((Row $T.pressPlay 'color=gray'))
  } elseif ($null -eq $State.Lyrics -or $State.Lyrics.Lines.Count -eq 0) {
    $rows.Add((Row $(if ($State.IsVideo) { $T.noSubtitles } else { $T.noLyrics }) 'color=gray'))
  } elseif (-not $State.Lyrics.Timed) {
    $rows.Add((Row '...' 'color=gray'))
  } else {
    $lines = $State.Lyrics.Lines
    # The shift of the lyrics moves the song's position back along them (a delay).
    $lyricPosition = $position - $State.Offset
    $current = -1
    for ($k = 0; $k -lt $lines.Count; $k++) {
      if ($lines[$k].t -le $lyricPosition) { $current = $k } else { break }
    }
    # All the lines (a song that has more than $LyricsBefore + $LyricsAfter keeps those around the
    # line being sung): the fence scrolls them, and keeps the line being sung in the middle.
    $first = [Math]::Max(0, $current - $LyricsBefore)
    $lastLine = [Math]::Min($lines.Count - 1, [Math]::Max($current, 0) + $LyricsAfter)
    for ($k = $first; $k -le $lastLine; $k++) {
      $line = $lines[$k]
      if ($k -lt $current) {
        $rows.Add((Row $line.text 'color=gray'))
      } elseif ($k -eq $current) {
        # The key changes with the line: the row after this one is brought to the middle.
        $rows.Add((Row '' "follow=$k hidden=true"))
        $rows.Add((Get-LyricCells $line $lyricPosition))
      } else {
        $rows.Add((Row $line.text))
      }
    }
    if (-not $State.Lyrics.Synced) {
      $rows.Add((Row $T.approx 'color=gray'))
    }
  }

  # At the bottom: files that Windows cannot play without a Store extension (a click opens the page
  # of the extension in the Microsoft Store), and what keeps the online subtitles from coming.
  $hints = New-Object System.Collections.Generic.List[string]
  $info = "icon=$(EmojiText $Emoji.Info) color=orange"
  foreach ($key in $StorePackages.Keys) {
    if (-not $State.Needs.ContainsKey($key) -or $State.Needs[$key].Count -eq 0) { continue }
    $package = $StorePackages[$key]
    $hints.Add((Row ($T.hintMissing -f $State.Needs[$key].Count, $package.Title) "$info href=ms-windows-store://pdp/?ProductId=$($package.Id)"))
  }
  if ($State.Radio.Failed) { $hints.Add((Row $T.radioFailed $info)) }
  if ($cfg.subtitles) {
    if (-not $OpenSubtitles.ApiKey) { $hints.Add((Row $T.subsKey "$info href=$($OpenSubtitlesApi.KeyPage)")) }
    elseif ($State.Subs.Refused) { $hints.Add((Row $T.subsRefused "$info href=$($OpenSubtitlesApi.KeyPage)")) }
    if ([DateTime]::UtcNow -lt $State.Subs.QuotaUntil) { $hints.Add((Row ($T.subsQuota -f $State.Subs.QuotaUntil.ToLocalTime().ToString('t')) $info)) }
  }
  if ($hints.Count -gt 0) {
    $rows.Add('---')
    foreach ($hint in $hints) { $rows.Add($hint) }
  }
  $rows.ToArray()
}

# ---- Clicks -------------------------------------------------------------------------------
# What each action of a row does; a slider's position comes as the argument.
$Actions = @{
  play = { Invoke-Play }
  pause = { Invoke-Pause }
  videotoggle = { Invoke-PlayPause }
  stop = { Stop-Media }
  prev = { Invoke-Prev }
  next = { Invoke-Next }
  repeat = { Invoke-Repeat }
  shuffle = { Invoke-Shuffle }
  crossfade = { Invoke-Crossfade }
  speeddown = { Set-Speed ($State.Speed - $Layout.SpeedStep) }
  speedup = { Set-Speed ($State.Speed + $Layout.SpeedStep) }
  speedreset = { Set-Speed 1.0 }
  mute = { Invoke-Mute }
  volume = { param($value) Set-VolumeFraction $value }
  seek = { param($value) Invoke-Seek $value }
  browse = { Start-Dialog 'folder' }
  playlist = { Start-Dialog 'playlist' }
  online = { Invoke-Online }
  subtitles = { Invoke-Subtitles }
  syncearlier = { Set-LyricsOffset ($State.Offset - $Layout.SyncStep) }
  synclater = { Set-LyricsOffset ($State.Offset + $Layout.SyncStep) }
  syncreset = { Set-LyricsOffset 0 }
  tabplayback = { $State.Tab = 'playback' }
  tabplaylist = { $State.Tab = 'playlist'; Show-CurrentInList }
  tabradios = { $State.Tab = 'radios' }
  rpopular = { Invoke-RadioPopular }
  rcountries = { Invoke-RadioCountries }
  rgenres = { Invoke-RadioGenres }
  rfavorites = { Show-RadioFavorites }
  rsearch = { Start-Dialog 'search' }
  rretry = { Invoke-RadioRetry }
  rlistup = { Move-RadioTop (-$Layout.RadioRows) }
  rlistdown = { Move-RadioTop $Layout.RadioRows }
  rlistpos = { param($value) Set-RadioFraction $value }
  rsel = { param($value) $State.Radio.Selected = [int]$value }
  rpick = { param($value) Invoke-RadioPick ([int]$value) }
  rplay = { param($value) $r = $State.Radio; $i = [int]$value; if ($r.View -eq 'stations' -and $i -ge 0 -and $i -lt $r.Stations.Count) { Start-Radio $r.Stations[$i]; $State.Tab = 'playback' } }
  rplaysel = { $picked = Get-RadioSelected; if ($picked) { Start-Radio $picked; $State.Tab = 'playback' } }
  rfavsel = { Invoke-RadioFavorite (Get-RadioSelected) }
  listup = { Move-ListTop (-$Layout.PlaylistRows) }
  listdown = { Move-ListTop $Layout.PlaylistRows }
  listpos = { param($value) Set-ListFraction $value }
  sel = { param($value) $State.Selected = [int]$value }
  playitem = { param($value) Start-FileAt ([int]$value); $State.Tab = 'playback' }
}

# A click on the picture of a video, and the play/pause media key: pauses what plays, plays what is
# paused (nothing when stopped).
function Invoke-PlayPause {
  if ($State.Mode -eq 'playing') { Invoke-Pause } elseif ($State.Mode -eq 'paused') { Invoke-Play }
}

# The keyboard's media keys (play/pause, next, previous, stop), as PecoFence reports them.
function Invoke-MediaKey([string]$name) {
  switch ($name) {
    'playpause' { Invoke-PlayPause }
    'next' { Invoke-Next }
    'prev' { Invoke-Prev }
    'stop' { Stop-Media }
  }
}

# The lines the clicks sent: `action` or `action:position`.
function Invoke-ActionLine([string]$line) {
  # PecoFence's own lines about the video it plays come the same way.
  if ($line.StartsWith('video:')) { Receive-VideoLine $line; return }
  if ($line.StartsWith('mediakey:')) { Invoke-MediaKey $line.Substring(9); return }
  $value = 0.0
  $colon = $line.IndexOf(':')
  if ($colon -gt 0) {
    [void][double]::TryParse($line.Substring($colon + 1), [Globalization.NumberStyles]::Float,
      [Globalization.CultureInfo]::InvariantCulture, [ref]$value)
    $line = $line.Substring(0, $colon)
  }
  $handler = $Actions[$line]
  if ($handler) { & $handler $value }
}

# ---- What happens between two redraws -----------------------------------------------------
# The background work that is done, and the settings that wait to be saved.
function Update-Background {
  if ($State.Scan -and $State.Scan.Handle.IsCompleted) {
    $songs = Receive-Background $State.Scan
    $State.Scan = $null
    $State.Candidates = @($songs | ForEach-Object { [string]$_ })
  }
  if ($State.Codec -and $State.Codec.Handle.IsCompleted) {
    $answer = Receive-Background $State.Codec
    $State.Codec = $null
    $installed = @{}
    foreach ($key in $answer) { if ($key) { $installed[[string]$key] = $true } }
    $now = (@($StorePackages.Keys | Where-Object { $installed[$_] }) -join ',')
    $before = if ($null -eq $State.Installed) { $null } else { (@($StorePackages.Keys | Where-Object { $State.Installed[$_] }) -join ',') }
    if ($now -ne $before) {
      # Known for the first time, or an extension was installed since: the files that were left out
      # come in (the files are listed again from what was found).
      $changed = $null -ne $State.Installed
      $State.Installed = $installed
      if ($changed -and $null -eq $State.Candidates) { $State.Candidates = @($State.AllFound) }
    }
  }
  if ($null -ne $State.Candidates -and $null -ne $State.Installed) {
    $State.AllFound = @($State.Candidates)
    Set-Songs
  }
  # Files wait for an extension: look again now and then, in case it was just installed.
  if ($State.Needs.Count -gt 0 -and -not $State.Codec -and
    ([DateTime]::UtcNow - $State.CodecCheckedAt).TotalSeconds -ge $Timing.CodecRecheckSeconds) {
    Start-CodecCheck
  }
  Complete-Dialog
  if ($State.SettingsDirtyAt -and ([DateTime]::UtcNow - $State.SettingsDirtyAt).TotalMilliseconds -ge $Timing.SettingsSaveDelayMs) {
    Save-Settings -Now
  }
}

# The position in the song; a song that ended or failed goes on to the next.
function Update-Playback {
  if ($State.Mode -eq 'stopped') { return 0.0 }
  if ($State.Radio.On) { return Update-RadioPlayback }
  $position = Get-EnginePosition
  $winrt = $State.Engine -eq 'winrt'
  $native = $State.Engine -eq 'native'
  if ($State.KnownDuration -le 0) {
    if ($native) {
      if ($State.Native.Duration -gt 0) { $State.Duration = $State.Native.Duration }
    } elseif ($winrt) {
      $length = $script:rt.PlaybackSession.NaturalDuration.TotalSeconds
      if ($length -gt 0) { $State.Duration = $length }
    } elseif ($player.NaturalDuration.HasTimeSpan) {
      $State.Duration = $player.NaturalDuration.TimeSpan.TotalSeconds
    }
  }
  if ($State.Lyrics -and -not $State.Lyrics.Timed -and $State.Duration -gt 0) {
    Set-UnsyncedTimes $State.Lyrics $State.Duration
  }
  $failed = $State.MediaFailed
  $ended = $State.MediaEnded
  if ($native) {
    # PecoFence says how its player goes; a video it has not answered for is given up.
    $elapsed = ([DateTime]::UtcNow - $State.StartedAt).TotalSeconds
    $failed = $State.Native.State -eq 'error' -or ($State.Native.State -eq 'loading' -and $elapsed -gt $Timing.VideoOpenSeconds)
    $ended = $State.Native.State -eq 'ended'
  } elseif ($winrt) {
    # The other player tells nothing by itself: a source that failed to open is "Failed", and a
    # song that is over stops ("Paused") at its end while we think it plays.
    $elapsed = ([DateTime]::UtcNow - $State.StartedAt).TotalSeconds
    $failed = ($State.RtSource -and [string]$State.RtSource.State -eq 'Failed') -or
      ($State.Duration -le 0 -and $elapsed -gt $Timing.RtOpenSeconds)
    $ended = $State.Mode -eq 'playing' -and $State.Duration -gt 0 -and $elapsed -gt 1 -and
      [string]$script:rt.PlaybackSession.PlaybackState -eq 'Paused' -and $position -ge $State.Duration - $Timing.RtEndedMarginSeconds
  }
  # A song still playing well after its duration (which Windows may have underestimated a
  # little) is over too, should the event be missed.
  $finished = $ended -or
    ($State.Mode -eq 'playing' -and $State.Duration -gt 0 -and $position -ge $State.Duration + $Timing.OverrunSeconds)
  if ($failed -and -not $winrt -and -not $native -and -not $State.Retried) {
    # WPF's player cannot play it: the other one may (a codec that only a Store extension gives it).
    $State.Retried = $true
    $State.MediaFailed = $false
    $State.StartedAt = [DateTime]::UtcNow
    try {
      Open-Engine $State.Files[$State.Cur] 'winrt'
      return 0.0
    } catch {
      Write-Log "Other player: $($_.Exception.Message)"
    }
  }
  if ($failed) {
    Write-Log "Cannot play: $($State.Files[$State.Cur])"
    Register-Needs $State.Files[$State.Cur]
    $State.Failures++
    if ($State.Failures -ge $State.Files.Count) { Stop-Media } else { Step 1 $true }
    return 0.0
  }
  if ($finished) {
    $State.Failures = 0
    if ($cfg.repeat -eq 'one') { Start-Track $State.Cur } else { Step 1 $true }
    return 0.0
  }
  # Crossfade: the last seconds of the file fade into the next one (a short file gets a shorter
  # crossfade, a third of it at most).
  if ($cfg.crossfade -and -not $State.Fade -and $State.Mode -eq 'playing' -and $State.Duration -gt 0 -and
    -not $native -and $cfg.repeat -ne 'one' -and (Test-NextTrack) -and -not (Test-NextIsVideo)) {
    $seconds = [Math]::Min([double]$CrossfadeSeconds, $State.Duration / 3)
    if ($seconds -ge 1 -and $position -ge $State.Duration - $seconds) {
      $State.Failures = 0
      Start-Crossfade $seconds
      return 0.0
    }
  }
  $position
}

# ---- Main loop ----------------------------------------------------------------------------
# What a click sends comes as lines on the standard input, read without waiting.
$stdin = [Console]::OpenStandardInput()
$inBuffer = New-Object byte[] 1024
$inRead = $stdin.ReadAsync($inBuffer, 0, $inBuffer.Length)
$inText = ''

Set-Volume
# The images of an earlier version, which grabbed the pictures of videos into this folder.
try { Remove-Item -LiteralPath (Join-Path $DataDir 'frames') -Recurse -Force -ErrorAction SilentlyContinue } catch { }
# The first screen shows at once; a big folder takes a moment to list, in the background.
Show-Screen (Get-Screen 0.0)
Start-CodecCheck
if ($cfg.playlist -and (Test-Path -LiteralPath $cfg.playlist -PathType Leaf)) {
  Open-Playlist $cfg.playlist
} else {
  $cfg.playlist = ''
  Start-Scan
}

$lastRender = [DateTime]::MinValue
while ($true) {
  try {
    # A click shows at once; the time and the lyrics follow every few tenths of a second.
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
    if ($dirty -or ([DateTime]::UtcNow - $lastRender).TotalMilliseconds -ge $Timing.RenderMs) {
      $lastRender = [DateTime]::UtcNow
      Pump
      Update-Background
      $position = Update-Playback
      Update-Crossfade
      Update-OnlineLyrics $position
      Update-OnlineSubtitles
      Update-Radio
      Show-Screen (Get-Screen $position)
    }
  } catch {
    Write-Log "$($_.Exception.Message) at line $($_.InvocationInfo.ScriptLineNumber)"
  }
  Start-Sleep -Milliseconds $Timing.PollMs
}
Save-Settings -Now
