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
  to `PATH`. `skills\pecofence-cli\SKILL.md` in the ZIP is the same text `pecofence-cli skill` prints.

## How it talks to the app

The app listens on the named pipe `\\.\pipe\PecoFence` (or `\\.\pipe\PecoFence.<instance>` for a
named test instance). Every CLI call opens one connection, sends one JSON request line, reads one
JSON response line and exits; there is no daemon and nothing stays resident. Requests are executed on
the app's UI thread, so while a PecoFence context menu or dialog is open the reply waits; after
`--timeout` (default 15 s) the CLI gives up with exit code 4 and the app discards the request instead
of running it late.

## Global flags

| Flag | Meaning |
|---|---|
| `--instance <NAME>` | Talk to a named test instance. Defaults to the `PECOFENCE_INSTANCE` environment variable (the same variable the app reads at start-up). Normal installs never need it. |
| `--timeout <MS>` | How long to wait for the reply (default 15000). |
| `--pretty` / `--compact` | Indented or single-line JSON. Default: pretty when stdout is a terminal, compact otherwise. |
| `-V, --version` | CLI version (the app version is in `status`). |

## Commands

Every leaf has `--help` with an example. `FENCE` accepts an id, a unique id prefix of at least six
hex digits, or a title (exact match first, then a unique case-insensitive substring). `RULE` accepts
an id, a 0-based index, or a name. Snapshots accept an id or a name.

| Command | Example |
|---|---|
| `status` | `pecofence-cli status` |
| `monitor list` | `pecofence-cli monitor list` |
| `describe [--schema <NAME>]` | `pecofence-cli describe --schema Settings` |
| `skill` | `pecofence-cli skill > SKILL.md` |
| `fence list` | `pecofence-cli fence list` |
| `fence get <FENCE>` | `pecofence-cli fence get Work` |
| `fence create --title <T> [--rect x,y,w,h] [--monitor <ID>] [--portal <DIR>]` | `pecofence-cli fence create --title Work --rect 100,100,600,400` |
| `fence delete <FENCE>` | `pecofence-cli fence delete Temp` |
| `fence rename <FENCE> <TITLE>` | `pecofence-cli fence rename Work Projects` |
| `fence move <FENCE> (--rect x,y,w,h \| --x <N> --y <N> \| --monitor <ID>)` | `pecofence-cli fence move Work --x 1200 --y 80` |
| `fence resize <FENCE> [--w <N>] [--h <N>]` | `pecofence-cli fence resize Work --w 800` |
| `fence set (<FENCE> \| --all) <PROP> <VALUE>` | `pecofence-cli fence set Work layout list` |
| `fence roll (<FENCE> \| --all)` | `pecofence-cli fence roll Work` |
| `fence unroll (<FENCE> \| --all)` | `pecofence-cli fence unroll --all` |
| `fence dock-top <FENCE>` | `pecofence-cli fence dock-top Work` |
| `fence merge <FENCE> --into <FENCE>` | `pecofence-cli fence merge Games --into Work` |
| `fence detach <FENCE>` | `pecofence-cli fence detach Games` |
| `fence hide-all` / `fence show-all` | `pecofence-cli fence hide-all` |
| `fence open-options <FENCE>` | `pecofence-cli fence open-options Work` |
| `item list [--fence <FENCE>]` | `pecofence-cli item list --fence Work` |
| `item move (<ITEM>... \| --glob <PATTERN> [--from <FENCE>]) --to <FENCE>` | `pecofence-cli item move --glob "*.pdf" --to Docs` |
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
| `snapshot restore <ID>` | `pecofence-cli snapshot restore before-cleanup` |
| `snapshot delete <ID>` | `pecofence-cli snapshot delete before-cleanup` |
| `peek start` / `peek end` | `pecofence-cli peek start` |

`fence set` properties (values as shown by `fence get`): `iconSize` 32/48/64/96, `spacing`
compact/normal/loose, `autoHeight`, `locked`, `excludeFromQuickHide`, `opacity` default/clear/solid,
`tint` `#RRGGBB` or `null`, `titleColor` theme/tint/white/black/`#RRGGBB`, `titleSize`
small/normal/large, `layout` icons/list/details, `sort` manual/name/type/date/size/openCount,
`reverse`, `groupByDate`, `labelLines`, `portalNavigate`, `portalTitleIcon`.

`rule add` conditions (all must match): `--ext pdf,docx`, `--type programs,folders,documents,images,
music,video,archives,shortcuts,installers`, `--name-contains`, `--name-not-contains`, `--starts-with`,
`--ends-with`, `--name-is`, `--glob "*.bak"`, `--folders-only` / `--files-only`, and `--json` with an
array of `Cond` objects (`describe --schema Cond`) for everything else (size, creation time, idle days).

`settings set` paths are dotted camelCase keys of the Settings object, e.g. `iconSize`, `theme`,
`themeStyle`, `hideRealIcons`, `autostart`, `peek.enabled`, `quickHide.enabled`, `quickHide.delayMs`,
`rollUp.hoverPeek`, `rollUp.clickToExpand`, `snapping.enabled`, `snapping.gapPx`, `icons.chameleon`.
`describe --schema Settings` lists them all with their allowed values. Values are parsed as JSON first
(`48`, `true`, `null`, `"dark"`), anything else is taken as a string; `--string` forces a string.

## Output contract

- Success: the result on stdout. Read-only commands return the object or array itself; every mutating
  command returns `{"changed": bool, ...}` plus the affected object (`fence`, `rule`, `settings`,
  ...). `changed: false` means the state was already as requested; it is not an error. Commands that
  take an automatic safety snapshot include `snapshotId`.
- Batches (`fence set --all`, `fence roll --all`, `fence unroll --all`) return
  `{"changed": <any>, "results": [{"fence", "title", "changed"} | {"fence", "title", "error"}]}` and
  exit 1 if any entry failed.
- A `warning` string may be added when the command was applied but something secondary failed
  (typically: the config file could not be written).
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
| 4 | No reply within `--timeout` (`timeout`) |

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
| `snapshot_not_found` | No snapshot matches the id or name. |
| `invalid_value` | A parameter is out of range or not one of `details.allowed`. |
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
DPI-independent form is in `geometry`.

## Test instances

`PECOFENCE_INSTANCE=<name> pecofence.exe ...` starts an independent instance with its own pipe
`\\.\pipe\PecoFence.<name>`. `pecofence-cli --instance <name>` (or the same environment variable)
addresses it. This exists for development and automated tests; a normal installation has exactly one
instance and needs neither.

## Limits

- No file dialogs: configuration import/export and backup restore remain GUI-only.
- Tabs hosted inside another fence's window have no geometry; `fence move`/`resize` on a tab fails
  with `unsupported` (address the host fence, or `fence detach` first).
- Deleting a fence also deletes the rules that target it; its items return to the desktop fence.
  The desktop (inbox) fence cannot be deleted.
- Changing `autostart` on the Microsoft Store build opens Windows' Startup Apps settings page instead
  of flipping the value silently (packaged apps cannot register themselves).
- The CLI does not work while the app is not running; there is no offline mode.

## For coding agents

`pecofence-cli skill` prints a SKILL.md written for Claude Code / Codex (when to use the tool, output
contract, safety rules, recipes). Save it into your skills folder, e.g.
`pecofence-cli skill > .claude/skills/pecofence-cli/SKILL.md`, and paste the short AGENTS.md snippet at
its end into your project's `AGENTS.md`. `pecofence-cli describe` prints the machine-readable catalog
the skill refers to.
