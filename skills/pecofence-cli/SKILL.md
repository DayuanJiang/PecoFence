---
name: pecofence-cli
description: Configure PecoFence (Windows desktop fences) from the terminal: list/create/move fences, move and rename icons with file metadata, set options, rules (with dry run), snapshots, event stream. Use when the user asks to organise their desktop icons or change PecoFence settings.
---

# pecofence-cli

`pecofence-cli` drives the PecoFence app that is already running on the user's Windows desktop.
Every command prints JSON; nothing else. Run `pecofence-cli <command> --help` when unsure about flags.

## When to use / not to use

- Use for: creating, moving, resizing, renaming, deleting fences; moving desktop icons between fences;
  renaming the files behind icons; per-fence options (layout, icon size, opacity, lock, roll-up);
  global settings; auto-sorting rules (with `--dry-run`); layout snapshots; quick-hide / show /
  Peek; waiting for desktop changes (`watch --once`); checking a config file (`config check`).
- Do not use for: installing PecoFence, editing `config.json` by hand (the app owns that file), moving
  files outside the desktop.

## Bootstrap

1. `pecofence-cli status` — exit 0 means the app is up. Exit 3 (`not_running`): ask the user to start
   PecoFence, then retry. Never fall back to editing config files.
2. `pecofence-cli describe` — command catalog, exit codes, error codes, notes (works offline).
3. `pecofence-cli describe --schema Settings` (or `FenceDto`, `Cond`, `RuleSet`) — exact keys and enums.
4. `pecofence-cli fence list` — get fence ids and titles before addressing anything. The desktop
   itself is the fence with `"kind": "inbox"`; its title follows the user's language (Desktop, 桌面,
   デスクトップ, ...), so never match on it: the selector `inbox` always addresses it.
5. `pecofence-cli item list --fence inbox` — every item with `kind` (folders/programs/installers/
   shortcuts/documents/images/music/video/archives/namespace/other), `ext`, `size`, `modified`,
   `created`, `openCount`, `lastOpened`, `shortcutTarget`, `fileName`, `path`. Sort by these first;
   open a file (read a PDF's first page, look at an image) only when name and kind are not enough.

## Output contract

- stdout: the result as JSON (single line when piped). Mutations return `{"changed": bool, ...}`;
  `changed:false` means it was already so — not an error.
- `snapshotId` appears only on the commands that take an automatic layout snapshot first:
  `fence delete` (when the fence has items), `rule apply` (when it moves something), `item move`
  (20 or more desktop items changing fence) and `snapshot restore`. Snapshots cover fence layouts only (fences, geometry, item membership), never
  settings or rules, so `settings set` and `rule import` return no `snapshotId`.
- stderr on failure: `{"error":{"code":"fence_not_found","message":"...","hint":"...","details":{...}}}`.
  Follow `hint`; `details.candidates` / `details.allowed` / `details.expected` are machine-readable.
- Exit codes: 0 ok, 1 the app returned an error (or part of a `--all` / `--glob` batch failed), 2 usage,
  3 not running, 4 timeout (the app is inside a menu/dialog: ask the user to close it). After an
  exit 4 run `fence list` before retrying: the app drops expired requests, but one that was picked up
  at the last moment may still have run.
- A `warning` field may accompany a successful result (e.g. applied but the config file was not saved,
  or the 20-snapshot limit prevented the automatic snapshot).

## Selectors

- Fence: full id, unique id prefix (at least 6 hex chars), or title (exact, case-insensitive; then a
  unique substring). `ambiguous_fence` lists candidates. Prefer ids after the first `fence list`.
  `inbox` is a fixed alias for the desktop fence, whatever its title (`--fence inbox`, `--to inbox`).
- Item: id, full path, or unique display name; `item move --glob "*.pdf"` matches file names.
  `item list --kind documents --ext pdf` filters on the client.
- Rule: id, 0-based index, or name.
- Snapshot: id or unique id prefix. `snapshot save` returns `snapshot.id`; keep it and restore by id.
  A name works only while it is unique; with several snapshots of the same name the call fails with
  `snapshot_not_found` and `details.candidates`.

## Quoting

Values are parsed as JSON first, then taken as text, so `48`, `true`, `null` and `list` need no quotes.
Quote anything that the shell would eat: values starting with `#` (Git Bash starts a comment there),
`[`, `*`, or containing spaces. `fence set Work tint "#ff8800"` or `tint '"#ff8800"'` both work;
`fence set Work tint #ff8800` loses the value and fails with `usage: missing FENCE`. Negative numbers
work bare: `settings set snapping.gapPx -4`. `--string` keeps numeric-looking text a string
(`fence set Work title 2024 --string`; `fence rename Work 2024` is simpler).

## Cheat-sheet

```
pecofence-cli fence list                                   # ids, titles, kind, rect (physical px), options
pecofence-cli fence create --title Work --rect 100,100,600,400
pecofence-cli fence move Work --x 1200 --y 80              # or --rect x,y,w,h / --monitor <id>
pecofence-cli fence set Work layout list                   # iconSize 48 | opacity clear | locked true
pecofence-cli fence set Work tint "#ff8800"                # quote colours (see Quoting)
pecofence-cli fence set --all locked true                  # one call per fence, summary in .results
pecofence-cli item list --fence inbox                      # id, name, path, kind, ext, size, modified, created, openCount, shortcutTarget
pecofence-cli item move --glob "*.pdf" --from inbox --to Docs   # or item ids / paths / names
pecofence-cli item rename "C:\Users\me\Desktop\IMG_2031.pdf" "2026-09 electricity bill"   # keeps .pdf
pecofence-cli rule apply --dry-run                         # what the rules would move, nothing changes
pecofence-cli watch --fence inbox --once                   # block until one item lands on the desktop
pecofence-cli config check                                 # lint the config in use (offline)
pecofence-cli paths                                        # config, backups, log, crash dumps
pecofence-cli settings set peek.enabled false              # dotted camelCase path, JSON value
pecofence-cli rule add --name PDFs --ext pdf --to Docs && pecofence-cli rule apply
pecofence-cli snapshot save before-cleanup                 # note .snapshot.id; restore with: snapshot restore <id>
```

## Safety

- Before bulk changes run `pecofence-cli snapshot save before-<task>` and keep the returned
  `snapshot.id`; tell the user `pecofence-cli snapshot restore <id>` undoes the layout changes.
- Snapshots are layout-only. The app takes one automatically before `fence delete` (with items),
  `rule apply` (when it moves items) and `snapshot restore`. Automatic snapshots never evict the
  user's own; at the 20-snapshot limit none is taken and the result carries a `warning`.
- Folder portals show real files. `item move` into or out of a portal fence is an Explorer file
  move on disk (undo only via Explorer Ctrl+Z; snapshots cannot revert it). `item move --glob`
  therefore skips portal items unless `--from <portal>` names that portal, and you should always
  pair `--glob` with `--from`. Check `fence list` for `"kind": "portal"` before choosing `--to`.
- Prefer `fence set --all`, `item move --glob --from`, and `rule add` + `rule apply` over many tiny
  calls. `--all` skips hosted tabs (`"skipped": "tab"` in `.results`); address the host fence.
- `item rename` changes the file's name on disk (undo: rename it back). It keeps the extension
  unless `--keep-ext false`; never strip or change an extension unless the user asked for it.
- Prefer `rule apply --dry-run` before `rule apply` when rules were just added or changed; show
  the `moves` list to the user if it is long or surprising.
- Deleting a fence returns its items to the desktop fence and removes rules that target it.
- Do not retry a command that timed out (exit 4) blindly: check `fence list` first; the app drops
  expired requests, but the state may already have changed.

## Recipes

**"Organise my desktop into Work, Games and Temp fences"**
```
pecofence-cli snapshot save before-organise      # keep .snapshot.id for `snapshot restore <id>`
pecofence-cli monitor list                      # pick workArea of the primary monitor
pecofence-cli fence create --title Work  --rect 40,40,520,360
pecofence-cli fence create --title Games --rect 600,40,520,360
pecofence-cli fence create --title Temp  --rect 1160,40,520,360
pecofence-cli item list --fence inbox            # decide per item (name/path) where it belongs
pecofence-cli item move --glob "*.lnk" --from inbox --to Games     # e.g. game shortcuts
pecofence-cli item move "C:\Users\me\Desktop\Q3 plan.docx" "budget" --to Work
pecofence-cli item move --glob "*.tmp" --from inbox --to Temp
```

**"My desktop is a mess: group it into something meaningful"**
```
pecofence-cli snapshot save before-cleanup       # keep .snapshot.id
pecofence-cli item list --fence inbox            # read kind, ext, modified, created, shortcutTarget for every item
# Group by what the metadata says (projects by name prefix, invoices/statements by name, screenshots
# by ext+created, installers by kind). Only for files whose name says nothing, read the file
# (a PDF's first page, an image) and decide.
pecofence-cli fence create --title "Thesis"   --rect 40,40,520,360
pecofence-cli fence create --title "Finance"  --rect 600,40,520,360
pecofence-cli item move <ids or paths...> --to Thesis
pecofence-cli item rename "C:\Users\me\Desktop\IMG_2031.pdf" "2026-09 electricity bill"   # meaningful names
pecofence-cli item move --glob "*bill*" --from inbox --to Finance
pecofence-cli rule add --name Invoices --name-contains invoice --to Finance   # keep it that way
pecofence-cli rule apply --dry-run && pecofence-cli rule apply
```
Report what you grouped and why; leave anything you are unsure about in the desktop fence.

**"Organise new downloads when they arrive"** (a script or scheduled task, not a resident agent)
```
pecofence-cli watch --fence inbox --events item.added --once   # exits 0 with the event when a file lands
pecofence-cli item list --fence inbox                          # then run one batch as above
```
Loop the two steps in a script; the agent runs only for the batch, not while waiting.

**"Put every PDF into a Docs fence and keep it that way"**
```
pecofence-cli fence create --title Docs --rect 100,100,600,400   # skip if `fence get Docs` works
pecofence-cli rule add --name PDFs --ext pdf --to Docs           # rules file new items automatically
pecofence-cli rule apply                                         # re-file existing items now
pecofence-cli rule list                                          # verify order; first match wins
```

**"Make all fences more transparent and lock them"**
```
pecofence-cli snapshot save before-style
pecofence-cli fence set --all opacity clear
pecofence-cli fence set --all locked true
pecofence-cli fence list | jq '[.[] | {title, opacity, locked}]'   # verify
```

## Limits

- `config import` / `backup restore` replace everything (settings, rules, layouts); the current layout is
  snapshotted first, but settings and rules are not covered by snapshots. Export first: `config export <abs path>`.
- Tabs hosted inside another fence share the host's window: `fence move/resize` on a tab fails with
  `unsupported`, `--all` batches skip tabs; address the host fence or `fence detach` first.
- Coordinates are physical pixels in virtual-screen space (multi-monitor: monitors left of the
  primary have negative x). Read `monitor list` for work areas; rects off every monitor or larger
  than the virtual screen are rejected with `invalid_value`.
- Titles, rule names and snapshot names are limited to 256 characters.
- `settings set autostart ...` on the Microsoft Store build opens Windows' Startup Apps page instead.
- `--instance <name>` / `PECOFENCE_INSTANCE` address a separately started test instance only; normal
  users never need it.
- Only `describe`, `skill`, `paths`, `log` and `config check` work without the app. `log` prints
  text, everything else JSON. `watch` never returns on its own; use `--once` from scripts.

## AGENTS.md snippet

```
PecoFence (desktop fences) is controlled with `pecofence-cli`: JSON output, exit 3 = app not running,
exit 4 = timeout (run `pecofence-cli fence list` before retrying; the command may have run).
Run `pecofence-cli describe` for the command catalog and `pecofence-cli skill` for usage rules.
Take `pecofence-cli snapshot save <name>` before bulk changes and restore by the returned id;
never edit its config.json directly. Moving items into or out of a folder portal moves real files.
```
