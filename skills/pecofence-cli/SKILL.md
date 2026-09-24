---
name: pecofence-cli
description: Configure PecoFence (Windows desktop fences) from the terminal: list/create/move fences, move icons, set options, rules, snapshots. Use when the user asks to organise their desktop icons or change PecoFence settings.
---

# pecofence-cli

`pecofence-cli` drives the PecoFence app that is already running on the user's Windows desktop.
Every command prints JSON; nothing else. Run `pecofence-cli <command> --help` when unsure about flags.

## When to use / not to use

- Use for: creating, moving, resizing, renaming, deleting fences; moving desktop icons between fences;
  per-fence options (layout, icon size, opacity, lock, roll-up); global settings; auto-sorting rules;
  layout snapshots; quick-hide / show / Peek.
- Do not use for: installing PecoFence, editing `config.json` by hand (the app owns that file), moving
  files outside the desktop, or anything needing a file dialog (config import/export).

## Bootstrap

1. `pecofence-cli status` — exit 0 means the app is up. Exit 3 (`not_running`): ask the user to start
   PecoFence, then retry. Never fall back to editing config files.
2. `pecofence-cli describe` — command catalog, exit codes, error codes, notes (works offline).
3. `pecofence-cli describe --schema Settings` (or `FenceDto`, `Cond`, `RuleSet`) — exact keys and enums.
4. `pecofence-cli fence list` — get fence ids and titles before addressing anything.

## Output contract

- stdout: the result as JSON (single line when piped). Mutations return `{"changed": bool, ...}`;
  `changed:false` means it was already so — not an error. Destructive commands add `snapshotId`.
- stderr on failure: `{"error":{"code":"fence_not_found","message":"...","hint":"...","details":{...}}}`.
  Follow `hint`; `details.candidates` / `details.allowed` / `details.expected` are machine-readable.
- Exit codes: 0 ok, 1 the app returned an error (or part of a `--all` / `--glob` batch failed), 2 usage,
  3 not running, 4 timeout (the app is inside a menu/dialog: ask the user to close it, retry).
- A `warning` field may accompany a successful result (e.g. applied but the config file was not saved).

## Selectors

- Fence: full id, unique id prefix (at least 6 hex chars), or title (exact, case-insensitive; then a
  unique substring). `ambiguous_fence` lists candidates. Prefer ids after the first `fence list`.
- Item: id, full path, or unique display name; `item move --glob "*.pdf"` matches file names.
- Rule: id, 0-based index, or name. Snapshot: id or name. `--to inbox` targets the desktop fence.

## Cheat-sheet

```
pecofence-cli fence list                                   # ids, titles, rect (physical px), options
pecofence-cli fence create --title Work --rect 100,100,600,400
pecofence-cli fence move Work --x 1200 --y 80              # or --rect x,y,w,h / --monitor <id>
pecofence-cli fence set Work layout list                   # iconSize 48 | opacity clear | locked true
pecofence-cli fence set --all locked true                  # one call per fence, summary in .results
pecofence-cli item list --fence 桌面                        # items with id, name, path, fence
pecofence-cli item move --glob "*.pdf" --to Docs           # or item ids / paths / names
pecofence-cli settings set peek.enabled false              # dotted camelCase path, JSON value
pecofence-cli rule add --name PDFs --ext pdf --to Docs && pecofence-cli rule apply
pecofence-cli snapshot save before-cleanup                 # restore with: snapshot restore before-cleanup
```

## Safety

- Before bulk changes run `pecofence-cli snapshot save before-<task>`; tell the user how to restore.
- The app snapshots automatically before deleting a fence that has items, replacing the rule set,
  replacing the whole Settings object, and restoring a snapshot (`snapshotId` in the result).
- Prefer `fence set --all`, `item move --glob`, and `rule add` + `rule apply` over many tiny calls.
- Deleting a fence returns its items to the desktop fence and removes rules that target it.
- Do not retry a command that timed out (exit 4) blindly: check `fence list` first; the app drops
  expired requests, but the state may already have changed.

## Recipes

**"Organise my desktop into Work, Games and Temp fences"**
```
pecofence-cli snapshot save before-organise
pecofence-cli monitor list                      # pick workArea of the primary monitor
pecofence-cli fence create --title Work  --rect 40,40,520,360
pecofence-cli fence create --title Games --rect 600,40,520,360
pecofence-cli fence create --title Temp  --rect 1160,40,520,360
pecofence-cli item list                          # decide per item (name/path) where it belongs
pecofence-cli item move --glob "*.lnk" --to Games          # e.g. game shortcuts
pecofence-cli item move "C:\Users\me\Desktop\Q3 plan.docx" "budget" --to Work
pecofence-cli item move --glob "*.tmp" --to Temp
```

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

- No file dialogs: config import/export and backup restore stay in the GUI.
- Tabs hosted inside another fence have no geometry: `fence move/resize` on a tab fails with
  `unsupported`; address the host fence or `fence detach` first.
- Coordinates are physical pixels in virtual-screen space (multi-monitor: monitors left of the
  primary have negative x). Read `monitor list` for work areas.
- `settings set autostart ...` on the Microsoft Store build opens Windows' Startup Apps page instead.
- `--instance <name>` / `PECOFENCE_INSTANCE` address a separately started test instance only; normal
  users never need it.

## AGENTS.md snippet

```
PecoFence (desktop fences) is controlled with `pecofence-cli`: JSON output, exit 3 = app not running.
Run `pecofence-cli describe` for the command catalog and `pecofence-cli skill` for usage rules.
Take `pecofence-cli snapshot save <name>` before bulk changes; never edit its config.json directly.
```
