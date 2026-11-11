# Clipboard history (script fence)

The history of the Windows clipboard in a PecoFence **script fence** (PecoFence 0.3.0 or later):
texts, lists of files and pictures, one line per entry, newest first, like the Windows clipboard
history (Win + V). One PowerShell script, no install. The script is built into PecoFence: it is
`crates/app/assets/scripts/clipboard-history.ps1`.

## Use it

It is in **Settings → Tools** (and in *New tool* of the tray and fence menus) as **Clipboard
history**: click **Add**. Or by hand: **Use your own script → Choose script…**, then pick a copy of
`clipboard-history.ps1` (or `pecofence-cli fence create --script <path>\clipboard-history.ps1`).

The fence is called **Clipboard History** (`<pecofence.title>` in the header of the script). Keep
the file encoded as UTF-8 **with a BOM** (as shipped): Windows PowerShell 5.1 reads the accents and
the other alphabets wrongly without it.

## What the fence shows

At the top (these rows stay in place; the entries scroll under them):

| Control | Effect |
|---|---|
| **Show entries** | Ticked: the entries are readable. Not ticked: every entry, text or picture, is blurred. It protects what must not be seen (passwords...) while you stream on Twitch or YouTube |
| **Reveal on click** | Available while **Show entries** is ticked (grayed out otherwise). Ticked: the entries are blurred anyway, and a click on one shows it for 5 seconds (`$RevealSeconds`). PecoFence cannot tell a script which row the mouse is over, so this stands in for "show only on mouse over" |
| **Delete all** | Deletes every entry that is not pinned |
| **Purge entries in:** (at the right of **Delete all**) | Ticked: every entry is deleted that long after the first one came (from 5 seconds to 24 hours); the countdown starts again with the next entry. **<** and **>** go from one usual delay to the next (5, 10, 15, 20, 30, 45 s, 1 min, 1 min 30 s, 2 min, 3 min, 5 min...), a click on the delay asks for minutes and seconds in a small window. The time left shows in gray as minutes:seconds, counting down in real time |

Each entry is a line, as in the Windows clipboard history: its text or its picture, in a cell that takes the
width of the fence (a text that does not fit ends with an ellipsis; the whole text, the time and the
size of a picture are in its tooltip), and against the right edge, whatever the width of the fence, a
**trash** button with a **pin** button under it. A click on the entry puts it on the clipboard again. A blurred text or list of
files shows a blurred picture of itself.

**Pin** puts an entry at the top of the list, above a line. A pinned entry is kept by **Delete all**,
by the auto-delete and when the list is full, and it is still there when PecoFence starts again;
the trash button, or **Unpin**, lets it go.

## What is kept

- Texts, lists of files (copied in File Explorer) and pictures (screenshots, "Copy image"). 30 entries
  at most (`$MaxEntries`, not counting the pinned ones); the oldest go first. Copying again what is already in the list brings that
  entry back to the top.
- What a program asks not to be kept (password managers set the clipboard formats
  `ExcludeClipboardContentFromMonitorProcessing` or `CanIncludeInClipboardHistory` = 0) is left
  out, as Windows' own history leaves it out.
- The entries are in memory only and are gone when PecoFence ends, except the pinned ones, which
  are written to `pinned.json` in the fence's data folder. The pictures are stored as files in its
  `images` folder while their entry lives; at the start, what is not pinned is removed.
- Nothing is sent anywhere. The boxes and the purge delay are kept in `settings.json`.
  Errors go to `clipboard-history.log` there.

## Settings

At the top of the script: `$Language`, `$MaxEntries`, `$MaxTextChars`, `$TextWidth` (the width of
the blurred picture of a hidden text, in DIPs), `$ImageHeight`,
`$RevealSeconds`, `$PollMs` and `$DefaultMinutes`.

The script uses the rows of cells, pictures, check boxes and scrolling of the script fence output
(see [docs/CLI.md](../../docs/CLI.md#script-fence-output)).
