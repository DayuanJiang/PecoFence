# pecofence-cli

`pecofence-cli.exe` is a small console program shipped next to `pecofence.exe`. It drives the
PecoFence instance that is already running on your desktop and prints JSON, so it works equally
well for people in a terminal and for coding agents (Claude Code, Codex, ...). It never edits
`config.json` itself; if the app is not running it says so and exits with code 3.

## Install and PATH

- **Microsoft Store build**: `pecofence-cli` is on `PATH` through an App Execution Alias, so any
  terminal can run it right after installation. The alias can be turned off in
  *Settings › Apps › Advanced app settings › App execution aliases*.
- **Portable ZIP**: run it from the extracted folder (`.\pecofence-cli.exe ...`) or add that folder
  to `PATH`. `SKILL.md` at the root of the ZIP is the same text `pecofence-cli skill` prints.

## How it talks to the app

The app listens on the named pipe `\\.\pipe\PecoFence` (or `\\.\pipe\PecoFence.<instance>` for a
named test instance). Every CLI call opens one connection, sends one JSON request line, reads one
JSON response line and exits; there is no daemon and nothing stays resident. Requests are executed on
the app's UI thread, so while a PecoFence context menu or dialog is open the reply waits; after
`--timeout` (default 15 s) the CLI gives up with exit code 4 and the app discards the request instead
of running it late. A request the app picked up right before that deadline may still have run, so
after an exit 4 read the state back (`fence list`) before retrying a mutation.

## Global flags

| Flag | Meaning |
|---|---|
| `--instance <NAME>` | Talk to a named test instance. Defaults to the `PECOFENCE_INSTANCE` environment variable (the same variable the app reads at start-up). Normal installs never need it. |
| `--timeout <MS>` | How long to wait for the reply (default 15000, minimum 100; smaller values are clamped by both the CLI and the app). |
| `--pretty` / `--compact` | Indented or single-line JSON. Default: pretty when stdout is a terminal, compact otherwise. |
| `-V, --version` | CLI version (the app version is in `status`). |

## Commands

Every leaf has `--help` with an example. `FENCE` accepts an id, a unique id prefix of at least six
hex digits, or a title (exact match first, then a unique case-insensitive substring); the word
`inbox` always means the desktop fence (the one `fence list` shows with `"kind": "inbox"`), whatever
its localised title. `RULE` accepts an id, a 0-based index, or a name. Snapshots accept an id, a
unique id prefix, or a name (only while that name is unique).

| Command | Example |
|---|---|
| `status` | `pecofence-cli status` |
| `monitor list` | `pecofence-cli monitor list` |
| `describe [--schema <NAME>]` | `pecofence-cli describe --schema Settings` |
| `skill` | `pecofence-cli skill > SKILL.md` |
| `fence list` | `pecofence-cli fence list` |
| `fence get <FENCE>` | `pecofence-cli fence get Work` |
| `fence create (--title <T> \| --portal <DIR>) [--rect x,y,w,h] [--monitor <ID>]` | `pecofence-cli fence create --title Work --rect 100,100,600,400` |
| `fence delete <FENCE>` | `pecofence-cli fence delete Temp` |
| `fence rename <FENCE> <TITLE>` | `pecofence-cli fence rename Work Projects` |
| `fence move <FENCE> (--rect x,y,w,h \| --x <N> --y <N> \| --monitor <ID>)` | `pecofence-cli fence move Work --x 1200 --y 80` |
| `fence resize <FENCE> [--w <N>] [--h <N>]` | `pecofence-cli fence resize Work --w 800` |
| `fence set (<FENCE> \| --all) <PROP> <VALUE> [--string]` | `pecofence-cli fence set Work layout list` |
| `fence roll (<FENCE> \| --all)` | `pecofence-cli fence roll Work` |
| `fence unroll (<FENCE> \| --all)` | `pecofence-cli fence unroll --all` |
| `fence dock-top <FENCE>` | `pecofence-cli fence dock-top Work` |
| `fence merge <FENCE> --into <FENCE>` | `pecofence-cli fence merge Games --into Work` |
| `fence detach <FENCE>` | `pecofence-cli fence detach Games` |
| `fence hide-all` / `fence show-all` | `pecofence-cli fence hide-all` |
| `fence open-options <FENCE>` | `pecofence-cli fence open-options Work` |
| `item list [--fence <FENCE>]` | `pecofence-cli item list --fence inbox` |
| `item move (<ITEM>... \| --glob <PATTERN> [--from <FENCE>]) --to <FENCE>` | `pecofence-cli item move --glob "*.pdf" --from inbox --to Docs` |
| `settings get [PATH]` | `pecofence-cli settings get peek.enabled` |
| `settings set <PATH> <VALUE> [--string]` | `pecofence-cli settings set peek.enabled false` |
| `settings open-ui` | `pecofence-cli settings open-ui` |
| `rule list` | `pecofence-cli rule list` |
| `rule add --name <N> --to <FENCE\|inbox> <conditions...> [--index <N>]` | `pecofence-cli rule add --name PDFs --ext pdf --to Docs` |
| `rule remove <RULE>` | `pecofence-cli rule remove PDFs` |
| `rule enable <RULE>` / `rule disable <RULE>` | `pecofence-cli rule disable 0` |
| `rule move <RULE> --to <INDEX>` | `pecofence-cli rule move PDFs --to 0` |
| `rule import <FILE\|->` | `pecofence-cli rule import rules.json` |
| `rule apply` | `pecofence-cli rule apply` |
| `snapshot list` | `pecofence-cli snapshot list` |
| `snapshot save <NAME>` | `pecofence-cli snapshot save before-cleanup` |
| `snapshot restore <ID>` | `pecofence-cli snapshot restore 3f9c2a1e` |
| `snapshot delete <ID>` | `pecofence-cli snapshot delete 3f9c2a1e` |
| `config export <FILE>` | `pecofence-cli config export C:\Users\me\Desktop\pecofence.json` (absolute path, overwrites) |
| `config import <FILE>` | `pecofence-cli config import C:\Users\me\Desktop\pecofence.json` (replaces everything; layout snapshotted first) |
| `backup list` | `pecofence-cli backup list` (daily backups beside config.json, newest first) |
| `backup restore <FILE>` | `pecofence-cli backup restore "C:\Users\me\AppData\Roaming\PecoFence\backups\2026-09-23.json"` (a path from `backup list`) |
| `peek start` / `peek end` | `pecofence-cli peek start` |

`fence create --portal <DIR>` shows a folder as a fence; `--title` is optional there and defaults to
the folder name.

`fence set` properties (values as shown by `fence get`): `title` (or `fence rename`), `iconSize`
32/48/64/96, `spacing` compact/normal/loose, `autoHeight`, `locked`, `excludeFromQuickHide`,
`opacity` default/clear/solid, `tint` `"#RRGGBB"` or `null`, `titleColor`
theme/tint/white/black/`"#RRGGBB"`, `titleSize` small/normal/large, `layout` icons/list/details,
`sort` manual/name/type/date/size/openCount, `reverse`, `groupByDate`, `labelLines`,
`portalNavigate`, `portalTitleIcon`. `--all` applies the option to every fence one call at a time and
skips tabs hosted inside another fence (their options belong to the host).

`rule add` conditions (all must match): `--ext pdf,docx`, `--type programs,folders,documents,images,
music,video,archives,shortcuts,installers`, `--name-contains`, `--name-not-contains`, `--starts-with`,
`--ends-with`, `--name-is`, `--glob "*.bak"`, `--folders-only` / `--files-only`, and `--json` with an
array of `Cond` objects (`describe --schema Cond`) for everything else (size, creation time, idle days).

`rule import` reads UTF-8 (a BOM is fine). Windows PowerShell 5 writes UTF-16 with `>`; such a file
is rejected with `invalid_value` and the hint to re-save it (`| Set-Content -Encoding utf8`).

`settings set` paths are dotted camelCase keys of the Settings object, e.g. `iconSize`, `theme`,
`themeStyle`, `hideRealIcons`, `autostart`, `peek.enabled`, `quickHide.enabled`, `quickHide.delayMs`,
`rollUp.hoverPeek`, `rollUp.clickToExpand`, `snapping.enabled`, `snapping.gapPx`, `icons.chameleon`.
`describe --schema Settings` lists them all with their allowed values (the name is matched
case-insensitively; an unknown name is a `usage` error listing `details.allowed`).

### Values and shell quoting

`fence set` and `settings set` parse `VALUE` as JSON first (`48`, `true`, `null`, `"dark"`, `[1,2]`);
anything that is not JSON is taken as a string, so `list` and `dark` need no quotes. `--string` forces
a string even for values that look like JSON (`fence set Work title 2024 --string`). Negative numbers
work as they are: `settings set snapping.gapPx -4`.

Quote values that start with `#`, `[` or `*`, or that contain spaces, so the shell does not swallow
them: in Git Bash and other POSIX shells `#ff8800` starts a comment, so write
`fence set Work tint "#ff8800"` (or `'"#ff8800"'`); in PowerShell `"#ff8800"` works as well. A bare
`#` makes the CLI see two arguments and fail with `usage: missing FENCE`, whose message repeats this
rule.

## Folder portals move real files

A portal fence is a live view of a folder on disk. `item move` with a portal as the source or the
destination is therefore an Explorer file move: the file leaves or enters that folder. Undo is only
possible through Explorer (Ctrl+Z on the desktop); layout snapshots do not cover file moves. For that
reason `item move --glob` without `--from` never touches items shown by a portal
(`"assignedBy": "portal"` in `item list`); name the portal with `--from <portal>` to include them. As
a rule, always pair `--glob` with `--from`, and check `fence list` for `"kind": "portal"` before
choosing `--to`.

## Output contract

- Success: the result on stdout. Read-only commands return the object or array itself; every mutating
  command returns `{"changed": bool, ...}` plus the affected object (`fence`, `rule`, `settings`,
  ...). `changed: false` means the state was already as requested; it is not an error.
- `snapshotId` is present only on the commands that take an automatic layout snapshot first:
  `fence delete` when the fence has items, `rule apply` when it moves something, and
  `snapshot restore`. Snapshots record fence layouts (fences, geometry, item membership) only, never
  settings or rules, so `settings set` and `rule import` do not return one. Automatic snapshots never
  evict user snapshots: at the 20-snapshot limit none is taken and the result carries a `warning`.
- Batches (`fence set --all`, `fence roll --all`, `fence unroll --all`) return
  `{"changed": <any>, "results": [{"fence", "title", "changed"} | {"fence", "title", "error"} |
  {"fence", "title", "skipped": "tab"}]}` and exit 1 if any entry failed. A connection-level error
  (`not_running`, `timeout`, `busy`, `version_mismatch`) aborts the batch at once and is reported as
  the command's error with its own exit code, instead of being repeated for every fence.
- A `warning` string may be added when the command was applied but something secondary failed
  (typically: the config file could not be written, or no automatic snapshot could be taken).
- Failure: stderr gets `{"error": {"code", "message", "hint"?, "details"?}}`. Nothing else is ever
  printed to stdout, so `pecofence-cli ... | jq` is always safe.
- Usage errors detected by the argument parser keep clap's human-readable message and exit code 2.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | The app returned an error (any error code not listed below), or part of a batch failed |
| 2 | Usage error (bad flags, `usage` from the app) |
| 3 | PecoFence is not running (`not_running`) |
| 4 | No reply within `--timeout` (`timeout`); run `fence list` before retrying, the command may have run |

## Error codes

| `code` | Meaning |
|---|---|
| `not_running` | No PecoFence instance is listening on the pipe (checked by the CLI). |
| `timeout` | No reply within `--timeout`; the app is probably inside a menu or dialog. |
| `busy` | Too many concurrent CLI connections; retry shortly. |
| `usage` | Malformed request, unknown method, missing or mistyped parameter. |
| `version_mismatch` | CLI and app speak different protocol versions; update both. |
| `fence_not_found` | No fence matches the selector. |
| `ambiguous_fence` | Several fences match; `details.candidates` lists `{id, title}`. |
| `item_not_found` | No item matches the id, path, name, or glob. |
| `ambiguous_item` | Several items share that display name; use the path or id. |
| `rule_not_found` | No rule matches the id, index, or name. |
| `snapshot_not_found` | No snapshot matches the id or name, or several share that name (`details.candidates`; use the id). |
| `invalid_value` | A parameter is out of range or not one of `details.allowed` (also: off-screen or oversized rects, names over 256 characters, a non-UTF-8 rules file). |
| `invalid_path` | The `settings` path does not exist in Settings. |
| `validation_failed` | The patched object did not deserialize; `details.expected` says what was expected. |
| `limit_reached` | The maximum number of fences or snapshots already exists. |
| `unsupported` | The operation does not apply to this target (inbox fence, hosted tab, ...). |
| `internal` | Unexpected failure in the app or in the transport. |

## Units and coordinates

`--rect`, `--x`, `--y`, `--w`, `--h` and every `rect` / `windowRect` / `workArea` in the output are
**physical pixels in virtual-screen coordinates**: the primary monitor's top-left corner is `0,0`,
monitors placed to the left of it have negative `x`. `monitor list` shows each monitor's `rect`,
`workArea` and `dpi`. A fence's `rect` is its expanded size (what `fence move`/`resize` change);
`windowRect` is what is currently on screen (only the title bar while rolled up). The saved,
DPI-independent form is in `geometry`. Rects that lie off every monitor or exceed the virtual screen
are rejected with `invalid_value`.

## Snapshots

`snapshot save <NAME>` stores the current fence layout and returns `{"changed": true, "snapshot":
{"id", "name", "ts", "fenceCount"}}`. Keep `snapshot.id`: `snapshot restore <ID>` (full id or a
unique prefix) is unambiguous, whereas a name only resolves while exactly one snapshot carries it.
Restoring snapshots the current layout first and returns that `snapshotId`, so a restore can itself
be undone. Snapshots do not include settings, rules, or files moved through portals.

## Test instances

`PECOFENCE_INSTANCE=<name> pecofence.exe ...` starts an independent instance with its own pipe
`\\.\pipe\PecoFence.<name>`. `pecofence-cli --instance <name>` (or the same environment variable)
addresses it. This exists for development and automated tests; a normal installation has exactly one
instance and needs neither.

## Limits

- `config import` and `backup restore` replace the whole configuration (settings, rules, layouts). The
  current layout is snapshotted first, but snapshots do not cover settings or rules; run
  `config export` first if you may want the old values back.
- Tabs hosted inside another fence's window share its geometry: `fence move`/`resize` on a tab fails
  with `unsupported` (address the host fence, or `fence detach` first), and `--all` batches skip tabs.
- Deleting a fence also deletes the rules that target it; its items return to the desktop fence.
  The desktop (inbox) fence cannot be deleted.
- Titles, rule names and snapshot names are limited to 256 characters; `rule add` needs at least one
  condition.
- Changing `autostart` on the Microsoft Store build opens Windows' Startup Apps settings page instead
  of flipping the value silently (packaged apps cannot register themselves).
- The CLI does not work while the app is not running; there is no offline mode.

## For coding agents

`pecofence-cli skill` prints a SKILL.md written for Claude Code / Codex (when to use the tool, output
contract, safety rules, recipes). Save it into your skills folder, e.g.
`pecofence-cli skill > .claude/skills/pecofence-cli/SKILL.md`, and paste the short AGENTS.md snippet at
its end into your project's `AGENTS.md`. `pecofence-cli describe` prints the machine-readable catalog
the skill refers to.
