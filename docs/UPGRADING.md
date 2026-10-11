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

## Switching between the ZIP and the installer

Releases offer `pecofence-v<version>-x64-portable.zip` and
`pecofence-v<version>-x64-setup.exe`, with the same program inside. Both keep
settings in `%APPDATA%\PecoFence`, so switching keeps your fences, rules and
settings. Exit PecoFence from its tray menu first.

- **ZIP to installer:** run the setup EXE. It can install into the folder you
  extracted the ZIP into and replaces the program files there. If you install
  somewhere else, you can delete the old ZIP folder afterwards.
- **Installer to ZIP:** uninstall through Windows Settings (your data is kept),
  then extract the ZIP and run `pecofence.exe`.
- If you started the ZIP with `--portable`, its configuration is in the `config`
  folder beside the executable, which other copies do not read. Use **Export or
  import configuration** in Settings to carry it over.

## Going back to 0.1.3 or earlier

Versions after 0.1.3 know a monitor by its model and connection. 0.1.3 and
earlier do not, so with a configuration a newer version has saved they show no
fences. Nothing is lost: the fences come back as soon as the newer version runs
again. To stay on the older version, restore a daily backup from before the
upgrade in Settings → 布局与备份 (the files are in the `backups` folder next to
`config.json`).

## Windows startup

A normal release launch registers the `PecoFence` logon task in Task Scheduler.
It starts at sign-in, 20 to 30 seconds before Explorer gets to the Run key on a PC
with a few startup programs. When startup is on, the running release points the
task at its own executable, so the copy that ran last starts with Windows. Earlier
versions' `PecoFence` and `openFence` Run-key entries are removed; the Run key is
used only when Task Scheduler refuses the task.

Portable and development launches do not change startup at launch. Turning the
switch on in a portable copy's Settings registers the task with `--portable`, so it
reads the `config` folder beside it again.

## Environment overrides

Use the `PECOFENCE_` prefix for application overrides, for example `PECOFENCE_INSTANCE` and
`PECOFENCE_ACRYLIC`. Existing `OPENFENCE_` overrides are still accepted; when both
are defined, the `PECOFENCE_` value wins.
