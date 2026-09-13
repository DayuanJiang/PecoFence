# Upgrading to PecoFence

PecoFence is the new name of this project. The executable is now `pecofence.exe`,
with `pecofence-watchdog.exe` beside it.

## Existing installations

Exit the older openFence application before starting PecoFence. Both names share
a compatibility instance lock so two versions cannot manage the same desktop at once.

New installations save configuration in `%APPDATA%\PecoFence`. If that location
has no configuration or backups and `%APPDATA%\OpenFence` contains an existing
installation, PecoFence continues using the old directory **in place**. Nothing
is copied or rewritten simply to change the product name. Existing PecoFence data
always takes priority.

This preserves fence layouts, rules, language preferences, snapshots and backups.
Filenames and custom fence/rule names are not renamed. Portable mode continues
using the `config` directory beside the executable.

New logs and WebView2 profiles use `%LOCALAPPDATA%\PecoFence`. An outstanding
desktop-icon recovery marker from the older name is recognized.

## Windows startup

A normal release launch registers the `PecoFence` startup entry when required.
The legacy `openFence` entry is removed only after a replacement was registered
successfully, or when startup is disabled. A working PecoFence entry pointing to
another copy is preserved.

Portable and development launches do not change startup entries. Changing the
autostart switch in Settings remains an explicit opt-in/out.

## Environment overrides

Use the `PECOFENCE_` prefix for application overrides, for example `PECOFENCE_INSTANCE` and
`PECOFENCE_ACRYLIC`. Existing `OPENFENCE_` overrides are still accepted; when both
are defined, the `PECOFENCE_` value wins.
