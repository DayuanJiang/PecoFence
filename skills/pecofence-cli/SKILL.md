---
name: pecofence-cli
description: "Configure PecoFence (Windows desktop fences) from the terminal: list/create/move fences, move and rename desktop icons using file metadata, per-fence options, global settings, auto-sorting rules, layout snapshots. Use when the user asks to organise their desktop icons or change PecoFence settings."
---

# pecofence-cli

`pecofence-cli` drives the PecoFence app already running on the user's Windows desktop. Every
command prints JSON (except `log`). `pecofence-cli <command> --help` shows the flags with an example;
`pecofence-cli describe` is the full catalog and `describe --schema FenceDto|Settings|Cond` the exact
shapes. This file is enough to start; open `describe` only for something not covered here. Not for:
installing PecoFence, editing its `config.json` (the app owns it), or files outside the desktop and
portal folders.

## Five rules

1. **Snapshot first.** Before bulk changes run `snapshot save before-<task>`, keep `.snapshot.id` and
   tell the user `pecofence-cli snapshot restore <id>` undoes the layout. Snapshots hold layouts only
   (fences, geometry, membership), never settings, rules or files.
2. **Portals are real folders.** A fence with `"kind": "portal"` shows a folder on disk; `item move`
   into or out of it moves the real files (undo only via Explorer). Run such a move with `--dry-run`
   first and show the user the `fileMove` entries and their `destination`. Pair `--glob` with `--from`.
3. **Address by id.** After `fence list`, use ids (or a unique prefix of 6+ hex digits), not titles.
   The desktop fence is always `inbox`, whatever its localised title (Desktop, 桌面, ...).
4. **Ask for less.** `--fields id,title,rect,fit` keeps only those keys (dotted paths, lists
   element-wise); a full fence is ~700 bytes. If your reader decodes stdout with a legacy code page
   (Python without `encoding="utf-8"` on a CJK system, Windows PowerShell 5), add `--ascii`.
5. **Exit codes.** 0 ok. 1 the app refused, or part of a batch failed: stderr has
   `{"error": {"code", "message", "hint", "details"}}`, follow `hint`. 2 usage. 3 not running: ask the
   user to start PecoFence; never fall back to editing files. 4 timeout: a PecoFence menu or dialog is
   open, ask the user to close it, then check `fence list` before retrying (it may have run).

## Start

```
pecofence-cli status                                        # exit 3 = not running
pecofence-cli fence list --fields id,title,kind,tabHost,rect,fit,portal
pecofence-cli item list --fence inbox --fields id,fileName,kind,ext,modified,shortcutTarget,shortcutArguments
pecofence-cli monitor list                                  # workArea of each monitor, for placement
```

Not on PATH (portable ZIP)? Call the exe by its full path, e.g.
`%LOCALAPPDATA%\Programs\PecoFence\pecofence-cli.exe`.

Items carry `kind` (folders, programs, installers, shortcuts, documents, images, music, video,
archives, namespace, other), `ext`, `size`, `modified` / `created` (Unix seconds), `openCount`,
`lastOpened`, `shortcutTarget`, `shortcutArguments`, `path` and `assignedBy`. Sort by these; open a
file only when name and kind say nothing. Two shortcuts are duplicates only when target **and**
arguments match.

## Reading replies

- Mutations return `{"changed": bool, ...}` plus the affected object; `changed: false` means it was
  already so. A `warning` next to a success means something secondary failed or was skipped: report it.
- `snapshotId` appears when the app took a snapshot on its own first: `fence delete` of a fence with
  items, `rule apply` that moves items, `item move` of 20+ desktop items, `snapshot restore`.
- Coordinates are physical px in virtual-screen space (monitors left of the primary have negative x).
  A tab (`tabHost` set) reports its host window's `rect`; skip tabs when checking overlaps.
- `fit: {columns, rows, fittingHeight, overflow}`: `overflow: true` means the fence scrolls. When the
  app changes a rect on its own (auto height, whole-row snapping, size limits) the reply carries
  `adjusted: {requested, applied, reason}`: continue from `applied`.

## Cheat-sheet

```
pecofence-cli fence create --title Work --rect 40,40,520,360
pecofence-cli fence create --title Tools --right-of Work      # aligned + snapping gap; --below/--above/--left-of, --size w,h
pecofence-cli fence create --portal "C:\Users\me\Pictures"     # a folder as a fence: real files
pecofence-cli fence move Work --x 1200 --y 80                  # or --rect x,y,w,h
pecofence-cli fence fit Tools                                  # height that shows every item
pecofence-cli fence set Work layout list                       # iconSize 48 | spacing compact | opacity clear | locked true | autoHeight true
pecofence-cli fence set --all locked true                      # one call per fence, summary in .results
pecofence-cli fence set Dock titleAlign right                  # default|left|center|right; expandUpward true; collapsedSize default|fit|full
pecofence-cli fence merge Games --into Work                    # Games becomes a tab; `fence detach Games` undoes
pecofence-cli fence rename Work Projects
pecofence-cli fence delete Temp                                # its items go back to the desktop fence
pecofence-cli item move <id|path|name>... --to Work
pecofence-cli item move --glob "*.pdf" --from inbox --to Docs --dry-run
pecofence-cli item rename "C:\Users\me\Desktop\IMG_2031.pdf" "2026-09 electricity bill"   # keeps .pdf
pecofence-cli rule add --name PDFs --ext pdf --to Docs         # or --type, --name-contains, --glob, --json '[Cond...]'
pecofence-cli rule apply --dry-run && pecofence-cli rule apply
pecofence-cli settings set theme dark                          # dotted camelCase path; `settings get` shows them
pecofence-cli snapshot list
pecofence-cli snapshot restore <id> --dry-run                 # fences it would add / remove / move / update
pecofence-cli config export "C:\Users\me\Desktop\pecofence-backup.json"   # settings + rules + layouts
```

## Details that bite

- Quoting: values are parsed as JSON, else taken as text, so `48`, `true`, `list` need no quotes.
  Quote what the shell would eat: `"#ff8800"` (in Git Bash `#` starts a comment), `[`, `*`, spaces,
  and monitor ids such as `"\\.\DISPLAY2"`. Negative numbers work bare (`--x -1800`).
- Portal moves reply `fileMove: true` before the files arrive; moved files get new ids, so list the
  fence again before addressing them. A name that already exists at the destination is skipped
  (`skipped` + `warning`), never overwritten. A portal navigated into a subfolder reports
  `portal.current`, and moves to it land there. `--glob` without `--from` never takes portal items.
- `item rename` renames on disk; keep the extension unless the user asked otherwise.
- Tabs share their host's window: `fence move` / `resize` on a tab is `unsupported`, `--all` skips
  tabs, `fence fit` works only on the tab currently shown. Address the host, or `fence detach` first.
- Rules: the first match wins, and new desktop items are filed by them as they arrive. A rule cannot
  target a portal and needs at least one condition.
- `config import` / `backup restore` replace settings, rules and layouts: `config export` first.
- The CLI cannot delete files: list duplicates and junk for the user to remove.

## Recipes

**Tidy a messy desktop**
```
pecofence-cli snapshot save before-cleanup                     # keep .snapshot.id
pecofence-cli item list --fields id,fileName,kind,ext,fenceTitle,shortcutTarget,shortcutArguments,modified
# Group by what the metadata says: projects and bills by name, screenshots by ext + created,
# installers by kind, games by shortcutTarget (steam://, launcher paths). Leave anything unclear in
# the desktop fence. Look inside portals too: a "Pictures" folder full of game shortcuts is worth saying.
pecofence-cli fence create --title Projects --rect 40,40,520,360
pecofence-cli fence create --title Finance --right-of Projects
pecofence-cli item move <ids...> --to Projects
pecofence-cli rule add --name Bills --name-contains invoice --to Finance   # keeps it that way
pecofence-cli rule apply --dry-run && pecofence-cli rule apply
pecofence-cli fence list --fields title,rect,fit                # any fit.overflow true? `fence fit <id>`
```
Report what you grouped and why.

**Keep new desktop files sorted**: add rules; new desktop items are filed as they land. Files in
other folders (Downloads) are only visible through a portal, and rules do not act on portals. For
decisions a rule cannot express, a script can block on
`pecofence-cli watch --fence inbox --events item.added --once` (exits with the event), then run one
batch and wait again.

**Restyle every fence**
```
pecofence-cli snapshot save before-style
pecofence-cli fence set --all opacity clear
pecofence-cli fence list --fields title,opacity
```

## AGENTS.md snippet

```
PecoFence (desktop fences) is controlled with `pecofence-cli`: JSON output, exit 3 = app not running,
exit 4 = timeout (run `pecofence-cli fence list` before retrying; the command may have run).
Run `pecofence-cli describe` for the command catalog and `pecofence-cli skill` for usage rules.
Take `pecofence-cli snapshot save <name>` before bulk changes and restore by the returned id;
never edit its config.json directly. Moving items into or out of a folder portal moves real files.
```
