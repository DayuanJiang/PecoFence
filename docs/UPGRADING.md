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

A normal release launch registers a `PecoFence` Task Scheduler logon task when
startup is enabled. It runs at Above Normal priority as the current user, without
administrator rights or a configured startup delay. If task registration fails,
PecoFence uses the Windows Run key as a fallback. The legacy `openFence` task and
Run value are removed when the replacement succeeds or startup is disabled. The
task follows the release that ran last.

Portable launches also honor the autostart setting. Their task (or Run-key fallback)
includes `--portable` so the next logon uses the `config` directory beside that
executable. Development builds do not change startup entries. Microsoft Store
installations continue to use the manifest's Windows startup task.

## Environment overrides

Use the `PECOFENCE_` prefix for application overrides, for example `PECOFENCE_INSTANCE` and
`PECOFENCE_ACRYLIC`. Existing `OPENFENCE_` overrides are still accepted; when both
are defined, the `PECOFENCE_` value wins.
