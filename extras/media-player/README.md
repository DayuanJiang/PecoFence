# Media player (script fence)

A media player for a PecoFence **script fence** (PecoFence 0.3.0 or later): music, sounds and
videos, like a small VLC or Windows Media Player. One PowerShell script, no install: the script
plays the files itself with Windows' media engine and shows the player in the fence. The script
is built into PecoFence: it is `crates/app/assets/scripts/media-player.ps1`.


## Use it

It is in **Settings → Tools** (and in *New tool* of the tray and fence menus) as **Media Player**:
click **Add**. Or by hand:

1. **Settings → Tools**: the Tools switch must be on.
2. **Use your own script → Choose script…**, then pick a copy of
   `crates/app/assets/scripts/media-player.ps1`
   (or `pecofence-cli fence create --script <path>\media-player.ps1`).
3. Make the fence tall enough for about 30 rows, and click **Browse...** to choose the folder
   with your files (or **Open playlist...**).

The fence is called **Media Player** (`<pecofence.title>` in the header of the script).

Keep the file encoded as UTF-8 **with a BOM** (as shipped): Windows PowerShell 5.1 reads the
emoji and accents wrongly without it.

## What the fence shows

| Row | Click |
|---|---|
| The tabs **Playback**, **Playlist** and **Radios** | Switch between the player, the list of files and the internet radios |
| The folder (or the playlist file) of the files | Opens its folder in File Explorer |
| **Browse...** and **Open playlist...** | Pick a folder (its sub-folders too), or a playlist (`.m3u`, `.m3u8`, `.pls`, `.wpl`) |
| The tags of a song: artist, album (year), genre and track number | |
| The album cover, or the picture of a video | |
| The title (the tags', else the file name), and under it the file's extension, sampling rate, bit depth and bit rate (a video: its size and frame rate too) | |
| Time, a **position slider**, duration | Click or drag to move in the file |
| Eight icon buttons: previous, play, pause, stop, next, repeat, shuffle, crossfade, then the playback speed (1.00), **<** and **>** (a click on the value goes back to 1.00) | Point at one to see its name; Previous and Next also name, on a second line, the file they would play |
| A volume icon, the **volume slider** and the volume in % | The icon mutes and unmutes |
| The **Online lyrics** and **Online subtitles** boxes | Tick the lookups below |
| **Synchronize:**, a value in seconds and **<** / **>** buttons | Shift the lyrics or subtitles by 0.1 s (**<** earlier, **>** later); a click on the value sets it back to 0 |
| The lyrics or subtitles | Scroll with the wheel or the scrollbar; the line being sung stays in the middle |
| At the bottom, what keeps files or subtitles from coming (a missing Store extension, a missing API key...) | Opens the page that fixes it |

**Previous** starts the file again after 3 seconds, else goes back one file. **Repeat** cycles
off, all and one file; **Shuffle** shuffles the play order. A button that is on is drawn pressed
in; repeat, shuffle and crossfade are grey when off. When the fence is too narrow, the buttons
that do not fit go on to the next line.

The folder or playlist you chose is remembered, with the volume and the other settings, in the
fence's data folder: when PecoFence starts again (at the start of Windows too) the Media Player opens
it again, with no need to choose it anew. Set `$RememberSource = $false` at the top of the script to
start from the Music folder each time instead.

**Media keys**: while a file plays or is paused, the keyboard's play/pause, next track, previous track
and stop keys drive the player, whichever window has the focus (many mice and headsets send them
too). Stopped, the player lets them go back to the other programs, so a browser's player has them
then. A key that another program already holds for itself is not available (PecoFence's log says
so). Set `$MediaKeys = $false` at the top of the script to leave the keys alone.

After the crossfade button come the playback speed (1.00 is normal; a click on it goes back to 1.00),
then **<** and **>**, which slow down and speed up the playback by 0.25 a click, from 0.25 to 4. The speed holds for the next files until you change
it, and starts at 1.00 each time PecoFence starts. Videos are played at that speed too, with the
sound's pitch kept.

The **Playlist** tab lists the files of the folder or playlist, 50 at a time, with buttons and a
slider to move through the pages. The tabs and the buttons stay in place and the page scrolls under
them (the mouse wheel, or the scrollbar at the right of the fence): a taller fence shows more of the
50 files at once. The file being played has a blue **▶**; a click selects a file, a double-click
plays it and goes back to Playback.

The **Radios** tab plays free internet radios. Its five buttons (point at one to see its name) choose
the list: the **popular** radios, the radios **by country**, **by genre**, your **favorites**, or a
**search by name** (a small window asks for the name). A country or a genre is picked from its
list with a click. A click on a radio selects it, and then **Play** and **Add to favorites** (or
**Remove from favorites**) appear; a double-click plays it and goes back to Playback. The radio being
played has a blue **▶**. In Playback a radio shows its name, its format and bit rate, the time it
has played and **LIVE** where the duration is; Previous and Next go to the radio before and after it
in the list (their tooltips name it), Pause and Play join the live stream again, and a radio that
cannot be played says so at the bottom. A stream that was playing and goes silent is opened again up
to three times. Radios have no lyrics or subtitles, so their boxes are left out. Like the Playlist,
the list of radios, countries or genres is paged by 50 and scrolls under the buttons.

The radios come from Radio Browser (<https://api.radio-browser.info>), a free and open directory:
no key and no payment, and no paid or licensed directory (SHOUTcast, TuneIn...) is used. Nothing
is asked of the internet until you click one of the five buttons; then the list of the
directory's servers is read once and one of them answers (a server that is down is skipped). Only
the MP3 and AAC streams the directory found alive are listed (not HLS playlists). The stream itself
is played by Windows' media engine, straight from the radio's own server. The favorites are kept
with their name and address in the fence's `settings.json`, so they play without the directory.
Set `$CountListens = $true` at the top of the script to tell Radio Browser one more person listens
when a radio starts (it ranks the radios, and the directory then sees this computer's address).

The **album cover** comes from the song's tags (mp3 `APIC`, m4a `covr`, flac picture, Ogg
`METADATA_BLOCK_PICTURE`), else from a `cover`, `folder`, `front`, `album` or `albumart` picture in
its folder. It is copied once to `covers` in the fence's data folder (the 200 newest are kept).

## Videos

The picture of a video is shown in the place of the album cover, above the title, at most 220 DIPs
tall (`$Layout.VideoHeight`) and never wider than the fence. Under the title: its size, frame rate,
sampling rate and bit rate (`640x360, 25 fps, 44.1 kHz, 826 kbit/s`).

PecoFence plays the video itself, picture and sound, on the graphics card (the `video=` row of
[Script fence output](../../docs/CLI.md#videos)): the picture is smooth, at the video's own rate,
and costs the processor next to nothing. The script only drives it (play, pause, volume, moves) and
reads from PecoFence where it is (position, end, failure). The picture is taken only while the
fence is in view; the sound goes on, also while the Playlist tab is shown. A click on the picture
plays or pauses.

**WebM and Ogv videos** play too, once the Store's extension they need is installed (see below).

**Subtitles**: a `Video name.srt` next to the video is shown as the lyrics are (the cue being
shown is highlighted, the next ones follow). A `.lrc` works too. Without one, the **Online
subtitles** box looks them up on opensubtitles.com (below). The online lyrics lookup is for songs
only.

Under the volume, two boxes side by side, **Online lyrics (lrclib.net)** and **Online subtitles
(opensubtitles.com)**, and under them **Synchronize:** with its value in seconds and its **<** and
**>** buttons: it shifts the lyrics of a song or the subtitles of a video (a positive value shows
them later), and the value is kept for that file.

## Lyrics

The lyrics of a song (or the subtitles of a video) are looked up in this order:

1. `Name.lrc`, `Name.srt` or `Name.txt` next to the file.
2. The lyrics stored in the tags: mp3 `USLT`, m4a `(c)lyr`, flac / ogg / opus `LYRICS` or
   `UNSYNCEDLYRICS`.
3. lrclib.net for a song, when **Online lyrics** is ticked; opensubtitles.com for a video, when
   **Online subtitles** is ticked (below).

A `.lrc` with a time per line highlights the line being sung; with word times
(`[01:23.45]<01:23.45>word <01:23.90>word`) the word being sung is in bold, and without them the
words are spread over the line. Lyrics without any time are spread over the whole song, which is
only approximate, and the fence says so. **Synchronize:** shifts them for the file being played
(from -60 to +60 seconds); the value is kept for that file.

All the lines of the lyrics are listed, under the player's buttons, which stay in place: the fence
scrolls them (the mouse wheel, or the scrollbar at the right), so a taller fence shows more lines.
The line being sung stays in the middle; scroll by hand to read ahead or back, and the lyrics
follow the singing again a few seconds after you stop. A song of more than `$LyricsBefore +
$LyricsAfter` lines (100 + 100) lists that many lines around the one being sung. A fence too short
to leave room for the lyrics under the buttons scrolls everything, the buttons included.

### Online lyrics (lrclib.net)

Off until you tick the box. A song without lyrics of its own is looked up on
[lrclib.net](https://lrclib.net/docs) after it has played for 3 seconds: its artist, title, album
and duration are sent (from the tags, or from a file named `Artist - Title`), nothing else. As the
service asks, the script names itself in the `User-Agent`, sends one request at a time at least
500 ms apart, asks a song once (the answer, and "not found" for a week, are kept in `lyrics-cache`
in the fence's data folder), and waits as long as a `429` or `503` answer's `Retry-After` asks.

## Crossfade

The button after Shuffle (two opposite arrows, ⇄) turns the crossfade on and off. On, the last 10
seconds of a file (`$CrossfadeSeconds`) fade out while the next file of the play order starts and
fades in over them, at constant power (no dip in the middle). A file shorter than 30 seconds gets a
crossfade of a third of its length. Nothing fades when nothing comes next: the end of the list
without Repeat, or Repeat one. A video is not faded: it plays its own sound from its start, so
nothing fades out of a video, and a crossfade does not start when the next file is a video.

During a crossfade, Pause holds both files, a move in the new file moves the fade with it, and
choosing another file (Next, Previous, a double-click in the Playlist tab, Stop) cuts the file
that fades out at once. It works between the two media players of Windows. The setting is kept with
the others.

## Online subtitles (opensubtitles.com)

Off until you tick the box. It needs **your own API key** (free): make an account on
opensubtitles.com, then create a key in *API consumers* of your profile
(<https://www.opensubtitles.com/en/consumers>), and put it at the top of the script (right-click
the fence > Edit script):

```powershell
$OpenSubtitles = @{ ApiKey = 'your key'; Languages = 'auto' }
```

`Languages` lists the languages of the subtitles, best first (`'fr,en'`); `auto` is PecoFence's
language, then English. Without a key, a line at the bottom of the player says so, and a click on
it opens the page where keys are made.

A video that has no subtitles of its own is looked up after it has played for 3 seconds (skipping
through videos sends nothing): first by the **fingerprint of the file** (opensubtitles.com's
"moviehash", made from its size and its first and last 64 KB, so the subtitles found fit this very
version), then, when that finds nothing, by its **file name** (`Long.Movie.2019.mp4` is looked up
as `long movie 2019`). Of what is found, the subtitle made for this very file comes first, then the
language listed first, then one made by people rather than by a machine, then the most downloaded.
It is downloaded as `.srt`. The fingerprint, the file name and the languages are sent; nothing
else.

The script follows opensubtitles.com's conditions
([Getting started](https://opensubtitles.tawk.help/article/getting-started),
[About the API](https://opensubtitles.tawk.help/article/about-the-api)):

- The API key goes with every request (`Api-Key` header), never to another server (the file of a
  download link is fetched without it).
- The `User-Agent` names the application and its version: `PecoFence-MediaPlayer v1.0`.
- The parameters of a search are in alphabetical order (`languages`, `moviehash`, `query`), in
  lower case, with `+` for the spaces; redirects are followed.
- Searching is free, downloading is counted: 5 subtitles a day without signing in (the script does
  not sign in). A subtitle is downloaded once and kept in `subtitles-cache` in the fence's data
  folder ("not found" for a week), so playing a video again asks nothing. When the quota is used up
  (the last download says so, or the server answers `406`), no lookup is made until the time the
  answer gives, and a line at the bottom of the player shows that time.
- One request at a time, a second apart. On `429` or `503` the script waits for the time of the
  `Retry-After` header (plus up to 30 %), five minutes after a network or server error, and an
  hour after the key was refused (`401` or `403`, shown at the bottom of the player).

Their terms of service forbid the commercial use of the subtitles. The lookup only shows them in
the fence; nothing is written next to your files.

## Formats and codecs

Played by Windows' media engine without anything installed (what Windows has in itself):

| | |
|---|---|
| Audio | mp3, m4a, m4b, aac, wma, wav, flac, aif, aiff, au, mp2, mka |
| Video | mp4, m4v, mov, mkv, avi, wmv, asf, mpg, mpeg, m2ts, mts, 3gp, 3g2 |

`.ts` (MPEG transport streams) is not listed, as it is also the extension of TypeScript files: add
it to `$VideoExtensions` at the top of the script to have it.

### What needs a library of the Microsoft Store

Windows leaves some codecs to free (or cheap) extensions of the Microsoft Store. The script checks
which are installed (every 30 seconds while a file waits for one, so a file comes in as soon as the
extension is installed, without a restart), and a line at the bottom of the player names the one a
file needs, with the number of files; a click on it opens its page in the Microsoft Store.

| Extension | For | Store page |
|---|---|---|
| **Web Media Extensions** | Ogg Vorbis, Opus (`.ogg`, `.oga`, `.opus`), WebM and Ogv (`.webm`, `.ogv`: VP8, Theora) | [`9N5TDP8VCMHS`](https://apps.microsoft.com/detail/9N5TDP8VCMHS) |
| **VP9 Video Extensions** | VP9 video (mostly in `.webm`) | [`9N4D0MSMP0PT`](https://apps.microsoft.com/detail/9N4D0MSMP0PT) |
| **HEVC Video Extensions** | H.265 / HEVC video (`.mp4`, `.mkv`, `.mov`...) | [`9NMZLZ57R3T7`](https://apps.microsoft.com/detail/9NMZLZ57R3T7) (paid; some PCs have a free version from their maker) |
| **AV1 Video Extension** | AV1 video | [`9MVZQVXJBQ9V`](https://apps.microsoft.com/detail/9MVZQVXJBQ9V) |
| **MPEG-2 Video Extension** | MPEG-2 video (`.mpg`, DVD and TV recordings) | [`9N95Q1ZZPMH4`](https://apps.microsoft.com/detail/9N95Q1ZZPMH4) |

How the script knows:

- Files of a format that always needs an extension (Ogg, Opus, WebM, Ogv) are left out of the list
  until it is installed, and counted in the line at the bottom.
- For the others, the codec is inside the file. A file that WPF's player cannot play is tried with
  Windows' other player; if that fails too, it is skipped and the codec read from the file's
  properties (HEVC, AV1, VP9, MPEG-2) says which extension to advise. A video that PecoFence
  cannot play is advised the same way.
- Songs are played by one of two Windows media players. WPF's plays what Windows has in itself. The
  one of Store apps (`Windows.Media.Playback`) also uses the extensions, so it plays Ogg and Opus
  once they are installed; the script takes it for the formats that need an extension, and after
  WPF's one failed for the others. Videos are played by PecoFence's own player, which uses
  Media Foundation, as the Store extensions do: WebM and the codecs above play there too once
  they are installed. No library comes with the script.

Not played by either engine, so not listed: AC-3 and DTS files on their own, APE, WavPack, CAF,
AMR, RealMedia, FLV. A file that cannot be played is skipped (`media-player.log` in the fence's
data folder says which).

## Settings in the script

At the top of the script: `$Language` (`auto`, or a language to force), `$AudioExtensions`,
`$VideoExtensions`, `$StoreAudioExtensions`, `$StoreVideoExtensions`, `$StorePackages` (the
extensions of the Store: package name, title and page), `$MediaKeys`, `$RememberSource`, `$Recurse`, `$MaxSongs`, `$LyricsBefore`,
`$LyricsAfter`, `$CrossfadeSeconds` and `$OpenSubtitles` (your API key and the languages of the
subtitles). Below them,
`$Layout` (the size of the cover and of the picture of a video), `$Timing` (including
`VideoOpenSeconds`, how long PecoFence has to open a video), `$Glyph`, `$Emoji`, `$Lrclib` and
`$OpenSubtitlesApi` (the address, User-Agent and delays of opensubtitles.com).

## Language

The texts follow PecoFence's display language (English, French, German, Spanish, Italian,
Brazilian Portuguese, Russian, Japanese, Korean, Simplified and Traditional Chinese), which
PecoFence gives the script in `PECOFENCE_LANGUAGE`; else Windows' language, else English. Set
`$Language` at the top of the script to force one.

## Notes

- The player uses PecoFence's rows of cells and picture rows: see *Script fence output* in
  `docs/CLI.md`.

- Playback stops with the fence (deleted, script changed, Tools switch off, PecoFence quits).
- The folder, playlist, volume, mute, shuffle, repeat, crossfade, the two online boxes, the shifts and the favorite
  radios are kept in the fence's `settings.json`; errors go to `media-player.log` there.
- The fence's data folder is not the folder of your files: nothing is written next to them.
